//! Tests for the converter module
//!
//! Run with: cargo test

use clash_sub::converter::{convert_subscription, get_proxy_name};
use serde_yaml::Value;

// ============================================================================
// Tests for get_proxy_name
// ============================================================================

mod get_proxy_name_tests {
    use super::*;

    #[test]
    fn test_get_name_from_valid_proxy() {
        let proxy: Value = serde_yaml::from_str(
            r#"
            name: "香港-01"
            type: ss
            server: hk1.example.com
            port: 443
            "#,
        )
        .unwrap();

        assert_eq!(get_proxy_name(&proxy), Some("香港-01".to_string()));
    }

    #[test]
    fn test_get_name_with_special_characters() {
        let proxy: Value = serde_yaml::from_str(
            r#"
            name: "🇭🇰 香港 Premium-01"
            type: vmess
            server: hk.example.com
            port: 443
            "#,
        )
        .unwrap();

        assert_eq!(
            get_proxy_name(&proxy),
            Some("🇭🇰 香港 Premium-01".to_string())
        );
    }

    #[test]
    fn test_get_name_missing() {
        let proxy: Value = serde_yaml::from_str(
            r#"
            type: ss
            server: example.com
            port: 443
            "#,
        )
        .unwrap();

        assert_eq!(get_proxy_name(&proxy), None);
    }

    #[test]
    fn test_get_name_null_value() {
        let proxy: Value = serde_yaml::from_str(
            r#"
            name: null
            type: ss
            "#,
        )
        .unwrap();

        assert_eq!(get_proxy_name(&proxy), None);
    }

    #[test]
    fn test_get_name_empty_string() {
        let proxy: Value = serde_yaml::from_str(
            r#"
            name: ""
            type: ss
            "#,
        )
        .unwrap();

        assert_eq!(get_proxy_name(&proxy), Some("".to_string()));
    }
}

// ============================================================================
// Tests for convert_subscription
// ============================================================================

mod convert_subscription_tests {
    use super::*;

    fn create_test_yaml() -> String {
        r#"
proxies:
  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
    cipher: aes-256-gcm
    password: password123
  - name: "香港-02"
    type: ss
    server: hk2.example.com
    port: 443
    cipher: aes-256-gcm
    password: password123
  - name: "台湾-01"
    type: ss
    server: tw1.example.com
    port: 443
    cipher: aes-256-gcm
    password: password123
  - name: "日本-01"
    type: ss
    server: jp1.example.com
    port: 443
    cipher: aes-256-gcm
    password: password123
  - name: "US-01"
    type: ss
    server: us1.example.com
    port: 443
    cipher: aes-256-gcm
    password: password123
"#
        .to_string()
    }

    #[test]
    fn test_basic_conversion() {
        let input = create_test_yaml();
        let result = convert_subscription(&input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Check for common anchor definition at top
        assert!(yaml.contains(".lb_common: &lb_common"));
        assert!(yaml.contains("url: http://www.gstatic.com/generate_204"));
        assert!(yaml.contains("interval: 180"));
        assert!(yaml.contains("strategy: consistent-hashing"));

        // Check for proxy groups
        assert!(yaml.contains("name: 默认流量"));
        assert!(yaml.contains("name: 节点选择"));
        assert!(yaml.contains("name: 全部节点负载组"));
        assert!(yaml.contains("name: 直接连接"));
    }

    #[test]
    fn test_region_groups_created() {
        let input = create_test_yaml();
        let result = convert_subscription(&input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Check for region groups
        assert!(yaml.contains("name: 香港负载组"));
        assert!(yaml.contains("name: 台湾负载组"));
        assert!(yaml.contains("name: 日本负载组"));
        assert!(yaml.contains("name: 美国负载组"));
    }

    #[test]
    fn test_merge_references() {
        let input = create_test_yaml();
        let result = convert_subscription(&input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Check that load-balance groups use merge reference
        assert!(yaml.contains("<<: *lb_common"));
    }

    #[test]
    fn test_node_selector_has_all_proxies() {
        let input = create_test_yaml();
        let result = convert_subscription(&input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Parse the result to check node selector
        let parsed: Value = serde_yaml::from_str(&yaml).unwrap();
        let groups = parsed["proxy-groups"].as_sequence().unwrap();

        let node_selector = groups
            .iter()
            .find(|g| g["name"].as_str() == Some("节点选择"))
            .unwrap();

        let proxies = node_selector["proxies"].as_sequence().unwrap();
        assert_eq!(proxies.len(), 5);
        assert!(proxies.iter().any(|p| p.as_str() == Some("香港-01")));
        assert!(proxies.iter().any(|p| p.as_str() == Some("台湾-01")));
    }

    #[test]
    fn test_rules_order() {
        let input = create_test_yaml();
        let result = convert_subscription(&input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        let parsed: Value = serde_yaml::from_str(&yaml).unwrap();
        let rules = parsed["rules"].as_sequence().unwrap();

        assert_eq!(rules.len(), 7);
        assert_eq!(rules[0].as_str(), Some("GEOSITE,private,直接连接"));
        assert_eq!(rules[1].as_str(), Some("GEOSITE,CN,直接连接"));
        assert_eq!(rules[2].as_str(), Some("GEOSITE,apple-cn,直接连接"));
        assert_eq!(rules[3].as_str(), Some("GEOSITE,steam@cn,直接连接"));
        assert_eq!(
            rules[4].as_str(),
            Some("GEOSITE,category-games@cn,直接连接")
        );
        assert_eq!(rules[5].as_str(), Some("GEOIP,CN,直接连接"));
        assert_eq!(rules[6].as_str(), Some("MATCH,默认流量"));
    }

    #[test]
    fn test_geosite_rules_present() {
        let input = create_test_yaml();
        let result = convert_subscription(&input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Check for GEOSITE rules
        assert!(yaml.contains("GEOSITE,private,直接连接"));
        assert!(yaml.contains("GEOSITE,CN,直接连接"));
        assert!(yaml.contains("GEOSITE,apple-cn,直接连接"));
        assert!(yaml.contains("GEOSITE,steam@cn,直接连接"));
        assert!(yaml.contains("GEOSITE,category-games@cn,直接连接"));
    }

    #[test]
    fn test_no_fixed_settings() {
        let input = create_test_yaml();
        let result = convert_subscription(&input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Should not contain fixed settings
        assert!(!yaml.contains("port: 7890"));
        assert!(!yaml.contains("socks-port:"));
        assert!(!yaml.contains("allow-lan:"));
        assert!(!yaml.contains("mode:"));
        assert!(!yaml.contains("log-level:"));
    }

    #[test]
    fn test_invalid_yaml() {
        let input = "this is not valid yaml: [[[";
        let result = convert_subscription(input);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_empty_proxies() {
        let input = r#"
proxies: []
"#;
        let result = convert_subscription(input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Should still generate basic structure
        assert!(yaml.contains("proxy-groups:"));
        assert!(yaml.contains("rules:"));
    }

    #[test]
    fn test_only_one_region() {
        let input = r#"
proxies:
  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
  - name: "香港-02"
    type: ss
    server: hk2.example.com
    port: 443
"#;
        let result = convert_subscription(input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Should have Hong Kong group
        assert!(yaml.contains("name: 香港负载组"));

        // Should have "其他负载组" (always included)
        assert!(yaml.contains("name: 其他负载组"));

        // Should NOT have Taiwan group (no matching proxies)
        let lines: Vec<&str> = yaml.lines().collect();
        let taiwan_count = lines
            .iter()
            .filter(|line| line.contains("台湾负载组"))
            .count();

        // Taiwan should appear in default traffic group list, but not as a separate group
        // So it should appear less than if it had its own group definition
        assert!(taiwan_count <= 1);
    }

    #[test]
    fn test_default_traffic_first_options() {
        let input = create_test_yaml();
        let result = convert_subscription(&input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        let parsed: Value = serde_yaml::from_str(&yaml).unwrap();
        let groups = parsed["proxy-groups"].as_sequence().unwrap();

        let default_traffic = groups
            .iter()
            .find(|g| g["name"].as_str() == Some("默认流量"))
            .unwrap();

        let proxies = default_traffic["proxies"].as_sequence().unwrap();

        // First option should be "节点选择"
        assert_eq!(proxies[0].as_str(), Some("节点选择"));

        // Second option should be "直接连接"
        assert_eq!(proxies[1].as_str(), Some("直接连接"));

        // Third should be "全部节点负载组"
        assert_eq!(proxies[2].as_str(), Some("全部节点负载组"));
    }
}

// ============================================================================
// Integration tests
// ============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow_with_mixed_regions() {
        let input = r#"
proxies:
  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
  - name: "HK-02"
    type: vmess
    server: hk2.example.com
    port: 443
  - name: "台湾节点1"
    type: trojan
    server: tw1.example.com
    port: 443
  - name: "JP-Tokyo-01"
    type: ss
    server: jp1.example.com
    port: 443
  - name: "日本大阪"
    type: ss
    server: jp2.example.com
    port: 443
  - name: "Singapore-SG1"
    type: ss
    server: sg1.example.com
    port: 443
  - name: "US-LA-01"
    type: ss
    server: us1.example.com
    port: 443
  - name: "UnknownNode"
    type: ss
    server: unknown.example.com
    port: 443
"#;
        let result = convert_subscription(input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Parse result
        let parsed: Value = serde_yaml::from_str(&yaml).unwrap();

        // Check proxies are preserved
        let proxies = parsed["proxies"].as_sequence().unwrap();
        assert_eq!(proxies.len(), 8);

        // Check groups exist
        let groups = parsed["proxy-groups"].as_sequence().unwrap();
        let group_names: Vec<String> = groups
            .iter()
            .map(|g| g["name"].as_str().unwrap().to_string())
            .collect();

        assert!(group_names.contains(&"默认流量".to_string()));
        assert!(group_names.contains(&"节点选择".to_string()));
        assert!(group_names.contains(&"全部节点负载组".to_string()));
        assert!(group_names.contains(&"香港负载组".to_string()));
        assert!(group_names.contains(&"台湾负载组".to_string()));
        assert!(group_names.contains(&"日本负载组".to_string()));
        assert!(group_names.contains(&"新加坡负载组".to_string()));
        assert!(group_names.contains(&"美国负载组".to_string()));
        assert!(group_names.contains(&"其他负载组".to_string()));
        assert!(group_names.contains(&"直接连接".to_string()));

        // Check rules
        let rules = parsed["rules"].as_sequence().unwrap();
        assert_eq!(rules.len(), 7);

        // Verify GEOSITE rules
        assert_eq!(rules[0].as_str(), Some("GEOSITE,private,直接连接"));
        assert_eq!(rules[1].as_str(), Some("GEOSITE,CN,直接连接"));
        assert_eq!(rules[2].as_str(), Some("GEOSITE,apple-cn,直接连接"));
        assert_eq!(rules[3].as_str(), Some("GEOSITE,steam@cn,直接连接"));
        assert_eq!(
            rules[4].as_str(),
            Some("GEOSITE,category-games@cn,直接连接")
        );
        assert_eq!(rules[5].as_str(), Some("GEOIP,CN,直接连接"));
    }

    #[test]
    fn test_yaml_anchor_appears_once() {
        let input = create_test_yaml();
        let result = convert_subscription(&input);

        assert!(result.is_ok());
        let yaml = result.unwrap();

        // Anchor definition should appear exactly once
        let anchor_count = yaml.matches(".lb_common: &lb_common").count();
        assert_eq!(anchor_count, 1);

        // Merge reference should appear multiple times
        let merge_count = yaml.matches("<<: *lb_common").count();
        assert!(merge_count > 0);
    }

    fn create_test_yaml() -> String {
        r#"
proxies:
  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
  - name: "台湾-01"
    type: ss
    server: tw1.example.com
    port: 443
"#
        .to_string()
    }
}

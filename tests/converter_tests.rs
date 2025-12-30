//! Tests for the converter module
//!
//! Run with: cargo test

use clash_sub::converter::{convert_subscription, extract_region, get_proxy_name, is_info_node};
use serde_yaml::Value;

// ============================================================================
// Tests for is_info_node
// ============================================================================

mod is_info_node_tests {
    use super::*;

    #[test]
    fn test_info_node_with_website() {
        assert!(is_info_node("官网: example.com"));
        assert!(is_info_node("网址：www.test.com"));
    }

    #[test]
    fn test_info_node_with_traffic() {
        assert!(is_info_node("剩余流量：10GB"));
        assert!(is_info_node("流量重置日期"));
    }

    #[test]
    fn test_info_node_with_expiry() {
        assert!(is_info_node("过期时间：2024-12-31"));
        assert!(is_info_node("到期：2025年1月"));
    }

    #[test]
    fn test_info_node_with_subscription() {
        assert!(is_info_node("订阅链接"));
        assert!(is_info_node("套餐到期"));
    }

    #[test]
    fn test_info_node_with_misc_keywords() {
        assert!(is_info_node("剩余：50%"));
        assert!(is_info_node("重置日期"));
        assert!(is_info_node("时间：2024"));
        assert!(is_info_node("TG群：@example"));
        assert!(is_info_node("更新时间"));
    }

    #[test]
    fn test_not_info_node_with_normal_proxy() {
        assert!(!is_info_node("香港-01"));
        assert!(!is_info_node("US-Server"));
        assert!(!is_info_node("Japan 01"));
        assert!(!is_info_node("Singapore"));
    }

    #[test]
    fn test_not_info_node_with_region_names() {
        assert!(!is_info_node("🇭🇰 香港"));
        assert!(!is_info_node("🇯🇵 日本"));
        assert!(!is_info_node("🇺🇸 美国"));
        assert!(!is_info_node("Taiwan"));
    }

    #[test]
    fn test_empty_string() {
        assert!(!is_info_node(""));
    }
}

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
            name: "🇭🇰 Hong Kong | 01"
            type: vmess
            server: test.com
        "#,
        )
        .unwrap();

        assert_eq!(
            get_proxy_name(&proxy),
            Some("🇭🇰 Hong Kong | 01".to_string())
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
    fn test_get_name_numeric_value() {
        let proxy: Value = serde_yaml::from_str(
            r#"
            name: 12345
            type: ss
        "#,
        )
        .unwrap();

        // name is a number, not a string
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
// Tests for extract_region
// ============================================================================

mod extract_region_tests {
    use super::*;

    // --- Delimiter-based extraction ---

    #[test]
    fn test_extract_with_hyphen() {
        assert_eq!(extract_region("香港-01"), Some("香港".to_string()));
        assert_eq!(
            extract_region("US-Server-01"),
            Some("US-Server".to_string())
        );
        assert_eq!(
            extract_region("Japan-Node-001"),
            Some("Japan-Node".to_string())
        );
    }

    #[test]
    fn test_extract_with_underscore() {
        assert_eq!(extract_region("HK_Node_1"), Some("HK_Node".to_string()));
        assert_eq!(extract_region("US_West_01"), Some("US_West".to_string()));
    }

    #[test]
    fn test_extract_with_space() {
        assert_eq!(extract_region("Japan 01"), Some("Japan".to_string()));
        assert_eq!(
            extract_region("Hong Kong 001"),
            Some("Hong Kong".to_string())
        );
    }

    #[test]
    fn test_extract_with_pipe() {
        assert_eq!(
            extract_region("Singapore|01"),
            Some("Singapore".to_string())
        );
        assert_eq!(extract_region("Taiwan｜02"), Some("Taiwan".to_string())); // Full-width pipe
    }

    #[test]
    fn test_extract_with_dot() {
        assert_eq!(extract_region("Korea·03"), Some("Korea".to_string()));
    }

    #[test]
    fn test_extract_with_hash() {
        assert_eq!(extract_region("Germany#05"), Some("Germany".to_string()));
    }

    #[test]
    fn test_extract_with_at() {
        assert_eq!(extract_region("France@01"), Some("France".to_string()));
    }

    // --- Trailing number extraction ---

    #[test]
    fn test_extract_trailing_numbers() {
        assert_eq!(extract_region("香港01"), Some("香港".to_string()));
        assert_eq!(extract_region("日本001"), Some("日本".to_string()));
        assert_eq!(
            extract_region("Singapore123"),
            Some("Singapore".to_string())
        );
    }

    #[test]
    fn test_extract_trailing_numbers_chinese() {
        assert_eq!(extract_region("🇭🇰香港02"), Some("🇭🇰香港".to_string()));
        assert_eq!(extract_region("🇯🇵东京03"), Some("🇯🇵东京".to_string()));
    }

    // --- Edge cases ---

    #[test]
    fn test_no_pattern_match() {
        // Single word without numbers or delimiters
        assert_eq!(extract_region("Singapore"), None);
        assert_eq!(extract_region("香港"), None);
    }

    #[test]
    fn test_only_numbers() {
        // This should return None because prefix would be empty
        assert_eq!(extract_region("01"), None);
        assert_eq!(extract_region("123"), None);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(extract_region(""), None);
    }

    #[test]
    fn test_delimiter_but_not_number_suffix() {
        // Delimiter present but suffix is not purely numeric
        assert_eq!(extract_region("Hong Kong-Premium"), None);
        assert_eq!(extract_region("US-West"), None);
    }

    #[test]
    fn test_complex_names() {
        // Multiple delimiters - should use the last one
        assert_eq!(
            extract_region("🇺🇸 US-West-01"),
            Some("🇺🇸 US-West".to_string())
        );
        assert_eq!(
            extract_region("Premium香港-Node-02"),
            Some("Premium香港-Node".to_string())
        );
    }

    #[test]
    fn test_mixed_chinese_english() {
        assert_eq!(extract_region("香港HK-01"), Some("香港HK".to_string()));
        assert_eq!(
            extract_region("Japan日本-02"),
            Some("Japan日本".to_string())
        );
    }
}

// ============================================================================
// Tests for convert_subscription
// ============================================================================

mod convert_subscription_tests {
    use super::*;

    fn create_test_yaml(proxies: &str) -> String {
        format!(
            r#"
proxies:
{}
"#,
            proxies
        )
    }

    #[test]
    fn test_basic_conversion() {
        let input = create_test_yaml(
            r#"  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "香港-02"
    type: ss
    server: hk2.example.com
    port: 443
    cipher: aes-256-gcm
    password: test"#,
        );

        let result = convert_subscription(&input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("默认代理"));
        assert!(output.contains("香港"));
        assert!(output.contains("load-balance"));
    }

    #[test]
    fn test_info_nodes_separated() {
        let input = create_test_yaml(
            r#"  - name: "剩余流量：10GB"
    type: ss
    server: info.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test"#,
        );

        let result = convert_subscription(&input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("订阅信息")); // Info group should be created
        assert!(output.contains("香港")); // Region group should be created
    }

    #[test]
    fn test_multiple_regions() {
        let input = create_test_yaml(
            r#"  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "日本-01"
    type: ss
    server: jp1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "美国-01"
    type: ss
    server: us1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test"#,
        );

        let result = convert_subscription(&input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("香港"));
        assert!(output.contains("日本"));
        assert!(output.contains("美国"));
    }

    #[test]
    fn test_ungrouped_proxies() {
        let input = create_test_yaml(
            r#"  - name: "SpecialNode"
    type: ss
    server: special.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test"#,
        );

        let result = convert_subscription(&input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("其他")); // Ungrouped proxies go to "其他"
    }

    #[test]
    fn test_output_structure() {
        let input = create_test_yaml(
            r#"  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test"#,
        );

        let result = convert_subscription(&input);
        assert!(result.is_ok());

        let output = result.unwrap();
        // Check required output fields
        assert!(output.contains("port: 7890"));
        assert!(output.contains("socks-port: 7891"));
        assert!(output.contains("allow-lan: true"));
        assert!(output.contains("mode: rule"));
        assert!(output.contains("log-level: info"));
        assert!(output.contains("external-controller: 127.0.0.1:9090"));
        assert!(output.contains("GEOIP,CN,DIRECT"));
        assert!(output.contains("MATCH,默认代理"));
    }

    #[test]
    fn test_invalid_yaml() {
        let input = "invalid yaml content: [[[";

        let result = convert_subscription(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_proxies() {
        let input = r#"
port: 7890
mode: rule
"#;

        let result = convert_subscription(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_proxies() {
        let input = r#"
proxies: []
"#;

        let result = convert_subscription(input);
        assert!(result.is_ok());

        let output = result.unwrap();
        // Should still have the main structure
        assert!(output.contains("默认代理"));
        assert!(output.contains("DIRECT"));
    }

    #[test]
    fn test_proxy_ordering_preserved() {
        let input = create_test_yaml(
            r#"  - name: "日本-01"
    type: ss
    server: jp1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "香港-01"
    type: ss
    server: hk1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "美国-01"
    type: ss
    server: us1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test"#,
        );

        let result = convert_subscription(&input);
        assert!(result.is_ok());

        let output = result.unwrap();
        // Check that regions appear in original order
        let jp_pos = output.find("日本").unwrap();
        let hk_pos = output.find("香港").unwrap();
        let us_pos = output.find("美国").unwrap();

        // First occurrence should maintain order
        assert!(jp_pos < hk_pos);
        assert!(hk_pos < us_pos);
    }

    #[test]
    fn test_proxy_with_vmess_type() {
        let input = r#"
proxies:
  - name: "香港-01"
    type: vmess
    server: hk1.example.com
    port: 443
    uuid: test-uuid-1234
    alterId: 0
    cipher: auto
"#;

        let result = convert_subscription(&input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("vmess"));
        assert!(output.contains("香港-01"));
    }

    #[test]
    fn test_proxy_with_trojan_type() {
        let input = r#"
proxies:
  - name: "日本-01"
    type: trojan
    server: jp1.example.com
    port: 443
    password: test-password
    sni: jp1.example.com
"#;

        let result = convert_subscription(&input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("trojan"));
        assert!(output.contains("日本-01"));
    }
}

// ============================================================================
// Integration tests
// ============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow() {
        // Simulate a real subscription with mixed content
        let input = r#"
proxies:
  - name: "剩余流量：50GB"
    type: ss
    server: info1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "套餐到期：2025-12-31"
    type: ss
    server: info2.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "🇭🇰 香港-01"
    type: vmess
    server: hk1.example.com
    port: 443
    uuid: test-uuid
    alterId: 0
    cipher: auto
  - name: "🇭🇰 香港-02"
    type: vmess
    server: hk2.example.com
    port: 443
    uuid: test-uuid
    alterId: 0
    cipher: auto
  - name: "🇯🇵 日本-01"
    type: trojan
    server: jp1.example.com
    port: 443
    password: test
  - name: "🇺🇸 美国-01"
    type: ss
    server: us1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "SpecialNode"
    type: ss
    server: special.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
"#;

        let result = convert_subscription(input);
        assert!(result.is_ok());

        let output = result.unwrap();

        // Verify structure
        assert!(output.contains("port: 7890"));
        assert!(output.contains("默认代理"));

        // Verify info nodes are grouped
        assert!(output.contains("订阅信息"));

        // Verify regions are detected
        assert!(output.contains("🇭🇰 香港"));
        assert!(output.contains("🇯🇵 日本"));
        assert!(output.contains("🇺🇸 美国"));

        // Verify ungrouped proxies
        assert!(output.contains("其他"));

        // Verify rules
        assert!(output.contains("GEOIP,CN,DIRECT"));
        assert!(output.contains("MATCH,默认代理"));

        // Verify load-balance configuration
        assert!(output.contains("load-balance"));
        assert!(output.contains("consistent-hashing"));
        assert!(output.contains("http://www.gstatic.com/generate_204"));
    }

    #[test]
    fn test_only_info_nodes() {
        let input = r#"
proxies:
  - name: "剩余流量：50GB"
    type: ss
    server: info1.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
  - name: "官网：example.com"
    type: ss
    server: info2.example.com
    port: 443
    cipher: aes-256-gcm
    password: test
"#;

        let result = convert_subscription(input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("订阅信息"));
        // Should not have region groups
        assert!(!output.contains("其他"));
    }
}

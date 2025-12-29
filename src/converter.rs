use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;

/// Represents the input Clash configuration - only extract proxies
#[derive(Debug, Deserialize)]
pub struct InputConfig {
    pub proxies: Vec<Value>,
}

/// Represents the output Clash configuration
#[derive(Debug, Serialize)]
pub struct OutputConfig {
    pub port: u16,
    #[serde(rename = "socks-port")]
    pub socks_port: u16,
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    pub mode: String,
    #[serde(rename = "log-level")]
    pub log_level: String,
    #[serde(rename = "external-controller")]
    pub external_controller: String,
    pub proxies: Vec<Value>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<String>,
}

/// Represents a proxy group
#[derive(Debug, Serialize, Clone)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub proxies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

/// Check if a proxy name is an info node (not a real proxy)
pub fn is_info_node(name: &str) -> bool {
    name.contains("网址")
        || name.contains("流量")
        || name.contains("过期")
        || name.contains("到期")
        || name.contains("官网")
        || name.contains("订阅")
        || name.contains("套餐")
        || name.contains("剩余")
        || name.contains("重置")
        || name.contains("时间")
        || name.contains("群")
        || name.contains("更新")
}

/// Get the proxy name from a proxy value
pub fn get_proxy_name(proxy: &Value) -> Option<String> {
    proxy.get("name")?.as_str().map(|s| s.to_string())
}

/// Extract region name from proxy name
/// Supports patterns like:
/// - "香港-01" -> "香港"
/// - "US-Server-01" -> "US-Server"
/// - "Japan 01" -> "Japan"
/// - "HK_Node_1" -> "HK_Node"
/// - "Singapore|01" -> "Singapore"
pub fn extract_region(name: &str) -> Option<String> {
    // Common delimiters: -, _, space, |, numbers at the end
    let delimiters = ['-', '_', ' ', '|', '·', '｜', '#', '@'];

    // Try to find a delimiter
    for delim in &delimiters {
        if let Some(pos) = name.rfind(*delim) {
            let prefix = &name[..pos];
            let suffix = &name[pos + delim.len_utf8()..];

            // Check if suffix looks like a number/index (e.g., "01", "1", "001")
            if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                if !prefix.is_empty() {
                    return Some(prefix.to_string());
                }
            }
        }
    }

    // Try to match trailing numbers (e.g., "香港01" -> "香港")
    let mut last_non_digit = name.len();
    for (i, c) in name.char_indices().rev() {
        if !c.is_ascii_digit() {
            last_non_digit = i + c.len_utf8();
            break;
        }
    }

    if last_non_digit > 0 && last_non_digit < name.len() {
        let prefix = &name[..last_non_digit];
        if !prefix.is_empty() {
            return Some(prefix.to_string());
        }
    }

    // If no pattern matched, return the whole name as its own region
    // This handles single proxies that don't follow naming conventions
    None
}

/// Error type for conversion
#[derive(Debug)]
pub struct ConvertError(pub String);

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ConvertError {}

/// Convert the subscription content
pub fn convert_subscription(content: &str) -> Result<String, ConvertError> {
    // Parse the input YAML - only extract proxies
    let input: InputConfig = serde_yaml::from_str(content)
        .map_err(|e| ConvertError(format!("Failed to parse YAML: {}", e)))?;

    // Separate info nodes and valid proxies
    let mut info_nodes: Vec<&Value> = Vec::new();
    let mut valid_proxies: Vec<&Value> = Vec::new();

    for proxy in &input.proxies {
        if let Some(name) = get_proxy_name(proxy) {
            if is_info_node(&name) {
                info_nodes.push(proxy);
            } else {
                valid_proxies.push(proxy);
            }
        }
    }

    // Get info node names for the info group
    let info_node_names: Vec<String> = info_nodes
        .iter()
        .filter_map(|p| get_proxy_name(p))
        .collect();

    // Get all valid proxy names
    let proxy_names: Vec<String> = valid_proxies
        .iter()
        .filter_map(|p| get_proxy_name(p))
        .collect();

    // Auto-detect regions and group proxies, preserving original order
    let mut region_proxies: HashMap<String, Vec<String>> = HashMap::new();
    let mut region_order: Vec<String> = Vec::new(); // Track first occurrence order
    let mut ungrouped_proxies: Vec<String> = Vec::new();

    for name in &proxy_names {
        if let Some(region) = extract_region(name) {
            // Track region order by first occurrence
            if !region_proxies.contains_key(&region) {
                region_order.push(region.clone());
            }
            region_proxies.entry(region).or_default().push(name.clone());
        } else {
            // Proxies that don't match any pattern go to ungrouped
            ungrouped_proxies.push(name.clone());
        }
    }

    // Build proxy groups
    let mut proxy_groups: Vec<ProxyGroup> = Vec::new();

    // Main selector group - "默认代理"
    let mut main_group_proxies: Vec<String> = Vec::new();
    // Add info group first if there are info nodes
    if !info_node_names.is_empty() {
        main_group_proxies.push("订阅信息".to_string());
    }
    // Then add region groups
    main_group_proxies.extend(region_order.clone());
    if !ungrouped_proxies.is_empty() {
        main_group_proxies.push("其他".to_string());
    }
    main_group_proxies.push("DIRECT".to_string());

    proxy_groups.push(ProxyGroup {
        name: "默认代理".to_string(),
        group_type: "select".to_string(),
        proxies: main_group_proxies,
        url: None,
        interval: None,
        strategy: None,
    });

    // Add info group (load-balance) if there are info nodes
    if !info_node_names.is_empty() {
        proxy_groups.push(ProxyGroup {
            name: "订阅信息".to_string(),
            group_type: "load-balance".to_string(),
            proxies: info_node_names,
            url: Some("http://www.gstatic.com/generate_204".to_string()),
            interval: Some(300),
            strategy: Some("consistent-hashing".to_string()),
        });
    }

    // Add load-balance group for each region (preserving original order)
    for region in &region_order {
        if let Some(proxies) = region_proxies.get(region) {
            if !proxies.is_empty() {
                let mut sorted_proxies = proxies.clone();
                sorted_proxies.sort();

                proxy_groups.push(ProxyGroup {
                    name: region.clone(),
                    group_type: "load-balance".to_string(),
                    proxies: sorted_proxies,
                    url: Some("http://www.gstatic.com/generate_204".to_string()),
                    interval: Some(300),
                    strategy: Some("consistent-hashing".to_string()),
                });
            }
        }
    }

    // Add ungrouped proxies as "其他" group if any
    if !ungrouped_proxies.is_empty() {
        ungrouped_proxies.sort();
        proxy_groups.push(ProxyGroup {
            name: "其他".to_string(),
            group_type: "load-balance".to_string(),
            proxies: ungrouped_proxies,
            url: Some("http://www.gstatic.com/generate_204".to_string()),
            interval: Some(300),
            strategy: Some("consistent-hashing".to_string()),
        });
    }

    // Build rules - simple: China direct, others proxy
    let rules = vec!["GEOIP,CN,DIRECT".to_string(), "MATCH,默认代理".to_string()];

    // Combine info nodes and valid proxies for output
    let mut all_proxies: Vec<Value> = info_nodes.into_iter().cloned().collect();
    all_proxies.extend(valid_proxies.into_iter().cloned());

    // Build output config with our own settings
    let output = OutputConfig {
        port: 7890,
        socks_port: 7891,
        allow_lan: true,
        mode: "Rule".to_string(),
        log_level: "info".to_string(),
        external_controller: "127.0.0.1:9090".to_string(),
        proxies: all_proxies,
        proxy_groups,
        rules,
    };

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&output)
        .map_err(|e| ConvertError(format!("Failed to serialize YAML: {}", e)))?;

    Ok(yaml)
}

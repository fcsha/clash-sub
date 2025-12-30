use serde::{Deserialize, Serialize};
use serde_yaml::Value;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxies: Option<Vec<String>>,
    #[serde(rename = "include-all", skip_serializing_if = "Option::is_none")]
    pub include_all: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
}

/// Get the proxy name from a proxy value
pub fn get_proxy_name(proxy: &Value) -> Option<String> {
    proxy.get("name")?.as_str().map(|s| s.to_string())
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

    // Get all proxy names
    let proxy_names: Vec<String> = input.proxies.iter().filter_map(get_proxy_name).collect();

    // Build proxy groups
    let mut proxy_groups: Vec<ProxyGroup> = Vec::new();

    // 1. 默认流量 (select group)
    // First option is "直接连接", then all load-balance groups, then all individual proxies
    let mut default_traffic_proxies: Vec<String> = vec!["直接连接".to_string()];

    // Add all load-balance groups
    default_traffic_proxies.push("全部节点负载组".to_string());
    default_traffic_proxies.extend(vec![
        "香港负载组".to_string(),
        "台湾负载组".to_string(),
        "日本负载组".to_string(),
        "新加坡负载组".to_string(),
        "美国负载组".to_string(),
        "韩国负载组".to_string(),
        "英国负载组".to_string(),
        "德国负载组".to_string(),
        "法国负载组".to_string(),
        "加拿大负载组".to_string(),
        "澳大利亚负载组".to_string(),
        "马来西亚负载组".to_string(),
        "土耳其负载组".to_string(),
        "阿根廷负载组".to_string(),
        "其他负载组".to_string(),
    ]);

    // Add all individual proxies
    default_traffic_proxies.extend(proxy_names);

    proxy_groups.push(ProxyGroup {
        name: "默认流量".to_string(),
        group_type: "select".to_string(),
        proxies: Some(default_traffic_proxies),
        include_all: None,
        filter: None,
        url: None,
        interval: None,
        strategy: None,
    });

    // 2. 全部节点负载组 (load-balance group with all proxies)
    proxy_groups.push(ProxyGroup {
        name: "全部节点负载组".to_string(),
        group_type: "load-balance".to_string(),
        proxies: None,
        include_all: Some(true),
        filter: None,
        url: Some("http://www.gstatic.com/generate_204".to_string()),
        interval: Some(180),
        strategy: Some("consistent-hashing".to_string()),
    });

    // 3. Fixed region load-balance groups with regex filters
    let regions = vec![
        ("香港负载组", "(?i)港|hk|hongkong|hong kong"),
        ("台湾负载组", "(?i)台|tw|taiwan"),
        ("日本负载组", "(?i)日本?|jp|japan"),
        ("新加坡负载组", "(?i)新|sg|singapore"),
        ("美国负载组", "(?i)美|us|usa|united states|america"),
        ("韩国负载组", "(?i)韩|kr|korea"),
        ("英国负载组", "(?i)英|uk|britain|united kingdom"),
        ("德国负载组", "(?i)德|de|germany"),
        ("法国负载组", "(?i)法|fr|france"),
        ("加拿大负载组", "(?i)加|ca|canada"),
        ("澳大利亚负载组", "(?i)澳|au|australia"),
        ("马来西亚负载组", "(?i)马来|my|malaysia"),
        ("土耳其负载组", "(?i)土耳其|tr|turkey"),
        ("阿根廷负载组", "(?i)阿根廷|ar|argentina"),
        ("其他负载组", ".*"),
    ];

    for (name, filter) in regions {
        proxy_groups.push(ProxyGroup {
            name: name.to_string(),
            group_type: "load-balance".to_string(),
            proxies: None,
            include_all: Some(true),
            filter: Some(filter.to_string()),
            url: Some("http://www.gstatic.com/generate_204".to_string()),
            interval: Some(180),
            strategy: Some("consistent-hashing".to_string()),
        });
    }

    // 4. 直接连接 (select group with only DIRECT)
    proxy_groups.push(ProxyGroup {
        name: "直接连接".to_string(),
        group_type: "select".to_string(),
        proxies: Some(vec!["DIRECT".to_string()]),
        include_all: None,
        filter: None,
        url: None,
        interval: None,
        strategy: None,
    });

    // Build rules - simple: China direct, others proxy
    let rules = vec!["GEOIP,CN,DIRECT".to_string(), "MATCH,默认流量".to_string()];

    // Build output config with our own settings
    let output = OutputConfig {
        port: 7890,
        socks_port: 7891,
        allow_lan: true,
        mode: "rule".to_string(),
        log_level: "info".to_string(),
        external_controller: "127.0.0.1:9090".to_string(),
        proxies: input.proxies,
        proxy_groups,
        rules,
    };

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&output)
        .map_err(|e| ConvertError(format!("Failed to serialize YAML: {}", e)))?;

    Ok(yaml)
}

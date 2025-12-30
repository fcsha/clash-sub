use regex::Regex;
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

/// Check if a region has any matching proxies
fn has_matching_proxies(proxy_names: &[String], pattern: &str) -> bool {
    if pattern == ".*" {
        return true; // "其他" always included
    }

    if let Ok(re) = Regex::new(pattern) {
        proxy_names.iter().any(|name| re.is_match(name))
    } else {
        false
    }
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

    // Define all possible regions
    let all_regions = [
        ("香港负载组", "(?i)港|hk|hongkong|hong kong"),
        ("台湾负载组", "(?i)台|tw|taiwan"),
        ("日本负载组", "(?i)日|jp|japan"),
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

    // Filter regions that have matching proxies
    let active_regions: Vec<(&str, &str)> = all_regions
        .iter()
        .filter(|(_, pattern)| has_matching_proxies(&proxy_names, pattern))
        .map(|(name, pattern)| (*name, *pattern))
        .collect();

    // Build proxy groups
    let mut proxy_groups: Vec<ProxyGroup> = Vec::new();

    // 1. 默认流量 (select group)
    let mut default_traffic_proxies: Vec<String> =
        vec!["节点选择".to_string(), "直接连接".to_string()];

    // Add all active load-balance groups
    default_traffic_proxies.push("全部节点负载组".to_string());
    default_traffic_proxies.extend(active_regions.iter().map(|(name, _)| name.to_string()));

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

    // 2. 节点选择 (select group with all individual proxies)
    proxy_groups.push(ProxyGroup {
        name: "节点选择".to_string(),
        group_type: "select".to_string(),
        proxies: Some(proxy_names),
        include_all: None,
        filter: None,
        url: None,
        interval: None,
        strategy: None,
    });

    // 3. 全部节点负载组 (load-balance group with all proxies)
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

    // 4. Active region load-balance groups with regex filters
    for (name, filter) in active_regions {
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

    // 5. 直接连接 (select group with only DIRECT)
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

    // Build rules - China direct, others proxy
    let rules = vec![
        "GEOIP,LAN,直接连接".to_string(),
        "GEOIP,CN,直接连接".to_string(),
        "MATCH,默认流量".to_string(),
    ];

    // Build output config
    let output = OutputConfig {
        proxies: input.proxies,
        proxy_groups,
        rules,
    };

    // Serialize to YAML
    let mut yaml = serde_yaml::to_string(&output)
        .map_err(|e| ConvertError(format!("Failed to serialize YAML: {}", e)))?;

    // Add common load-balance config at the top with anchor
    let lb_common = ".lb_common: &lb_common\n  url: http://www.gstatic.com/generate_204\n  interval: 180\n  strategy: consistent-hashing\n\n";
    yaml = lb_common.to_string() + &yaml;

    // Replace url/interval/strategy in all load-balance groups with merge reference
    let lines: Vec<&str> = yaml.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check if this is a load-balance group with url config
        if line.contains("type: load-balance") {
            result_lines.push(line.to_string());
            i += 1;

            // Process following lines
            while i < lines.len() {
                let current = lines[i];

                if current.contains("url: http://www.gstatic.com/generate_204") {
                    // Found url line, get indent
                    let indent_len = current.len() - current.trim_start().len();
                    let indent = " ".repeat(indent_len);

                    // Add merge reference instead
                    result_lines.push(indent + "<<: *lb_common");

                    // Skip the next 2 lines (interval and strategy)
                    i += 3;
                    break;
                } else if current.contains("name:") || current.trim().starts_with('-') {
                    // Reached next group, stop
                    break;
                } else {
                    result_lines.push(current.to_string());
                    i += 1;
                }
            }
        } else {
            result_lines.push(line.to_string());
            i += 1;
        }
    }

    yaml = result_lines.join("\n");
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }

    Ok(yaml)
}

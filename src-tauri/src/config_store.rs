use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Subscription {
    pub name: String,
    pub url: String,
    pub last_updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SelectorGroup {
    pub tag: String,
    #[serde(rename = "type", default = "default_selector_type")]
    pub group_type: String,
    #[serde(default)]
    pub outbounds: Vec<String>,
    #[serde(default = "default_direct")]
    pub default: String,
    pub url: Option<String>,
    pub interval: Option<u64>,
}

fn default_selector_type() -> String {
    "selector".to_string()
}

fn default_direct() -> String {
    "direct".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Rule {
    #[serde(default)]
    pub id: String,
    pub rule_type: String,
    #[serde(default)]
    pub value: Vec<String>,
    pub action: String,
    #[serde(default)]
    pub hits: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DnsConfig {
    #[serde(default)]
    pub servers: Vec<Value>,
    #[serde(default)]
    pub rules: Vec<Value>,
    #[serde(default = "default_dns_final")]
    pub final_: String,
}

fn default_dns_final() -> String {
    "local".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MitmConfig {
    #[serde(default)]
    pub enabled: bool,
    pub ca_cert: Option<String>,
    pub ca_key: Option<String>,
    #[serde(default)]
    pub ca_path: Option<String>,
    #[serde(default)]
    pub hostname_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunConfig {
    #[serde(default)]
    pub enabled: bool,
    pub interface_name: Option<String>,
    pub mtu: Option<u32>,
    #[serde(default = "default_tun_stack")]
    pub stack: String,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interface_name: None,
            mtu: None,
            stack: default_tun_stack(),
        }
    }
}

fn default_tun_stack() -> String {
    "system".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub subscriptions: Vec<Subscription>,
    #[serde(default)]
    pub outbounds: Vec<Value>,
    #[serde(default)]
    pub selectors: Vec<SelectorGroup>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub dns: DnsConfig,
    #[serde(default)]
    pub mitm: MitmConfig,
    #[serde(default)]
    pub tun: TunConfig,
    #[serde(default)]
    pub inbounds: Vec<Value>,
    #[serde(default = "default_mode")]
    pub mode: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            subscriptions: Vec::new(),
            outbounds: Vec::new(),
            selectors: Vec::new(),
            rules: Vec::new(),
            dns: DnsConfig::default(),
            mitm: MitmConfig::default(),
            tun: TunConfig::default(),
            inbounds: Vec::new(),
            mode: default_mode(),
        }
    }
}

fn default_mode() -> String {
    "rule".to_string()
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    app_data_dir: PathBuf,
}

impl ConfigStore {
    pub fn new(app: &AppHandle) -> Result<Self, std::io::Error> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        let profiles_dir = app_data_dir.join("profiles");
        fs::create_dir_all(&profiles_dir)?;

        // Migration: if old config.json exists and no profiles exist yet
        let legacy_path = app_data_dir.join("config.json");
        if legacy_path.exists() {
            let has_profiles = fs::read_dir(&profiles_dir)?
                .filter_map(|e| e.ok())
                .any(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    name.ends_with(".json")
                });

            if !has_profiles {
                if let Ok(profile) = migrate_from_legacy_config(&legacy_path) {
                    let store = Self { app_data_dir: app_data_dir.clone() };
                    let _ = store.save_profile(&profile);
                    let _ = store.set_active_profile(&profile.name);
                    // Rename old config to backup
                    let _ = fs::rename(&legacy_path, app_data_dir.join("config.json.bak"));
                    return Ok(store);
                }
            }
        }

        let store = Self { app_data_dir: app_data_dir.clone() };

        // Ensure at least one profile exists and is active
        let has_profiles = fs::read_dir(&profiles_dir)?
            .filter_map(|e| e.ok())
            .any(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.ends_with(".json")
            });

        if !has_profiles {
            let default = Profile::default();
            let _ = store.save_profile(&default);
            let _ = store.set_active_profile(&default.name);
        } else if store.get_active_profile().is_none() {
            // Profiles exist but none is active: activate the first one
            let profiles = store.list_profiles();
            if let Some(first) = profiles.first() {
                let _ = store.set_active_profile(first);
            }
        }

        Ok(store)
    }

    fn profiles_dir(&self) -> PathBuf {
        self.app_data_dir.join("profiles")
    }

    fn profile_path(&self, name: &str) -> PathBuf {
        self.profiles_dir().join(format!("{}.json", name))
    }

    fn active_file_path(&self) -> PathBuf {
        self.profiles_dir().join("active.txt")
    }

    pub fn load_profile(&self, name: &str) -> Option<Profile> {
        let path = self.profile_path(name);
        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save_profile(&self, profile: &Profile) -> Result<(), String> {
        let path = self.profile_path(&profile.name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content =
            serde_json::to_string_pretty(profile).map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(&path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_profiles(&self) -> Vec<String> {
        let dir = self.profiles_dir();
        let mut names = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".json") {
                    names.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
        names.sort();
        names
    }

    pub fn get_active_profile(&self) -> Option<Profile> {
        let path = self.active_file_path();
        let name = fs::read_to_string(&path).ok()?;
        let name = name.trim();
        if name.is_empty() {
            return None;
        }
        self.load_profile(name)
    }

    pub fn set_active_profile(&self, name: &str) -> Result<(), String> {
        if self.load_profile(name).is_none() {
            return Err(format!("Profile {} 不存在", name));
        }
        let path = self.active_file_path();
        fs::write(&path, name).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_runtime_config(&self, profile: &Profile) -> String {
        build_runtime_config(profile)
    }

    pub fn get_runtime_config_path(&self) -> PathBuf {
        self.app_data_dir.join("config.json")
    }

    pub fn write_runtime_config(&self, content: &str) -> Result<(), String> {
        let path = self.get_runtime_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file = fs::File::create(&path).map_err(|e| e.to_string())?;
        file
            .write_all(content.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn migrate_from_legacy_config(legacy_path: &PathBuf) -> Result<Profile, String> {
    let content = fs::read_to_string(legacy_path)
        .map_err(|e| format!("读取旧配置失败: {}", e))?;
    let config: Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析旧配置失败: {}", e))?;

    let name = config
        .get("_helio")
        .and_then(|h| h.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Default")
        .to_string();

    // Extract outbounds and selectors
    let mut outbounds = Vec::new();
    let mut selectors = Vec::new();
    if let Some(obs) = config.get("outbounds").and_then(Value::as_array) {
        for ob in obs {
            let ob_type = ob.get("type").and_then(Value::as_str).unwrap_or("");
            let tag = ob.get("tag").and_then(Value::as_str).unwrap_or("");
            if matches!(ob_type, "selector" | "urltest" | "fallback") {
                let members: Vec<String> = ob
                    .get("outbounds")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                let default = ob
                    .get("default")
                    .and_then(Value::as_str)
                    .unwrap_or("direct")
                    .to_string();
                selectors.push(SelectorGroup {
                    tag: tag.to_string(),
                    group_type: ob_type.to_string(),
                    outbounds: members,
                    default,
                    url: ob.get("url").and_then(Value::as_str).map(String::from),
                    interval: ob
                        .get("interval")
                        .and_then(Value::as_str)
                        .and_then(|s| s.trim_end_matches('s').parse().ok()),
                });
            } else if !matches!(tag, "direct" | "block" | "dns-out") {
                outbounds.push(ob.clone());
            }
        }
    }

    // Extract rules
    let mut rules = Vec::new();
    if let Some(rls) = config
        .get("route")
        .and_then(|r| r.get("rules"))
        .and_then(Value::as_array)
    {
        for r in rls {
            // Skip sniff rules
            if r.get("inbound").is_some()
                && r.get("action").and_then(Value::as_str) == Some("sniff")
            {
                continue;
            }

            let (rule_type, value, action) = if let Some(d) = r.get("domain").and_then(Value::as_array) {
                (
                    "DOMAIN",
                    d.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(d) = r.get("domain_suffix").and_then(Value::as_array) {
                (
                    "DOMAIN-SUFFIX",
                    d.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(k) = r.get("domain_keyword").and_then(Value::as_array) {
                (
                    "DOMAIN-KEYWORD",
                    k.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(g) = r.get("geosite").and_then(Value::as_str) {
                (
                    "GEOSITE",
                    vec![g.to_string()],
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(g) = r.get("geoip").and_then(Value::as_str) {
                (
                    "GEOIP",
                    vec![g.to_string()],
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(i) = r.get("ip_cidr").and_then(Value::as_array) {
                (
                    "IP-CIDR",
                    i.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(p) = r.get("protocol").and_then(Value::as_str) {
                (
                    "PROTOCOL",
                    vec![p.to_string()],
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else {
                continue;
            };

            rules.push(Rule {
                id: String::new(),
                rule_type: rule_type.to_string(),
                value,
                action,
                hits: "0".to_string(),
            });
        }
    }

    // Extract inbounds
    let inbounds = config
        .get("inbounds")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    // Extract DNS
    let dns = if let Some(dns_val) = config.get("dns") {
        DnsConfig {
            servers: dns_val
                .get("servers")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            rules: dns_val
                .get("rules")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            final_: dns_val
                .get("final")
                .and_then(Value::as_str)
                .unwrap_or("local")
                .to_string(),
        }
    } else {
        DnsConfig::default()
    };

    // Extract TUN from inbounds
    let mut tun = TunConfig::default();
    if let Some(tun_inbound) = inbounds
        .iter()
        .find(|i| i.get("type").and_then(Value::as_str) == Some("tun"))
    {
        tun.enabled = true;
        tun.interface_name = tun_inbound
            .get("interface_name")
            .and_then(Value::as_str)
            .map(String::from);
        tun.mtu = tun_inbound
            .get("mtu")
            .and_then(Value::as_u64)
            .map(|n| n as u32);
        tun.stack = tun_inbound
            .get("stack")
            .and_then(Value::as_str)
            .unwrap_or("system")
            .to_string();
    }

    // Determine mode from route
    let mode = if let Some(route) = config.get("route") {
        let final_outbound = route
            .get("final")
            .and_then(Value::as_str)
            .unwrap_or("Proxy");
        let auto_detect = route
            .get("auto_detect_interface")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if final_outbound == "direct" {
            "direct".to_string()
        } else if !auto_detect {
            "global".to_string()
        } else {
            "rule".to_string()
        }
    } else {
        default_mode()
    };

    let profile = Profile {
        name,
        subscriptions: Vec::new(),
        outbounds,
        selectors,
        rules,
        dns,
        mitm: MitmConfig::default(),
        tun,
        inbounds: inbounds
            .into_iter()
            .filter(|i| i.get("type").and_then(Value::as_str) != Some("tun"))
            .collect(),
        mode,
    };

    Ok(profile)
}

pub fn build_runtime_config(profile: &Profile) -> String {
    let mut config = json!({
        "log": {"level": "info", "timestamp": true},
    });

    // Inbounds
    let mut inbounds: Vec<Value> = if profile.inbounds.is_empty() {
        vec![json!({
            "type": "mixed",
            "tag": "mixed-in",
            "listen": "127.0.0.1",
            "listen_port": 6152
        })]
    } else {
        profile.inbounds.clone()
    };

    // Outbounds: nodes + selectors + direct/block
    let mut outbounds: Vec<Value> = profile.outbounds.clone();

    // Convert selectors to sing-box outbound format
    for selector in &profile.selectors {
        let mut sel = json!({
            "type": selector.group_type.clone(),
            "tag": selector.tag.clone(),
            "outbounds": selector.outbounds.clone(),
            "default": selector.default.clone(),
        });
        if let Some(url) = &selector.url {
            sel["url"] = json!(url);
        }
        if let Some(interval) = selector.interval {
            sel["interval"] = json!(format!("{}s", interval));
        }
        outbounds.push(sel);
    }

    // Ensure direct and block exist
    if !outbounds
        .iter()
        .any(|o| o.get("tag").and_then(Value::as_str) == Some("direct"))
    {
        outbounds.push(json!({"type": "direct", "tag": "direct"}));
    }
    if !outbounds
        .iter()
        .any(|o| o.get("tag").and_then(Value::as_str) == Some("block"))
    {
        outbounds.push(json!({"type": "block", "tag": "block"}));
    }

    config["outbounds"] = json!(outbounds);

    // Route
    let mut route_rules: Vec<Value> = Vec::new();

    // Add sniff rule for mixed-in
    if inbounds
        .iter()
        .any(|i| i.get("tag").and_then(Value::as_str) == Some("mixed-in"))
    {
        route_rules.push(json!({"inbound": "mixed-in", "action": "sniff"}));
    }

    // Convert profile rules to sing-box format
    for rule in &profile.rules {
        let mut rule_json = json!({});
        match rule.rule_type.as_str() {
            "DOMAIN" => rule_json["domain"] = json!(rule.value),
            "DOMAIN-SUFFIX" => rule_json["domain_suffix"] = json!(rule.value),
            "DOMAIN-KEYWORD" => rule_json["domain_keyword"] = json!(rule.value),
            "GEOSITE" => {
                if let Some(v) = rule.value.first() {
                    rule_json["geosite"] = json!(v);
                }
            }
            "GEOIP" => {
                if let Some(v) = rule.value.first() {
                    rule_json["geoip"] = json!(v);
                }
            }
            "IP-CIDR" => rule_json["ip_cidr"] = json!(rule.value),
            "IP-CIDR6" => rule_json["ip_cidr"] = json!(rule.value),
            "PROTOCOL" => {
                if let Some(v) = rule.value.first() {
                    rule_json["protocol"] = json!(v);
                }
            }
            _ => {}
        }
        rule_json["outbound"] = json!(rule.action);
        route_rules.push(rule_json);
    }

    // Default rules if empty
    if route_rules.len() <= 1 {
        route_rules.push(json!({
            "outbound": "direct",
            "domain_suffix": ["apple.com", "icloud.com", "push.apple.com", "gateway.push.apple.com"]
        }));
    }

    let (final_outbound, auto_detect) = match profile.mode.as_str() {
        "direct" => ("direct", true),
        "global" => ("Proxy", false),
        _ => ("Proxy", true),
    };

    config["route"] = json!({
        "auto_detect_interface": auto_detect,
        "final": final_outbound,
        "rules": route_rules
    });

    // DNS
    if !profile.dns.servers.is_empty() {
        config["dns"] = json!({
            "servers": profile.dns.servers,
            "rules": profile.dns.rules,
            "final": profile.dns.final_,
        });
    }

    // TUN
    if profile.tun.enabled {
        let mut tun_inbound = json!({
            "type": "tun",
            "tag": "tun-in",
            "interface_name": profile.tun.interface_name.as_deref().unwrap_or("utun"),
            "mtu": profile.tun.mtu.unwrap_or(9000),
            "stack": profile.tun.stack,
            "auto_route": true,
            "strict_route": true,
            "address": ["172.18.0.1/30", "fdfe:dcba:9876::1/126"],
        });

        // 排除本机代理端口和局域网，防止断网
        tun_inbound["inet4_route_exclude_address"] = json!([
            "127.0.0.1/8",
            "192.168.0.0/16",
            "10.0.0.0/8",
            "172.16.0.0/12"
        ]);
        tun_inbound["inet6_route_exclude_address"] = json!([
            "::1/128",
            "fe80::/10"
        ]);

        inbounds.push(tun_inbound);
    }

    config["inbounds"] = json!(inbounds);

    // Always enable experimental Clash API for activity page / proxy data
    config["experimental"] = json!({
        "clash_api": {
            "external_controller": "127.0.0.1:9090",
            "secret": ""
        }
    });

    // MITM: if enabled and CA cert exists, add TLS inbound for HTTPS decryption
    if profile.mitm.enabled && profile.mitm.ca_cert.is_some() {
        let mut tls_inbound = json!({
            "type": "mixed",
            "tag": "mitm-in",
            "listen": "127.0.0.1",
            "listen_port": 6153,
            "tls": {
                "enabled": true
            }
        });
        if let Some(cert) = &profile.mitm.ca_cert {
            tls_inbound["tls"]["certificate"] = json!(cert);
        }
        if let Some(key) = &profile.mitm.ca_key {
            tls_inbound["tls"]["key"] = json!(key);
        }
        inbounds.push(tls_inbound);
    }

    serde_json::to_string_pretty(&config).unwrap_or_else(|_| "{}".to_string())
}

pub fn refresh_proxy_selector_in_profile(profile: &mut Profile) -> Result<(), String> {
    let mut members: Vec<String> = profile
        .outbounds
        .iter()
        .filter(|item| {
            let ob_type = item.get("type").and_then(Value::as_str).unwrap_or("");
            let tag = item.get("tag").and_then(Value::as_str).unwrap_or("");
            !matches!(ob_type, "selector" | "direct" | "block" | "dns")
                && !is_reserved_node_tag(tag)
        })
        .filter_map(|item| item.get("tag").and_then(Value::as_str).map(String::from))
        .collect();

    // Add other selector tags as members (except Proxy itself)
    for selector in &profile.selectors {
        if selector.tag != "Proxy" && !members.contains(&selector.tag) {
            members.push(selector.tag.clone());
        }
    }

    if !members.iter().any(|tag| tag == "direct") {
        members.push("direct".to_string());
    }
    members.sort();
    members.dedup();
    if members.is_empty() {
        members.push("direct".to_string());
    }

    if let Some(proxy) = profile.selectors.iter_mut().find(|s| s.tag == "Proxy") {
        proxy.outbounds = members.clone();
        if !proxy.outbounds.contains(&proxy.default) {
            proxy.default = members
                .first()
                .cloned()
                .unwrap_or_else(|| "direct".to_string());
        }
    } else {
        profile.selectors.push(SelectorGroup {
            tag: "Proxy".to_string(),
            group_type: "selector".to_string(),
            outbounds: members,
            default: "direct".to_string(),
            url: None,
            interval: None,
        });
    }
    Ok(())
}

pub fn is_reserved_node_tag(tag: &str) -> bool {
    matches!(tag, "direct" | "block" | "dns-out" | "Proxy")
}

pub fn update_profile_from_runtime(profile: &mut Profile, config: &Value) {
    // Update inbounds
    if let Some(inbounds) = config.get("inbounds").and_then(Value::as_array) {
        profile.inbounds = inbounds.clone();
    }

    // Update outbounds and selectors
    let mut outbounds = Vec::new();
    let mut selectors = Vec::new();
    if let Some(obs) = config.get("outbounds").and_then(Value::as_array) {
        for ob in obs {
            let ob_type = ob.get("type").and_then(Value::as_str).unwrap_or("");
            let tag = ob.get("tag").and_then(Value::as_str).unwrap_or("");
            if matches!(ob_type, "selector" | "urltest" | "fallback") {
                let members = ob
                    .get("outbounds")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                let default = ob
                    .get("default")
                    .and_then(Value::as_str)
                    .unwrap_or("direct")
                    .to_string();
                selectors.push(SelectorGroup {
                    tag: tag.to_string(),
                    group_type: ob_type.to_string(),
                    outbounds: members,
                    default,
                    url: ob.get("url").and_then(Value::as_str).map(String::from),
                    interval: ob
                        .get("interval")
                        .and_then(Value::as_str)
                        .and_then(|s| s.trim_end_matches('s').parse().ok()),
                });
            } else if !matches!(tag, "direct" | "block" | "dns-out") {
                outbounds.push(ob.clone());
            }
        }
    }
    profile.outbounds = outbounds;
    profile.selectors = selectors;

    // Update rules
    let mut rules = Vec::new();
    if let Some(rls) = config
        .get("route")
        .and_then(|r| r.get("rules"))
        .and_then(Value::as_array)
    {
        for r in rls {
            if r.get("inbound").is_some()
                && r.get("action").and_then(Value::as_str) == Some("sniff")
            {
                continue;
            }
            let (rule_type, value, action) = if let Some(d) =
                r.get("domain").and_then(Value::as_array)
            {
                (
                    "DOMAIN",
                    d.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(d) = r.get("domain_suffix").and_then(Value::as_array) {
                (
                    "DOMAIN-SUFFIX",
                    d.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(k) = r.get("domain_keyword").and_then(Value::as_array) {
                (
                    "DOMAIN-KEYWORD",
                    k.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(g) = r.get("geosite").and_then(Value::as_str) {
                (
                    "GEOSITE",
                    vec![g.to_string()],
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(g) = r.get("geoip").and_then(Value::as_str) {
                (
                    "GEOIP",
                    vec![g.to_string()],
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(i) = r.get("ip_cidr").and_then(Value::as_array) {
                (
                    "IP-CIDR",
                    i.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else if let Some(p) = r.get("protocol").and_then(Value::as_str) {
                (
                    "PROTOCOL",
                    vec![p.to_string()],
                    r.get("outbound")
                        .and_then(Value::as_str)
                        .unwrap_or("direct")
                        .to_string(),
                )
            } else {
                continue;
            };
            rules.push(Rule {
                id: String::new(),
                rule_type: rule_type.to_string(),
                value,
                action,
                hits: "0".to_string(),
            });
        }
    }
    profile.rules = rules;

    // Update mode from route
    if let Some(route) = config.get("route") {
        let final_outbound = route
            .get("final")
            .and_then(Value::as_str)
            .unwrap_or("Proxy");
        let auto_detect = route
            .get("auto_detect_interface")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        profile.mode = if final_outbound == "direct" {
            "direct".to_string()
        } else if !auto_detect {
            "global".to_string()
        } else {
            "rule".to_string()
        };
    }

    // Update DNS
    if let Some(dns_val) = config.get("dns") {
        profile.dns = DnsConfig {
            servers: dns_val
                .get("servers")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            rules: dns_val
                .get("rules")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            final_: dns_val
                .get("final")
                .and_then(Value::as_str)
                .unwrap_or("local")
                .to_string(),
        };
    }
}

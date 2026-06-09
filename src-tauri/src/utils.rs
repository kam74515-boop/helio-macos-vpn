pub fn run_cmd(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(args[0])
        .args(&args[1..])
        .output()
        .map_err(|e| format!("{}: {}", args.join(" "), e))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn run_cmd_stderr(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(args[0])
        .args(&args[1..])
        .output()
        .map_err(|e| format!("{}: {}", args.join(" "), e))?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(combined)
}

pub fn guess_icon(name: &str) -> &str {
    let lower = name.to_lowercase();
    if lower.contains("chrome") || lower.contains("google chrome") { "language" }
    else if lower.contains("safari") { "explore" }
    else if lower.contains("firefox") { "language" }
    else if lower.contains("terminal") || lower.contains("iterm") { "terminal" }
    else if lower.contains("xray") || lower.contains("v2ray") { "alt_route" }
    else if lower.contains("clash") || lower.contains("mihomo") { "alt_route" }
    else if lower.contains("sing-box") { "alt_route" }
    else if lower.contains("cursor") || lower.contains("code") { "deployed_code" }
    else if lower.contains("trae") { "memory" }
    else if lower.contains("wechat") || lower.contains("微信") { "chat" }
    else if lower.contains("feishu") || lower.contains("lark") || lower.contains("飞书") { "send" }
    else if lower.contains("dingtalk") || lower.contains("钉钉") { "send" }
    else if lower.contains("mail") || lower.contains("邮件") { "mail" }
    else if lower.contains("music") || lower.contains("音乐") || lower.contains("spotify") { "music_note" }
    else if lower.contains("slack") { "chat" }
    else if lower.contains("discord") { "chat" }
    else if lower.contains("zoom") { "videocam" }
    else if lower.contains("telegram") { "send" }
    else if lower.contains("quark") || lower.contains("夸克") { "cloud" }
    else if lower.contains("notion") { "description" }
    else if lower.contains("figma") { "palette" }
    else if lower.contains("docker") { "deployed_code" }
    else if lower.contains("node") || lower.contains("npm") { "deployed_code" }
    else if lower.contains("python") { "deployed_code" }
    else if lower.contains("antigravity") { "explore" }
    else if lower.contains("codex") { "deployed_code" }
    else if lower.starts_with("com.apple") { "build" }
    else if lower.contains("kernel") || lower.contains("system") || lower.contains("sys") { "build" }
    else if lower.contains("launchd") || lower.contains("core") { "build" }
    else { "memory" }
}

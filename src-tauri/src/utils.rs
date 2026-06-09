use std::collections::HashMap;
use std::sync::Mutex;
use base64::prelude::*;

static ICON_CACHE: std::sync::OnceLock<Mutex<HashMap<String, Option<String>>>> = std::sync::OnceLock::new();

fn icon_cache() -> &'static Mutex<HashMap<String, Option<String>>> {
    ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Extract the real macOS app icon for a process and return it as a base64 PNG data URI.
/// Uses lsof to find the executable path, then reads the .app bundle's .icns icon.
pub fn get_app_icon_base64(pid: u32, name: &str) -> Option<String> {
    let cache_key = name.to_string();
    {
        let cache = icon_cache().lock().unwrap();
        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }
    }

    let result = get_app_icon_base64_impl(pid, name);

    {
        let mut cache = icon_cache().lock().unwrap();
        cache.insert(cache_key, result.clone());
    }

    result
}

fn get_app_icon_base64_impl(pid: u32, _name: &str) -> Option<String> {
    // 1. Get the executable path via lsof
    let lsof_out = run_cmd(&["lsof", "-p", &pid.to_string()]).ok()?;

    let mut exe_path: Option<String> = None;
    for line in lsof_out.lines() {
        // lsof txt line: COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
        if line.contains(" txt ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                exe_path = Some(parts[8].to_string());
                break;
            }
        }
    }

    let exe_path = exe_path?;

    // 2. Derive .app bundle path from executable path
    // e.g. /Applications/Google Chrome.app/Contents/MacOS/Google Chrome
    let app_path = if exe_path.contains(".app/Contents/MacOS/") {
        exe_path.split(".app/Contents/MacOS/").next()
            .map(|p| format!("{}.app", p))
    } else {
        None
    }?;

    if !std::path::Path::new(&app_path).exists() {
        return None;
    }

    // 3. Read CFBundleIconFile from Info.plist
    let info_plist = format!("{}/Contents/Info.plist", app_path);
    if !std::path::Path::new(&info_plist).exists() {
        return None;
    }

    let icon_name = run_cmd(&["defaults", "read", &info_plist, "CFBundleIconFile"]).ok()?;
    let icon_name = icon_name.trim();
    if icon_name.is_empty() {
        return None;
    }

    // 4. Build full icon path (.icns in Resources)
    let icon_path = if icon_name.ends_with(".icns") {
        format!("{}/Contents/Resources/{}", app_path, icon_name)
    } else {
        format!("{}/Contents/Resources/{}.icns", app_path, icon_name)
    };

    if !std::path::Path::new(&icon_path).exists() {
        return None;
    }

    // 5. Convert to PNG (32px thumbnail) via sips
    let tmp_path = format!("/tmp/helio_icon_{}.png", pid);
    let _ = run_cmd(&[
        "sips", "-Z", "32", "-s", "format", "png",
        &icon_path, "--out", &tmp_path,
    ]);

    // 6. Read PNG and base64-encode
    let png_data = std::fs::read(&tmp_path).ok()?;
    let _ = std::fs::remove_file(&tmp_path);

    if png_data.is_empty() {
        return None;
    }

    Some(format!("data:image/png;base64,{}", BASE64_STANDARD.encode(&png_data)))
}

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
    if lower.contains("chrome") || lower.contains("google chrome") || lower.contains("chromium") { "language" }
    else if lower.contains("safari") || lower.contains("webkit") { "explore" }
    else if lower.contains("firefox") || lower.contains("gecko") { "language" }
    else if lower.contains("edge") { "language" }
    else if lower.contains("terminal") || lower.contains("iterm") || lower.contains("warp") || lower.contains("alacritty") { "terminal" }
    else if lower.contains("xray") || lower.contains("v2ray") || lower.contains("v2rayn") || lower.contains("v2rayx") { "alt_route" }
    else if lower.contains("clash") || lower.contains("mihomo") || lower.contains("clashx") || lower.contains("stash") { "alt_route" }
    else if lower.contains("sing-box") || lower.contains("singbox") { "alt_route" }
    else if lower.contains("cursor") { "deployed_code" }
    else if lower.contains("vscode") || lower.contains("code -") || lower.contains("code.") || lower.contains("visual studio code") { "deployed_code" }
    else if lower.contains("trae") { "memory" }
    else if lower.contains("wechat") || lower.contains("微信") || lower.contains("weixin") { "chat" }
    else if lower.contains("feishu") || lower.contains("lark") || lower.contains("飞书") { "send" }
    else if lower.contains("dingtalk") || lower.contains("钉钉") { "send" }
    else if lower.contains("mail") || lower.contains("邮件") || lower.contains("outlook") || lower.contains("thunderbird") { "mail" }
    else if lower.contains("music") || lower.contains("音乐") || lower.contains("spotify") || lower.contains("netease") || lower.contains("qqmusic") { "music_note" }
    else if lower.contains("slack") { "chat" }
    else if lower.contains("discord") { "chat" }
    else if lower.contains("zoom") || lower.contains("teams") || lower.contains("腾讯会议") || lower.contains("lark") { "videocam" }
    else if lower.contains("telegram") || lower.contains("tg") { "send" }
    else if lower.contains("quark") || lower.contains("夸克") { "cloud" }
    else if lower.contains("notion") { "description" }
    else if lower.contains("figma") || lower.contains("sketch") || lower.contains("adobe") { "palette" }
    else if lower.contains("docker") || lower.contains("container") { "deployed_code" }
    else if lower.contains("node") || lower.contains("npm") || lower.contains("pnpm") || lower.contains("yarn") { "deployed_code" }
    else if lower.contains("python") || lower.contains("pip") { "deployed_code" }
    else if lower.contains("antigravity") { "explore" }
    else if lower.contains("codex") { "deployed_code" }
    else if lower.contains("git") { "deployed_code" }
    else if lower.contains("go") || lower.contains("golang") { "deployed_code" }
    else if lower.contains("rust") || lower.contains("cargo") { "deployed_code" }
    else if lower.contains("java") || lower.contains("jdk") || lower.contains("jvm") { "deployed_code" }
    else if lower.contains("php") { "deployed_code" }
    else if lower.contains("ruby") || lower.contains("gem") { "deployed_code" }
    else if lower.contains("obs") || lower.contains("stream") || lower.contains("录屏") { "videocam" }
    else if lower.contains("photo") || lower.contains("image") || lower.contains("图片") { "palette" }
    else if lower.contains("video") || lower.contains("movie") || lower.contains("film") { "videocam" }
    else if lower.contains("game") || lower.contains("steam") || lower.contains("epic") { "sports_esports" }
    else if lower.contains("bank") || lower.contains("pay") || lower.contains("wallet") || lower.contains("支付宝") || lower.contains("微信") { "account_balance" }
    else if lower.contains("map") || lower.contains("导航") || lower.contains("location") { "location_on" }
    else if lower.contains("shop") || lower.contains("store") || lower.contains("taobao") || lower.contains("jd") || lower.contains("buy") { "shopping_cart" }
    else if lower.contains("book") || lower.contains("read") || lower.contains("kindle") { "menu_book" }
    else if lower.contains("news") || lower.contains("rss") || lower.contains("feed") { "newspaper" }
    else if lower.contains("calendar") || lower.contains("日程") { "calendar_today" }
    else if lower.contains("clock") || lower.contains("timer") || lower.contains("alarm") { "schedule" }
    else if lower.contains("weather") || lower.contains("天气") { "wb_sunny" }
    else if lower.contains("translate") || lower.contains("翻译") { "translate" }
    else if lower.contains("calc") || lower.contains("计算器") { "calculate" }
    else if lower.contains("finder") || lower.contains("explorer") || lower.contains("文件") { "folder" }
    else if lower.contains("settings") || lower.contains("system preferences") || lower.contains("偏好设置") { "settings" }
    else if lower.contains("help") || lower.contains("support") { "help" }
    else if lower.contains("download") || lower.contains("迅雷") || lower.contains("aria") { "download" }
    else if lower.contains("upload") || lower.contains("sync") || lower.contains("dropbox") || lower.contains("onedrive") || lower.contains("icloud") { "cloud_upload" }
    else if lower.contains("print") || lower.contains("printer") { "print" }
    else if lower.contains("scan") || lower.contains("ocr") { "document_scanner" }
    else if lower.contains("password") || lower.contains("keychain") || lower.contains("1password") || lower.contains("bitwarden") { "vpn_key" }
    else if lower.contains("vpn") || lower.contains("proxy") || lower.contains("shadowsocks") || lower.contains("trojan") { "vpn_lock" }
    else if lower.contains("ssh") || lower.contains("sftp") || lower.contains("scp") { "terminal" }
    else if lower.contains("ftp") || lower.contains("filezilla") { "folder_open" }
    else if lower.contains("torrent") || lower.contains("bt") || lower.contains("qbittorrent") || lower.contains("transmission") { "swap_vert" }
    else if lower.contains("ide") || lower.contains("jetbrains") || lower.contains("intellij") || lower.contains("pycharm") || lower.contains("goland") || lower.contains("webstorm") { "code" }
    else if lower.contains("excel") || lower.contains("spreadsheet") || lower.contains("numbers") || lower.contains("csv") { "table_chart" }
    else if lower.contains("word") || lower.contains("document") || lower.contains("pages") || lower.contains("writer") { "description" }
    else if lower.contains("ppt") || lower.contains("presentation") || lower.contains("keynote") || lower.contains("slides") { "slideshow" }
    else if lower.contains("pdf") || lower.contains("preview") || lower.contains("acrobat") { "picture_as_pdf" }
    else if lower.contains("browser") || lower.contains("brave") || lower.contains("opera") || lower.contains("arc") { "language" }
    else if lower.contains("postman") || lower.contains("insomnia") || lower.contains("api") || lower.contains("httpie") { "api" }
    else if lower.contains("db") || lower.contains("database") || lower.contains("sql") || lower.contains("mysql") || lower.contains("postgres") || lower.contains("mongo") || lower.contains("redis") { "storage" }
    else if lower.contains("server") || lower.contains("nginx") || lower.contains("apache") || lower.contains("caddy") { "dns" }
    else if lower.contains("log") || lower.contains("syslog") { "receipt_long" }
    else if lower.contains("backup") || lower.contains("time machine") { "backup" }
    else if lower.contains("update") || lower.contains("upgrade") || lower.contains("patch") { "system_update" }
    else if lower.contains("security") || lower.contains("firewall") || lower.contains("antivirus") || lower.contains("defender") { "security" }
    else if lower.contains("monitor") || lower.contains("activity") || lower.contains("top") || lower.contains("htop") { "monitoring" }
    else if lower.contains("chat") || lower.contains("message") || lower.contains("im") || lower.contains("qq") || lower.contains("whatsapp") || lower.contains("line") { "chat" }
    else if lower.contains("social") || lower.contains("twitter") || lower.contains("x.com") || lower.contains("facebook") || lower.contains("instagram") || lower.contains("tiktok") || lower.contains("reddit") || lower.contains("微博") || lower.contains("小红书") { "public" }
    else if lower.contains("com.apple") { "build" }
    else if lower.contains("kernel") || lower.contains("system") || lower.contains("sys") || lower.contains("daemon") { "build" }
    else if lower.contains("launchd") || lower.contains("core") || lower.contains("agent") { "build" }
    else if lower.contains("helper") || lower.contains("service") || lower.contains("worker") { "build" }
    else if lower.contains("plugin") || lower.contains("extension") { "extension" }
    else { "memory" }
}

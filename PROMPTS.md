# Kimi 修改提示词集合

> 以下提示词按优先级排列，每个提示词独立可用，包含完整上下文和修改要求。项目路径: `/Users/karl/apps/vpn/surge-material-prototype`

---

## P1: 修复 TUN 路由配置字段名错误

### 文件
`src-tauri/src/config_store.rs` 第 668 行附近

### 问题
`build_runtime_config` 函数中 TUN inbound 使用 `inet4_route_address` 列出私有网段（127.0.0.1/8, 192.168.0.0/16, 10.0.0.0/8, 172.16.0.0/12），但这些地址的意图是**排除**这些网段不被 TUN 接管，所以应该用 `inet4_route_exclude_address` 和 `inet6_route_exclude_address`。当前写法会导致 TUN 只路由这些私有地址，而不是排除它们。

### 当前代码 (config_store.rs:668-677)
```rust
tun_inbound["inet4_route_address"] = json!([
    "127.0.0.1/8",
    "192.168.0.0/16",
    "10.0.0.0/8",
    "172.16.0.0/12"
]);
tun_inbound["inet6_route_address"] = json!([
    "::1/128",
    "fe80::/10"
]);
```

### 修改为
```rust
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
```

---

## P2: 修复 traffic.rs 无操作语句

### 文件
`src-tauri/src/commands/traffic.rs` 第 70 行

### 问题
```rust
if elapsed < 0.1 { elapsed as f64; }
```
`elapsed as f64;` 是一个表达式语句，返回值被丢弃，没有任何效果。`elapsed` 本身是 `f64`，`elapsed as f64` 还是 `f64`，什么都没做。这段代码的意图应该是：如果两次采样间隔太短，直接返回零速度避免计算出异常值。

### 修改为
```rust
let elapsed = if elapsed < 0.1 { 1.0 } else { elapsed };
```
或者更清晰地：
```rust
if elapsed < 0.1 {
    let mut snap = state.traffic_snapshot.lock().unwrap();
    snap.prev_rx = total_rx;
    snap.prev_tx = total_tx;
    snap.prev_time = now;
    return Ok(TrafficStats {
        upload_kbps: 0.0,
        download_kbps: 0.0,
        total_upload_mb: (total_tx as f64 / 1024.0 / 1024.0 * 10.0).round() / 10.0,
        total_download_mb: (total_rx as f64 / 1024.0 / 1024.0 * 10.0).round() / 10.0,
        history: snap.history_rx.clone(),
    });
}
```

---

## P3: App.jsx 修复 traffic-update 事件空回调和引擎生命周期不对称

### 文件
`src/App.jsx`

### 问题 1: traffic-update 事件监听回调为空
第 47-49 行：
```jsx
const unlisten = listen("traffic-update", (_event) => {
  // Real-time traffic events supplement polling
}).catch(() => {});
```
回调函数体为空，收到的流量数据被完全丢弃。

### 问题 2: 关闭代理时不停止引擎
`setSystemProxy(false)` 只调用 `set_system_proxy(false)` 但不调用 `stop_engine()`。而开启代理时会调用 `start_engine`。这意味着关闭系统代理后 sing-box 进程仍在运行占用资源。

### 修改

将 `App.jsx` 的 `setSystemProxy` 和 `useEffect` 修改为：

```jsx
const setSystemProxy = async (enable) => {
  try {
    if (enable) {
      await safeInvoke("start_engine", { config: null });
      await safeInvoke("start_monitoring");
    }
    await safeInvoke("set_system_proxy", { enable });
    if (!enable) {
      await safeInvoke("stop_monitoring");
      await safeInvoke("stop_engine");
    }
    const state = await safeInvoke("get_proxy_state");
    setSystemProxyState(Boolean(state?.system_proxy_enabled));
  } catch (e) {
    console.error("Failed to set system proxy", e);
    setSystemProxyState(false);
    addToast("代理设置失败: " + e, "error");
  }
};
```

注意：需要从 `useToast()` 获取 `addToast`。由于 `App` 组件被 `ToastProvider` 包裹，`useToast` 不能直接在 `App` 中使用。解决方案是将 `setSystemProxy` 的逻辑提取到 `ToastProvider` 内部的子组件中，或者在 catch 中只做 console.error。

推荐的架构调整：创建一个 `AppContent` 组件放在 `ToastProvider` 内部：

```jsx
export function App() {
  return (
    <ToastProvider>
      <AppContent />
    </ToastProvider>
  );
}

function AppContent() {
  const { addToast } = useToast();
  // ... 原来的 App 逻辑全部移到这里
}
```

对于 traffic-update 事件，补充实际处理逻辑：

```jsx
const unlisten = await listen("traffic-update", (event) => {
  // event.payload 是 TrafficStats 对象
  // 可以用来更新全局流量状态或触发页面刷新
  console.log("Traffic update:", event.payload);
}).catch(() => {});
```

---

## P4: MitmPage 主开关持久化到 Profile

### 文件
1. `src/pages/MitmPage.jsx`
2. `src-tauri/src/commands/mitm.rs`

### 问题
`MitmPage` 的主开关 `enabled` 是 `useState(false)`，切换后不会保存到后端 Profile 的 `mitm.enabled` 字段。页面刷新或导航后状态丢失。

### 修改

**后端** — 在 `src-tauri/src/commands/mitm.rs` 中添加新命令：

```rust
#[tauri::command]
pub fn set_mitm_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let store = app.state::<ConfigStore>();
    let mut profile = store.get_active_profile()
        .ok_or("无法获取活跃 Profile")?;
    profile.mitm.enabled = enabled;
    store.save_profile(&profile)?;
    Ok(())
}
```

**后端** — 在 `src-tauri/src/lib.rs` 的 `invoke_handler` 宏中添加：
```rust
commands::mitm::set_mitm_enabled,
```

**前端** — 修改 `MitmPage.jsx`：

```jsx
// 替换 useState(false) 为从后端读取 + 写回
const [enabled, setEnabled] = useState(false);
const { data: certStatus, refresh: refreshStatus } = useTauriPoll("get_ca_status", null, 30000);
const { data: hostnames, refresh: refreshHostnames } = useTauriPoll("get_mitm_hostnames", null, 3000);

// 从 certStatus 或单独接口读取初始 enabled 状态
useEffect(() => {
  if (canUseTauri()) {
    safeInvoke("get_mitm_hostnames", null).then(() => {
      // 可以添加一个 get_mitm_enabled 命令，或从 profile 读取
    });
  }
}, []);

const handleToggle = async (val) => {
  setEnabled(val);
  if (!canUseTauri()) return;
  try {
    await safeInvoke("set_mitm_enabled", { enabled: val });
    addToast(val ? "HTTPS 解密已启用" : "HTTPS 解密已关闭", "success");
  } catch (e) {
    setEnabled(!val); // 回滚
    addToast(`切换失败: ${e}`, "error");
  }
};
```

然后在 JSX 中：
```jsx
<Toggle checked={enabled} onChange={handleToggle} />
```

---

## P5: build_runtime_config 输出 MITM 配置到 sing-box

### 文件
`src-tauri/src/config_store.rs` 的 `build_runtime_config` 函数

### 问题
Profile 中存储了 `MitmConfig`（enabled, hostname_list, ca_cert, ca_key），但 `build_runtime_config` 生成 sing-box 运行时 JSON 时完全没有输出 MITM 相关配置。这意味着即使前端开启了 MITM 并添加了主机名，sing-box 实际运行时不会进行 HTTPS 解密。

### 修改

在 `build_runtime_config` 函数中，`config["experimental"]` 部分之前（约第 681 行 `config["inbounds"] = json!(inbounds);` 之后），添加 MITM 配置输出：

```rust
// MITM: 如果启用了 HTTPS 解密，在 experimental 中配置
if profile.mitm.enabled && profile.mitm.ca_cert.is_some() {
    let experimental = config.get_mut("experimental")
        .and_then(|v| v.as_object_mut())
        .unwrap_or(&mut serde_json::Map::new());

    // sing-box 的 TLS 配置需要通过 clash_api 的 TLS 字段注入
    // 或通过自定义 inbound 的 TLS 配置
    // 具体取决于 sing-box 版本，以下是参考实现
    let mut tls_config = serde_json::Map::new();
    tls_config.insert("enabled".to_string(), json!(true));
    if let Some(cert) = &profile.mitm.ca_cert {
        tls_config.insert("certificate".to_string(), json!(cert));
    }
    if let Some(key) = &profile.mitm.ca_key {
        tls_config.insert("key".to_string(), json!(key));
    }

    // 如果有主机名列表，注入到 inbound 的 TLS 中
    if !profile.mitm.hostname_list.is_empty() {
        tls_config.insert(
            "server_name".to_string(),
            json!(profile.mitm.hostname_list),
        );
    }
}
```

**注意**：sing-box 的 MITM 实现方式与 Clash/Surge 不同。sing-box 1.11+ 支持 `inbound` 上的 `tls` 配置用于 MITM，或使用 `tls_fragment` / `reality`。请查阅当前使用的 sing-box 版本文档，确定正确的 MITM 配置字段名。如果 sing-box 版本不支持原生 MITM，则应在前端明确标注该功能不可用。

---

## P6: useTauriPoll 依赖数组缺少 args

### 文件
`src/hooks/tauri.js` 第 24 行

### 问题
```js
}, [command, interval, tick]);
```
如果调用方传入动态 `args`，effect 不会在 args 变化时重新执行，导致数据过时。

### 修改

```js
}, [command, interval, tick, JSON.stringify(args)]);
```

---

## P7: CA is_trusted 始终返回 false

### 文件
`src-tauri/src/commands/ca.rs` 第 66 行

### 问题
`CaStatus` 的 `is_trusted` 字段硬编码为 `false`，无法反映实际的系统信任状态。

### 修改

在 `get_ca_status` 中添加实际的系统信任检测：

```rust
let is_trusted = if has_cert {
    // 检查 macOS 系统钥匙串是否信任该证书
    let cert = cert_path.to_string_lossy().to_string();
    let result = std::process::Command::new("security")
        .args(["find-certificate", "-a", "-c", "Helio CA", "/Library/Keychains/System.keychain"])
        .output();
    result.map(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).is_empty()).unwrap_or(false)
} else {
    false
};
```

然后将 `is_trusted: false` 改为 `is_trusted`。

---

## P8: ProcessesPage 详情面板使用硬编码数据

### 文件
`src/pages/ProcessesPage.jsx` 第 11-13 行、第 37 行

### 问题
`ProcessDetail` 组件中：
- "命中策略" 始终显示 `"--"`
- "最近地址" 始终显示 `"--"`
- "DNS" 始终显示 `"system-resolver"`
- MiniLine 图表始终使用硬编码的 `[4, 5, 4, 6, 5, 4, 5, 4, 5, 6]`

### 修改

这些字段需要后端支持。在 `src-tauri/src/types.rs` 的 `ProcessInfo` 中添加字段：

```rust
pub struct ProcessInfo {
    // ... 现有字段 ...
    pub policy: String,        // 命中策略
    pub last_address: String,  // 最近地址
    pub dns_resolver: String,  // DNS 解析器
    pub traffic_history: Vec<f64>, // 流量历史
}
```

在 `src-tauri/src/commands/process.rs` 的 `get_processes_impl` 中填充这些字段（可以从连接列表推导，或暂时设为默认值）：

```rust
result.push(ProcessInfo {
    // ... 现有字段 ...
    policy: "direct".to_string(),
    last_address: "-".to_string(),
    dns_resolver: "system".to_string(),
    traffic_history: Vec::new(),
});
```

前端 `ProcessesPage.jsx` 修改为：

```jsx
const rows = [
  ["当前速度", process.speed || "--"],
  ["累计流量", process.total || "--"],
  ["活动连接", process.connections != null ? String(process.connections) : "--"],
  ["命中策略", process.policy || "--"],
  ["最近地址", process.lastAddress || process.last_address || "--"],
  ["DNS", process.dnsResolver || process.dns_resolver || "system"],
];

// MiniLine 使用真实历史或空数组
<MiniLine color="cyan" values={process.trafficHistory || process.traffic_history || []} />
```

---

## P9: CapturePage 多个 Tab 空壳

### 文件
`src/pages/CapturePage.jsx`

### 问题
Tab 栏有 6 个选项（最近的请求、活动连接、DNS、设备、流量统计、日志簿），但只有第一个 Tab 有内容渲染。其余 5 个 Tab 切换后无内容。

### 修改要求

为每个 Tab 实现最小可用内容：

1. **活动连接** — 复用现有的 `clashConnectionsRaw` 数据，以表格形式展示（ID, 进程, 状态, 目标, 上传, 下载）
2. **DNS** — 调用 `get_clash_connections` 并过滤 DNS 相关连接，或显示一个 "DNS 查询日志功能需要 sing-box DNS 日志支持" 的占位提示
3. **设备** — 调用 `get_lan_devices`（参考 DevicesPage），展示设备列表
4. **流量统计** — 调用 `get_traffic_stats`，展示上传/下载/总计
5. **日志簿** — 显示 sing-box 日志占位（需要后端 `log` 事件监听），或显示 "日志簿功能开发中" 提示

在 `CapturePage.jsx` 中添加条件渲染：

```jsx
{tab === "最近的请求" && (
  /* 现有的请求表格 */
)}
{tab === "活动连接" && (
  <div className="request-table">
    {connectionsData?.slice(0, 50).map((c) => (
      <div key={c.id} className="request-row">
        <span>{c.id.slice(0, 8)}</span>
        <span>{c.metadata?.host || c.metadata?.destinationIP || "-"}</span>
        <span>{(c.chains || []).join(" → ") || "direct"}</span>
        <span>{formatBytes(c.upload)}</span>
        <span>{formatBytes(c.download)}</span>
      </div>
    ))}
  </div>
)}
{tab === "DNS" && (
  <div style={{ padding: 24, textAlign: "center", color: "var(--muted)" }}>
    DNS 查询日志需要 sing-box DNS 日志模块支持，功能开发中。
  </div>
)}
{tab === "设备" && (
  <DevicesTab />
)}
{tab === "流量统计" && (
  <TrafficTab />
)}
{tab === "日志簿" && (
  <div style={{ padding: 24, textAlign: "center", color: "var(--muted)" }}>
    日志簿功能需要 sing-box 日志事件流支持，功能开发中。
  </div>
)}
```

---

## P10: import * as MuiIcons 导致打包体积膨胀

### 文件
`src/data/mock.js` 第 1 行

### 问题
```js
import * as MuiIcons from "@mui/icons-material";
```
这会导入 `@mui/icons-material` 的全部 2000+ 个图标组件，严重增加 bundle 体积。

### 修改

改为按需导入 `iconMap` 中实际使用的图标。从 `iconMap` 对象中提取所有引用的图标名称，然后逐个具名导入：

```js
// 替换 import * as MuiIcons from "@mui/icons-material"
import {
  Language, Public, Cloud, CloudQueue, Storage, Dns,
  Security, Shield, VpnKey, Lock, LockOpen,
  Safari, Apple, Android, Chrome, Firefox,
  Mail, Chat, Message, Phone,
  Videocam, Camera, MusicNote, Headphones,
  Games, Sports, Movie, Image,
  GitHub, Code, Terminal, DeveloperMode,
  Shop, Store, ShoppingBag, Restaurant, Coffee,
  Flight, DirectionsCar, Map, Navigation,
  School, Work, Business, Home,
  // ... 从 iconMap 中提取所有用到的图标名称
} from "@mui/icons-material";

const MuiIcons = {
  Language, Public, Cloud, CloudQueue, Storage, Dns,
  Security, Shield, VpnKey, Lock, LockOpen,
  Safari, Apple, Android, Chrome, Firefox,
  Mail, Chat, Message, Phone,
  Videocam, Camera, MusicNote, Headphones,
  Games, Sports, Movie, Image,
  GitHub, Code, Terminal, DeveloperMode,
  Shop, Store, ShoppingBag, Restaurant, Coffee,
  Flight, DirectionsCar, Map, Navigation,
  School, Work, Business, Home,
  // ...
};
```

或者更简洁地使用 Vite 的 `optimizeDeps.include` 配置只包含需要的图标。

---

## P11: process.rs 中 get_processes_impl 是同步阻塞操作

### 文件
`src-tauri/src/commands/process.rs` 第 5-115 行

### 问题
`get_processes_impl` 在 async 函数中直接使用 `sysinfo::System::new_all()` 和 `run_cmd()`，这些是 CPU 密集型同步操作，会阻塞 tokio 运行时。`lsof` 和 `ps` 命令调用也可能耗时数百毫秒。

### 修改

用 `tokio::task::spawn_blocking` 包装整个函数体：

```rust
pub async fn get_processes_impl() -> Result<Vec<ProcessInfo>, String> {
    tokio::task::spawn_blocking(|| {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        // ... 其余逻辑不变 ...
    })
    .await
    .map_err(|e| format!("进程查询任务失败: {}", e))?
}
```

同理，`get_connections_impl` 也应该用 `spawn_blocking` 包装。

---

## P12: proxy.rs get_proxy_state_impl 缺少 async 但被当 async 调用

### 文件
`src-tauri/src/commands/proxy.rs` 第 4 行

### 问题
```rust
pub async fn get_proxy_state_impl() -> Result<ProxyState, String> {
```
函数标记为 `async` 但内部没有任何 `.await`，全部是同步的 `std::process::Command` 调用。这在技术上不会出错（async 函数可以不 await），但会在每次调用时占用一个 tokio 工作线程。虽然不严重，但与 `network.rs` 中调用它的方式（`get_proxy_state_impl().await`）不一致。

### 修改建议
保持 `async` 签名（因为被 `.await` 调用），但用 `spawn_blocking` 包装同步部分：

```rust
pub async fn get_proxy_state_impl() -> Result<ProxyState, String> {
    tokio::task::spawn_blocking(|| {
        let service = primary_network_service();
        let http = run_cmd_stderr(&["networksetup", "-getwebproxy", &service]).unwrap_or_default();
        let socks = run_cmd_stderr(&["networksetup", "-getsocksfirewallproxy", &service]).unwrap_or_default();
        // ... 其余逻辑不变 ...
    })
    .await
    .map_err(|e| format!("获取代理状态失败: {}", e))?
}
```

---

## 修改优先级总结

| 优先级 | 编号 | 标题 | 影响 |
|--------|------|------|------|
| **紧急** | P1 | TUN 路由字段名错误 | 增强模式无法正常工作 |
| **高** | P3 | App.jsx 引擎生命周期不对称 | 资源泄漏、关闭代理后进程残留 |
| **高** | P4 | MITM 开关不持久化 | 用户设置丢失 |
| **高** | P5 | build_runtime_config 不输出 MITM | MITM 功能实际不生效 |
| **中** | P2 | traffic.rs 无操作语句 | 首次采样可能返回异常速度值 |
| **中** | P7 | CA is_trusted 始终 false | 用户无法知道证书是否被系统信任 |
| **中** | P8 | ProcessDetail 硬编码数据 | 详情面板数据不真实 |
| **中** | P9 | CapturePage 多 Tab 空壳 | 5/6 功能不可用 |
| **中** | P11 | process.rs 阻塞 tokio | 可能导致 UI 卡顿 |
| **低** | P6 | useTauriPoll 缺少 args 依赖 | 动态参数场景数据不更新 |
| **低** | P10 | import * 导致包体积膨胀 | 首次加载慢 |
| **低** | P12 | proxy.rs 同步操作占 async | 轻微性能影响 |

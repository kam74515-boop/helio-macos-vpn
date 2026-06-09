# Helio 代码审查报告

生成时间：2026-06-09
审查范围：Rust 后端 + React 前端 + 构建配置
基线提交：d857244

---

## 一、Rust 后端审查

### 1.1 singbox.rs — 引擎与配置管理

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🔴 高 | 配置读取逻辑重复 | L11-L22, L25-L33 | `get_singbox_config_json` 和 `get_singbox_config` 有完全相同的文件读取+JSON解析逻辑，应提取为 `read_config_file()` 公共函数 |
| 🔴 高 | sidecar 启动逻辑重复 | L215-L260, L262-L312 | `update_singbox_config` 和 `start_engine` 都有相同的 kill→spawn→log relay 流程，应提取为 `restart_engine()` |
| 🟡 中 | 默认配置可能不合法 | L188-L212 | `default_singbox_config()` 的 route.final="Proxy"，但 outbounds 中 Proxy selector 的 default="direct"，如果用户没有导入节点，Proxy 指向 direct 是合法的，但配置语义上容易混淆 |
| 🟡 中 | 模式切换不验证引擎状态 | L176-L177 | `set_proxy_mode` 调用 `start_engine` 后没有等待确认 sidecar 是否成功启动，如果启动失败用户不会收到明确反馈 |
| 🟡 中 | `_helio` 元数据字段非标准 | L104-L108 | 在 sing-box 配置中注入 `_helio` 字段，sing-box 可能会忽略或在未来版本中报错，建议改用独立 metadata 文件 |
| 🟢 低 | 规则解析可读性差 | L54-L89 | 长链 `else if` 规则类型判断，建议用 match 或查表方式重构 |
| 🟢 低 | `has_runtime_config_shape` 过于宽松 | L314-L321 | 只检查 inbounds/outbounds/route 存在，不验证 sing-box 兼容性 |

### 1.2 network.rs — 网络与测速

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🔴 高 | 魔法数字 999.0 | L339 | 连接失败时的延迟固定返回 999.0，应定义为常量 `LATENCY_TIMEOUT_MS` |
| 🟡 中 | `spawn_blocking` 包裹 `connect_timeout` | L319-L327 | `TcpStream::connect_timeout` 本身已有超时机制，`spawn_blocking` 增加了不必要的线程调度开销 |
| 🟡 中 | 外部 IP 获取依赖 curl 命令 | L202-L217 | 在 App 打包后 curl 可能不可用，应改用 reqwest 进行 HTTP 请求 |
| 🟡 中 | 硬编码网络接口 en0 | L165, L184 | `get_ssid` 和 `get_local_ip` 都硬编码 `en0`，在有线网络或接口名不同的 Mac 上会失败 |
| 🟡 中 | ping 解析逻辑脆弱 | L90-L107 | `parse_ping_avg` 对 "round-trip min/avg/max/stddev" 格式使用 `split('/')`，如果输出格式变化（如不同 locale）会解析错误 |
| 🟢 低 | 测速 fallback 逻辑 | L256-L263 | 如果节点不在配置中，fallback 到 ping 节点名，但节点名通常不是有效主机名 |
| 🟢 低 | `get_system_snapshot` 硬编码 8.8.8.8 | L41 | 延迟测试目标固定为 Google DNS，应可配置 |

### 1.3 subscription.rs — 订阅解析

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🔴 高 | 测试覆盖不足 | L445-L493 | 只有 vless/vmess/ss 三种协议的单元测试，缺少 trojan/hysteria2/tuic/anytls 的测试 |
| 🟡 中 | 每次导入创建新 reqwest Client | L27 | `reqwest::Client::new()` 每次调用都创建新连接池，应复用或至少设置超时 |
| 🟡 中 | `looks_like_subscription_body` 误判 | L68-L71 | 包含换行符就认为是订阅体，但单个 URI 也可能有换行符 |
| 🟡 中 | 手写 percent_decode | L415-L430 | 应使用 `percent-encoding` crate，手写实现可能有边界情况遗漏 |
| 🟡 中 | `parse_host_port` IPv6 端口缺失 | L323-L330 | IPv6 格式 `[host]:port` 中如果缺少 `:port`，`strip_prefix(':')` 返回 None 导致整行解析失败 |
| 🟢 低 | `parse_standard_proxy` 密码为空即失败 | L180-L205 | trojan/hysteria2 密码为空时返回 None，但某些场景下空密码是合法的 |

### 1.4 proxy.rs — 系统代理

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟡 中 | 只检查 exit code 不检查 stderr | L103-L108 | `run_networksetup` 只检查 `status.success()`，但 networksetup 可能在成功状态下输出错误 |
| 🟡 中 | 回滚逻辑不完整 | L91-L97 | 只在开启代理失败时回滚，关闭代理失败时没有处理 |
| 🟡 中 | 硬编码代理端口 6152 | L21-L23 | `http_port` 和 `socks_port` 默认值写死为 "6152"，应与 sing-box 配置中的 listen_port 保持一致 |
| 🟢 低 | `primary_network_service()` 解析脆弱 | L27-L46 | 依赖 `networksetup -listallhardwareports` 的固定输出格式 |

### 1.5 traffic.rs — 流量统计

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🔴 高 | 流量估算极不准确 | L32-L39 | `netstat -ib` 的 `ipkts * 1500` 估算非常粗糙，实际包大小从 64B 到 1500B 不等，误差可达 20 倍以上 |
| 🟡 中 | 首次调用返回 0 | L71-L77 | `upload_kbps`/`download_kbps` 在首次调用时 `prev_tx == 0` 返回 0，需要至少两次调用才有有效数据 |
| 🟡 中 | `extract_bytes` 阈值过滤 | L107 | `if n > 1000` 可能过滤掉小流量接口的合法数据 |
| 🟢 低 | history 时间间隔不固定 | L80-L88 | 历史数据只保留 24 个样本，但采样间隔取决于调用频率 |

### 1.6 process.rs — 进程与连接

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🔴 高 | 误用 disk_usage 作为网络流量 | L50-L53 | `sysinfo::Process::disk_usage()` 返回磁盘 I/O，不是网络流量，这是概念性错误 |
| 🟡 中 | 每次调用重建 System | L45-L46 | `sysinfo::System::new_all()` 每次调用都全量扫描进程，在进程多时性能开销大 |
| 🟡 中 | `chrono_local()` 硬编码 UTC+8 | L133-L142 | `hours = ((secs % 86400) / 3600 + 8) % 24` 硬编码东八区，应使用系统时区 |
| 🟡 中 | lsof 输出解析脆弱 | L84-L126 | `get_connections_impl` 依赖 lsof 的固定列位置，不同版本输出格式可能不同 |
| 🟢 低 | Pid 类型转换 | L51 | `sysinfo::Pid::from(*pid as usize)` 在 macOS 上 `Pid` 可能是 `u32`，`usize` 转换可能不兼容 |

### 1.7 types.rs / state.rs — 数据模型

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟡 中 | 使用 `std::sync::Mutex` 而非 `tokio::sync::Mutex` | state.rs L5 | 在 async 上下文中 `std::sync::Mutex` 会阻塞运行时线程，应使用 `tokio::sync::Mutex` |
| 🟡 中 | 数值类型用 String | types.rs L51-L52 | `ping` 和 `state` 是 `String`，`hits` 也是 `String`，应使用更具体的类型 |
| 🟢 低 | `TrafficSnapshot` 无类型约束 | state.rs L11-L17 | `history_rx`/`history_tx` 的 24 样本限制只在运行时检查 |

---

## 二、前端审查

### 2.1 App.jsx — 应用根组件

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟡 中 | 默认代理名是 mock 值 | L21 | `selectedProxy` 默认值为 `"tu5-VM-0-11-ubuntu"`，在真实环境下不存在，应设为空或 "direct" |
| 🟡 中 | 引擎启动无状态检查 | L44 | `safeInvoke("start_engine")` 在每次挂载时调用，如果引擎已在运行会重复 kill/start |
| 🟡 中 | `pageProps` 对象每次重建 | L57 | 每次渲染都创建新对象，可能导致子组件不必要的重渲染，应使用 `useMemo` |
| 🟢 低 | unlisten 处理不清晰 | L47-L53 | `unlisten.then?.(fn => fn())` 语法虽然能工作，但可读性差 |

### 2.2 ActivityPage.jsx — 活动页

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟡 中 | 依赖数组使用 join | L31 | `[proxyOptions.join("|"), ...]` 能工作但不够优雅，应使用更稳定的依赖管理 |
| 🟡 中 | 嵌套三元运算符 | L47 | 模式标签映射使用三层嵌套三元，建议提取为常量映射表 |
| 🟡 中 | Toast 显示长文本 | L60 | `summary` 可能包含多行节点测速结果，Toast 容器可能显示不下 |
| 🟢 低 | mock 数据回退 | L33-L44 | 在 Tauri 环境下仍使用 mock 数据作为 fallback |

### 2.3 PolicyPage.jsx — 代理配置页

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟡 中 | 乐观更新问题 | L34 | `setMode(newMode)` 在调用后端前更新 UI，如果后端失败 UI 与真实状态不一致 |
| 🟡 中 | 模式状态双轨维护 | L9, L36 | 同时维护中文标签 `mode` 和英文 key，容易不同步，建议只维护 key 并派生标签 |
| 🟢 低 | `window.prompt` 样式 | L53 | 在 Tauri 环境下 `window.prompt` 使用系统对话框，与 App 风格不一致 |

### 2.4 Toast.jsx — 通知系统

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟡 中 | ID 可能冲突 | L9 | `Date.now() + Math.random()` 在极端情况下可能冲突，建议使用 `crypto.randomUUID()` |
| 🟡 中 | setTimeout 未清理 | L11-L13 | 组件卸载时未清理 pending 的 setTimeout，可能导致内存泄漏 |
| 🟢 低 | 换行符不渲染 | L26 | `toast-message` 中的 `\n` 在 HTML 中不会显示为换行，需要 `white-space: pre-line` |

### 2.5 hooks/tauri.js — Tauri Hooks

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟡 中 | args 变化不触发重新获取 | L20 | `useTauriPoll` 的依赖数组只有 `[command, interval]`，`args` 变化不会重新触发 effect |
| 🟡 中 | 错误未暴露给调用方 | L15 | 错误只打印 console.error，hook 没有返回 error 状态 |
| 🟢 低 | `useTauriData` 和 `useTauriPoll` 功能重叠 | 整体 | 两个 hook 功能相似，可以合并或明确分工 |

### 2.6 utils/tauri.js — Tauri 工具

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟡 中 | 依赖内部 API | L4 | `window.__TAURI_INTERNALS__` 是 Tauri 内部 API，可能在版本升级后失效 |
| 🟢 低 | `safeInvoke` 返回类型不一致 | L7-L9 | Tauri 环境下返回 Promise，非 Tauri 返回 null，调用方需要额外处理 |

---

## 三、构建与配置审查

### 3.1 tauri.conf.json

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟡 中 | CSP 为 null | L27 | `"csp": null` 关闭了内容安全策略，生产环境应配置合理的 CSP |
| 🟡 中 | 更新器配置是占位符 | L48-L53 | updater endpoints 使用 `example.com`，pubkey 是 `UPDATE_SIGNING_PUBLIC_KEY` 占位符 |
| 🟢 低 | Android 配置残留 | L43-L45 | 项目目标为 macOS，但保留了 Android 配置 |

### 3.2 Cargo.toml

| 严重程度 | 问题 | 位置 | 说明 |
|---------|------|------|------|
| 🟢 低 | 缺少 `percent-encoding` | 依赖列表 | `subscription.rs` 手写 percent_decode，应引入 `percent-encoding` crate |
| 🟢 低 | `tokio` features 可能不足 | L28 | 当前 features: `["process", "time", "macros"]`，如果后续需要 fs/io 可能需要扩展 |

---

## 四、架构层面问题

### 4.1 状态同步
- 前端 `systemProxy` 状态由前端维护，但可能被外部（如用户手动修改系统代理）修改，导致前后端状态不一致
- 建议：增加定期同步机制，或监听系统代理变化事件

### 4.2 配置管理
- 没有配置版本控制：修改配置后无法回滚到之前版本
- 没有配置验证：写入 config.json 前只检查 JSON 格式，不运行 `sing-box check` 验证
- 没有配置备份：config.json 被覆盖后无法恢复
- 建议：引入配置版本号、备份机制和 sing-box 预验证

### 4.3 错误恢复
- sidecar 启动失败后没有重试或降级策略
- 建议：增加启动重试（最多 3 次）和失败后的明确用户提示

### 4.4 日志管理
- sing-box 的 stdout/stderr 只通过 `log::info` 输出到 Tauri 日志插件，没有持久化到文件
- 建议：增加日志文件轮转，方便用户排查问题

---

## 五、安全问题

| 严重程度 | 问题 | 说明 |
|---------|------|------|
| 🟡 中 | 订阅 URL 无超时 | `import_subscription` 中 reqwest Client 没有设置超时，可能导致长时间挂起 |
| 🟡 中 | 凭证明文存储 | 订阅解析出的节点凭证（UUID、密码）明文存储在 config.json 中 |
| 🟡 中 | 路径非 UTF-8 风险 | `start_engine` 中 `config_path.to_str().unwrap()` 可能 panic |
| 🟢 低 | networksetup 参数未过滤 | `set_system_proxy` 中 `service` 如果包含特殊字符可能导致命令注入（虽然 macOS 上概率极低） |

---

## 六、优先级修复建议

### P0（立即修复）
1. **process.rs 中 disk_usage 误用** — 这是概念性错误，进程页显示的是磁盘 I/O 而非网络流量
2. **测试覆盖补充** — 为 trojan/hysteria2/tuic/anytls 添加单元测试
3. **流量估算准确性** — `netstat -ib` 的 `ipkts * 1500` 估算需要替换为更精确的方法

### P1（近期修复）
1. 提取 `singbox.rs` 中的公共函数（配置读取、引擎重启）
2. 将 `std::sync::Mutex` 替换为 `tokio::sync::Mutex`
3. 使用 `percent-encoding` crate 替代手写 percent_decode
4. 为 reqwest Client 设置全局超时
5. 修复 Toast 的 setTimeout 清理和换行符渲染
6. 统一模式状态管理（PolicyPage 中只维护 key，派生标签）

### P2（中期优化）
1. 引入配置版本控制和备份机制
2. 增加 sing-box 配置预验证（`sing-box check`）
3. 使用 reqwest 替代 curl 获取外部 IP
4. 增加 sidecar 启动失败重试机制
5. 配置 CSP 安全策略
6. 修复 `useTauriPoll` 的 args 依赖问题

### P3（长期改进）
1. 引入 Profile 管理系统
2. 增加日志文件持久化
3. 支持有线网络接口自动检测
4. 增加配置导入/导出功能
5. 完善错误恢复和降级策略

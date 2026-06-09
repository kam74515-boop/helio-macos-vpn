# Helio macOS 真实化实现计划

> 目标：保留当前 Surge 信息结构 + Material 3 扁平化前端，把 Helio 从“可视化原型 + sing-box sidecar”推进到“可运行、可授权、可诊断的 macOS 代理/VPN 应用”。

## 1. 当前判断

### 1.1 图标为什么比 Surge 糊

当前实现从进程路径找到 `.app` 包，再用 `sips` 把 `.icns` 转成 PNG。此前转换尺寸是 32px，但 UI 中进程图标通常显示为 34-42 CSS px，在 Retina 屏幕上实际需要约 68-84 物理像素，所以会被浏览器/WebView 放大，显得糊。

Surge 是原生 macOS 应用，通常使用 `NSWorkspace` / `NSImage` 获取 app icon。`NSImage` 保留多倍率 image representation，AppKit 会根据屏幕 scale 选择高分辨率版本，所以更清晰。

本轮修复方向：

- 将 `.icns -> PNG` 转换尺寸提升到 128px。
- 真实 app 图标不再套通用彩色底块和额外阴影。
- WebView 中按 CSS 尺寸缩放显示高分辨率 PNG。

仍需后续增强：

- 增加按 bundle id / bundle path 的持久图标缓存，避免每次重启重新转换。
- 对 helper 进程做父应用归并，例如 `Google Chrome Helper` 归并到 `Google Chrome.app`。
- 如未来接入 Swift/Objective-C bridge，可直接用 `NSWorkspace.shared.icon(forFile:)` 导出 256px PNG。

### 1.2 当前真实能力

已具备：

- sing-box sidecar 打包进 Tauri app。
- 订阅导入：支持 URI/base64 节点内容，覆盖 VLESS、VMess、Trojan、Hysteria2、TUIC、AnyTLS、Shadowsocks。
- 代理页新增/编辑/删除节点，写入 `config.json` 的 `outbounds[]`。
- 策略组新增，写入 sing-box `type: "selector"` outbound。
- 活动页选择代理/策略组，写入 selector `default`。
- 进程列表通过 `sysinfo`、`lsof`、`ps` 获取真实进程，部分进程可显示真实 app icon。
- macOS system proxy 开关已有基础实现。

未真实完成：

- TUN / 增强模式授权接管。
- sing-box Clash API / libbox command server 数据接入。
- 规则编辑完整 CRUD。
- HTTP 捕获、HTTPS MITM、重写和 mock。
- 设备 / 网关 / DHCP。
- 完整配置页、DNS 页、脚本页、更新授权页。
- 可分发签名、公证、Network Extension entitlement。

## 2. 目标架构

### 2.1 保留当前前端

当前 React/Tauri 前端继续保留，作为主 UI：

- 页面结构继续对标 Surge：活动、概览、进程、设备、策略、规则、捕获、解密、重写、更多。
- 所有按钮必须对应真实 Tauri command 或显示明确不可用原因。
- 所有配置修改必须落盘到 app data 或 profile database，不能只改 React state。

### 2.2 后端分层

推荐拆成 5 层：

1. `ConfigStore`
   - 管理 profiles、订阅源、节点、策略组、规则、DNS、MITM、TUN 参数。
   - 输出 sing-box runtime config。

2. `EngineController`
   - 启停 sing-box。
   - 校验 `sing-box check`。
   - 监听 stdout/stderr。
   - 维护 engine health。

3. `TelemetryService`
   - 活动连接、进程归属、流量统计、DNS 延迟、节点延迟。
   - 优先接入 sing-box experimental Clash API / libbox command server。

4. `MacSystemService`
   - system proxy。
   - app icon、进程路径、bundle id。
   - 权限诊断。
   - 后续接 Network Extension / System Extension / privileged helper。

5. `FeatureServices`
   - CaptureService。
   - MitmService。
   - RewriteService。
   - GatewayService。
   - ScriptService。

## 3. TUN / 增强模式方案

### 3.1 参考项目事实

本地参考：

- `/Users/karl/apps/vpn/_refs/sing-box/docs/configuration/inbound/tun.md`
- `/Users/karl/apps/vpn/_refs/sing-box-for-apple/Library/Network/ExtensionProvider.swift`
- `/Users/karl/apps/vpn/_refs/sing-box-for-apple/Library/Network/ExtensionPlatformInterface.swift`
- `/Users/karl/apps/vpn/_refs/sing-box-for-apple/SFI/SFI.entitlements`
- `/Users/karl/apps/vpn/_refs/sing-box-for-apple/SFM.System/SFM.entitlements`

关键结论：

- sing-box 支持 `type: "tun"` inbound，macOS 在支持范围内。
- `sing-box-for-apple` 正式路径是 `NEPacketTunnelProvider` + Libbox。
- App Store / 正式分发需要 Apple Developer 账号、Network Extension entitlement、正确 provisioning profile、签名和公证。
- System Extension 版本还需要 `com.apple.developer.system-extension.install` 和额外用户批准。

### 3.2 两条实现路线

#### 路线 A：开发期 CLI TUN / Root Helper

用途：快速验证 UI、规则、节点、DNS、TUN config 是否可用。

做法：

- 继续使用当前 sing-box sidecar。
- 增加 `tun` inbound config。
- 通过 privileged helper 或管理员权限启动需要高权限的 TUN。
- UI 提供“授权增强模式”流程和失败诊断。

优点：

- 对当前 Tauri 工程改动较小。
- 可快速验证 sing-box 配置和页面逻辑。

风险：

- 不是最适合正式分发的 macOS VPN 形态。
- 管理员权限、launchd helper、签名、公证会带来维护成本。

#### 路线 B：正式 Network Extension / Packet Tunnel

用途：真正对标 Surge 的 macOS VPN 接管完成度。

做法：

- 新增 macOS Network Extension target。
- 参考 `sing-box-for-apple` 的 `ExtensionProvider` 和 `ExtensionPlatformInterface`。
- 将 sing-box 从 CLI sidecar 迁移到 libbox command server。
- 主 app 通过 Tauri command 调 Swift bridge，Swift bridge 管理 `NETunnelProviderManager`。
- 用户在 macOS 系统弹窗中批准 VPN 配置。

优点：

- macOS 官方 VPN 授权模型。
- 更接近 Surge / sing-box-for-apple 的生产路径。
- 更利于稳定接管、状态同步和日志诊断。

风险：

- 需要 Apple Developer Team ID 和 Network Extension entitlement。
- Tauri + Xcode extension 工程集成复杂。
- 没有签名和 entitlement 时无法完整本地验证。

推荐策略：

- M1 先做路线 A，验证产品逻辑。
- M2-M3 并行搭建路线 B，最终以 Network Extension 为生产路径。

### 3.3 TUN runtime config 草案

```json
{
  "type": "tun",
  "tag": "tun-in",
  "address": ["172.18.0.1/30", "fdfe:dcba:9876::1/126"],
  "mtu": 9000,
  "auto_route": true,
  "strict_route": true,
  "stack": "system",
  "dns_mode": "hijack",
  "dns_address": ["172.18.0.2"]
}
```

注意：

- `auto_route`、`strict_route`、DNS hijack 必须根据 macOS 路由和 Network Extension 实际行为测试。
- TUN 模式要自动排除本机代理端口、局域网、Apple push/APNs 等关键地址，避免断网。

## 4. 页面真实化计划

### 4.1 活动页

目标：

- 显示真实 engine status、网络、配置、出站模式、外部 IP。
- 显示实时上传/下载、活动连接、总流量、代理节点延迟。
- 策略组和代理选择写入 sing-box selector。

数据源：

- sing-box Clash API：proxies、connections、traffic。
- 本地系统：Wi-Fi、system proxy、external IP、latency。

验收：

- 断网、订阅为空、节点失败时 UI 明确显示原因。
- 切换代理后 `config.json` 和 engine 行为同步。

### 4.2 概览页

目标：

- 系统代理：真实开关。
- 增强模式：TUN 授权、启停和状态。
- HTTP/SOCKS5：监听地址、端口、局域网访问开关。
- 网关模式：先显示未实现诊断，后续接 GatewayService。

验收：

- 每张卡片点击后都有配置面板。
- 不可用状态显示缺少什么权限或能力。

### 4.3 进程页

目标：

- 显示真实进程、真实 app icon、连接数、累计流量。
- 连接归属尽量从 sing-box/libbox connection owner 获取，fallback 到 `lsof`。

数据源：

- `LibboxFindConnectionOwner`。
- `lsof` / `ps` / `sysinfo`。
- app bundle icon cache。

验收：

- 常见 App：Chrome、Cursor、微信、飞书、终端、xray/sing-box 能显示真实名称和图标。
- 无权限或系统保护进程显示降级原因。

### 4.4 设备页

目标：

- TUN 模式下显示本机网络接口和路由状态。
- 网关模式下显示局域网设备。

数据源：

- ARP / neighbor table。
- DHCP helper。
- sing-box routing stats。

验收：

- 未开启网关时为空态必须说明原因。
- 开启网关需要管理员授权和安全提示。

### 4.5 策略页

目标：

- 完成节点导入、手动添加、编辑、删除。
- 完成 selector/urltest/fallback 策略组。
- 支持 Clash/Mihomo YAML 转 sing-box。

验收：

- 节点保存后 `outbounds[]` 正确更新。
- 策略组保存后 selector/urltest/fallback 正确更新。
- 订阅更新失败不会覆盖现有可用配置。

### 4.6 规则页

目标：

- 显示真实 route rules。
- 支持新增、编辑、删除、排序、命中计数。
- 支持 rule-set 文件下载、编译、校验。

验收：

- 规则编辑后可通过 `sing-box check`。
- 规则顺序变化立即反映在 config。

### 4.7 捕获页

目标：

- 显示真实 HTTP/TCP 连接列表。
- 支持按客户端、域名、方法、策略过滤。

关键限制：

- sing-box 本身不是完整 HTTP Debugger。
- 要达到 Surge 的 HTTP capture，需要额外 CaptureService：本地 HTTP proxy、请求摘要存储、body 限制、敏感信息脱敏。

验收：

- 未开启 capture 时不会假显示请求。
- 开启后能看到真实请求元数据。

### 4.8 解密页

目标：

- 生成 CA。
- 安装/信任 CA。
- 管理 MITM hostname 列表。
- 支持 QUIC 屏蔽和 HTTP/2 MITM 选项。

关键限制：

- CA 信任需要系统授权。
- 部分 App certificate pinning 无法解密，必须在 UI 标注。

验收：

- CA 状态可检测。
- 未信任 CA 时捕获 HTTPS 显示明确失败原因。

### 4.9 重写页

目标：

- URL redirect。
- Header rewrite。
- Body rewrite。
- Mock response。

关键限制：

- 需要 MITM/CaptureService 配合。
- sing-box route rules 不能单独完成 Surge 级 HTTP rewrite。

验收：

- 每条 rewrite 规则有启用状态、命中次数、测试入口。
- 修改规则后实时生效或明确提示需要重启。

### 4.10 更多 / 设置页

目标：

- 通用设置、外观、DNS、模块、配置、授权与更新、脚本。
- 所有入口有真实子页面。

验收：

- 任何设置项不能只是静态卡片。
- 与 engine、profile、system permission 有清晰绑定。

## 5. 里程碑

### M0：视觉与窗口适配修复

状态：本轮开始执行。

范围：

- 修复图标糊。
- 修复小窗口挤压、溢出、内容被裁切。
- 保证所有页面在 900x600、1200x700、1440x900 可用。

验收：

- `npm run build` 通过。
- `cargo test` 通过。
- macOS app 打包通过。
- 小窗口手动检查无明显遮挡。

### M1：配置真实化闭环

范围：

- 完成 Profile / ConfigStore。
- YAML 订阅导入。
- `sing-box check` 保存前校验。
- 节点、策略组、规则、DNS 全部落盘。

验收：

- 任一 UI 配置修改都能定位到 config diff。
- 重启 app 后状态不丢。

### M2：活动与进程真实化

范围：

- 接入 Clash API / libbox command server。
- 真实连接列表。
- 真实流量曲线。
- 进程归属和 app icon cache。

验收：

- 活动页与进程页不再依赖 mock。
- 连接列表和进程流量能随真实网络变化刷新。

### M3：TUN 授权和增强模式

范围：

- 开发期：CLI TUN / privileged helper。
- 正式期：Network Extension target。
- 权限诊断页。
- TUN route/DNS 防断网策略。

验收：

- 用户授权后增强模式能接管不遵循系统代理的 App。
- 授权失败、签名缺失、entitlement 缺失都有明确提示。

### M4：HTTP Capture / MITM / Rewrite

范围：

- CaptureService。
- CA 管理。
- Hostname whitelist。
- Rewrite rule engine。
- Mock response。

验收：

- HTTP 请求可见。
- HTTPS 在 CA 信任和 hostname 命中时可解密。
- Rewrite / Mock 有命中计数和测试结果。

### M5：设备 / 网关 / 分发

范围：

- Gateway/DHCP。
- LAN device view。
- Developer ID 签名。
- 公证。
- 自动更新。

验收：

- 可稳定安装、启动、授权、更新。
- 用户不需要命令行操作。

## 6. 风险与决策

### 6.1 必须尽早决策

- 是否申请 Apple Network Extension entitlement。
- 是否接受 Tauri + Swift Network Extension 混合工程。
- 是否商业闭源；这会影响参考项目和依赖许可证选择。

### 6.2 技术风险

- Tauri app 直接集成 Network Extension 的工程复杂度高。
- CLI TUN 与正式 Network Extension 行为不完全一致。
- Surge 级 MITM/Rewrite 不是 sing-box 单独能覆盖的能力。
- 进程流量统计需要 connection owner 和流量采样，不是普通进程列表能推导出来。

### 6.3 产品风险

- 页面全部真实化后，很多功能会出现“需要授权/需要签名/需要配置”的中间状态。
- UI 必须表达真实状态，不能用漂亮卡片掩盖不可用能力。

## 7. 下一步执行顺序

1. 完成本轮 UI 清晰度和窗口适配验证。
2. 把所有页面的静态按钮替换为真实命令或明确不可用状态。
3. 建立 `ConfigStore`，把配置从单文件操作收敛成 profile 模型。
4. 接入 sing-box Clash API / libbox command server。
5. 做开发期 CLI TUN 验证。
6. 搭建 Network Extension PoC，确认签名、entitlement、Tauri 集成路径。

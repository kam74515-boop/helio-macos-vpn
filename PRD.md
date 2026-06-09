# Helio macOS 代理与 VPN 客户端 PRD

文档版本：v0.1  
更新时间：2026-06-09  
产品阶段：MVP / 技术验证  
目标平台：macOS Apple Silicon 优先  
仓库：`https://github.com/kam74515-boop/helio-macos-vpn`

## 1. 产品概述

Helio 是一个面向 macOS 的开源代理与 VPN 客户端。产品目标是以 Surge 的信息结构和完成度为长期参照，保留当前已实现的 Surge 风格可视化前端，并逐步接入 `sing-box` 与 `sing-box-for-apple` 的成熟内核和 Apple 平台能力。

短期目标不是重新手写一个代理内核，也不是只做静态 UI，而是先把当前前端变成一个能真实导入订阅、启动本地代理、切换系统代理、展示基础网络状态的 macOS App。中长期目标是通过 `sing-box-for-apple` 的 Network Extension、Profile 管理、Libbox 命令通道和系统扩展能力，补齐 TUN、全局透明代理、真实连接观测、日志、远程配置更新等 Surge 级基础能力。

## 2. 背景与问题

现有代理客户端大多存在三类问题：

- 消费级客户端功能简单，适合一键连接，但不适合开发者观察连接、规则命中、进程流量、HTTP 请求和配置调试。
- 工具型客户端能力强，但 UI 信息密度和视觉体验偏工程化，用户需要较高学习成本。
- macOS 上完整代理能力依赖系统代理、Network Extension、TUN、权限和签名公证，单纯手写桌面壳很难达到稳定可用。

Helio 的产品机会是：用更清晰、更扁平、更像现代 macOS 工具的可视化界面承接成熟内核能力，让用户既能快速连接，也能像使用 Surge 一样查看和管理网络行为。

## 3. 产品定位

Helio 定位为：

- 面向 macOS 的开源代理与 VPN 可视化客户端。
- 内核基于 `sing-box`，平台能力优先复用 `sing-box-for-apple`。
- UI 信息结构对标 Surge，视觉风格采用 Material 3 扁平化表达与 macOS 原生窗口骨架。
- 用户既可以导入订阅快速连接，也可以查看活动、规则、进程、设备、捕获、MITM、重写等高级功能入口。

Helio 不定位为：

- 自研代理协议内核。
- 仅用于展示的静态设计稿。
- 闭源商业 VPN 品牌。
- 第一阶段就完整复刻 Surge 的所有高级能力。

## 4. 目标用户

### 4.1 开发者 / 高级代理用户

需要导入多协议节点，管理规则，观察请求和进程流量，排查代理失败、DNS、延迟和规则命中问题。

核心诉求：

- 节点和订阅稳定可用。
- 系统代理/TUN 行为明确可控。
- 活动页能看到真实连接、延迟、流量、进程。
- 规则、捕获、MITM、重写有清晰入口。

### 4.2 设计和产品用户

需要一个比传统代理客户端更易读、更现代的 macOS 工具界面，能快速理解当前连接状态和配置。

核心诉求：

- UI 精致、信息分组清楚。
- 不需要理解复杂配置也能连接。
- 错误状态可读，不出现“按钮可点但功能无效”。

### 4.3 开源贡献者

希望基于成熟内核和清晰 UI 继续开发 macOS 代理客户端能力。

核心诉求：

- 架构边界清楚。
- 前端、内核适配层、平台能力分层。
- 文档明确当前完成度和未来任务。

## 5. 产品目标

### 5.1 MVP 目标

MVP 需要达到“可作为本机 macOS 代理客户端试用”的最低标准：

- 可以安装和启动 `Helio.app`。
- 可以启动内置 `sing-box` sidecar。
- 可以导入 URI/base64 节点订阅。
- 可以生成可被 `sing-box check` 通过的配置。
- 可以打开/关闭系统代理。
- 活动页展示真实网络摘要、系统代理状态、延迟、基础流量、进程和连接信息。
- 代理配置页展示真实节点和策略组。
- 功能不可用时必须明确提示，不允许用假数据伪装可用。

### 5.2 中期目标

- 接入 `sing-box-for-apple` 的 Network Extension 能力。
- 支持 TUN / 增强模式 / 全局透明代理。
- 引入 Profile 管理、远程订阅更新、配置校验和格式化。
- 引入真实日志、命令通道、连接列表、出站组状态。
- 支持常见协议的订阅导入和转换。

### 5.3 长期目标

- 形成接近 Surge 的 macOS 网络控制台完成度。
- 支持 HTTP 捕获、HTTPS 解密、重写、脚本、规则命中统计、设备网关模式。
- 建立正式发布链路：签名、公证、自动更新、GitHub Release。
- 建立可持续开源协作规范。

## 6. 成功指标

### 6.1 MVP 指标

- App 首次启动成功率 >= 95%。
- 默认配置 `sing-box check` 通过率 100%。
- URI/base64 订阅导入成功后，节点列表可见率 100%。
- 打开系统代理后，macOS 当前网络服务代理状态读取一致率 >= 95%。
- 关闭系统代理后，不遗留 HTTP/HTTPS/SOCKS 代理状态。
- App 内不出现假按钮、假导入成功、假启动成功。

### 6.2 产品体验指标

- 用户能在 30 秒内完成导入订阅并连接。
- 活动页一屏内能看清核心状态：网络、配置、出站模式、外部 IP、延迟、上下行、连接数、总流量。
- 关键错误需要在用户操作后 1 秒内反馈。

### 6.3 工程指标

- `npm run build` 通过。
- `cargo test` 通过。
- `npx tauri build` 通过。
- 核心配置生成逻辑有测试覆盖。
- macOS App 产物包含主程序与 `sing-box` sidecar。

## 7. 产品范围

### 7.1 MVP 范围

MVP 包含：

- macOS App 壳。
- 当前 React 前端页面保留。
- `sing-box` sidecar 启动/停止。
- 基础订阅导入。
- 基础配置生成。
- 系统代理开关。
- 活动页基础数据。
- 代理配置页基础节点和策略组管理。
- 规则页基础展示。
- 捕获、MITM、重写、设备、更多设置作为功能入口和后续承载页。

### 7.2 MVP 不包含

MVP 不承诺：

- 完整 TUN / Network Extension。
- Surge 级 HTTP 捕获。
- HTTPS MITM 可用链路。
- 重写脚本运行时。
- 网关模式 / DHCP 接管。
- 多平台版本。
- 正式签名与公证。
- App Store 发布。

## 8. 信息架构

主导航保持当前结构：

| 一级分组 | 页面 | MVP 状态 | 长期目标 |
| --- | --- | --- | --- |
| 状态 | 活动 | 真实基础状态 | Surge 式实时仪表盘 |
| 状态 | 概览 | 配置卡片 | 网络接管总控 |
| 客户端 | 进程 | 进程连接统计 | 进程级流量、阻断、规则归因 |
| 客户端 | 设备 | 空状态/入口 | 网关设备、DHCP、局域网代理 |
| 代理 | 策略 | 节点和策略配置 | 策略组、延迟测试、自动选择 |
| 代理 | 规则 | 规则展示 | 规则编辑、命中计数、测试 |
| HTTP | 捕获 | 入口/列表框架 | HTTP 请求捕获与筛选 |
| HTTP | 解密 | 入口/证书页 | CA、MITM 主机名、QUIC 处理 |
| HTTP | 重写 | 入口/规则卡片 | URL/Header/Body/Mock/脚本 |
| 系统 | 更多 | 设置入口 | DNS、模块、配置、授权更新 |

## 9. 功能需求

### 9.1 App 启动与运行状态

需求编号：APP-001  
优先级：P0

用户打开 `Helio.app` 后，应用应加载主窗口并自动准备本地代理运行环境。

验收标准：

- App 可以从 `.app` 启动。
- 主窗口显示当前前端 UI。
- App bundle 内包含 `sing-box` sidecar。
- App 启动时不会因为旧配置格式导致 `sing-box` 崩溃。
- 如果没有有效配置，自动写入默认可运行配置。

### 9.2 sing-box sidecar 启动/停止

需求编号：CORE-001  
优先级：P0

Helio 需要通过 Tauri 后端启动和停止内置 `sing-box` sidecar。

验收标准：

- 默认配置可通过 `sing-box check`。
- `start_engine` 可以无配置参数启动当前配置。
- `stop_engine` 可以停止当前 sidecar。
- 重启时不会遗留多个 sidecar 进程。
- 启动失败时前端展示失败原因。

### 9.3 系统代理开关

需求编号：CORE-002  
优先级：P0

用户点击活动页或概览页的“系统代理”后，Helio 应切换 macOS 当前默认网络服务的 HTTP、HTTPS、SOCKS 代理。

验收标准：

- 不写死 `Wi-Fi`，优先识别默认路由对应的网络服务。
- 开启时设置 `127.0.0.1:6152`。
- 关闭时清除 HTTP、HTTPS、SOCKS 代理状态。
- 开关状态来自系统代理真实读取结果。
- 失败时前端回滚 UI 状态并提示。

### 9.4 订阅导入

需求编号：SUB-001  
优先级：P0

用户可以在代理配置页导入订阅链接或直接粘贴 URI/base64 节点内容。

MVP 支持：

- `vless://`
- `vmess://`
- `trojan://`
- `hysteria2://` / `hy2://`
- `tuic://`
- `anytls://`
- `ss://`
- base64 节点集合

MVP 暂不支持：

- Clash/Mihomo YAML 订阅自动转换。
- 订阅分组规则完整转换。
- 复杂插件和自定义脚本字段。

验收标准：

- 不能把无法解析的节点导入成 `unknown`。
- 无可用节点时返回明确错误。
- YAML 订阅需要明确提示暂不支持。
- 导入成功后生成 sing-box config。
- 导入成功后自动重启内核。
- 导入成功后代理配置页刷新节点列表。

### 9.5 配置生成与校验

需求编号：CONFIG-001  
优先级：P0

Helio 需要将导入的节点生成 sing-box 可运行配置。

验收标准：

- 生成 `mixed` inbound，监听 `127.0.0.1:6152`。
- 生成 `Proxy` selector。
- 生成 `direct` outbound。
- 生成 route final 指向 `Proxy`。
- 生成新版 sniff rule，不使用已移除的 legacy inbound sniff 字段。
- 生成配置可被当前内置 `sing-box check` 通过。

### 9.6 活动页

需求编号：UI-001  
优先级：P0

活动页是 Helio 的主控制台，需要展示当前代理与网络状态。

MVP 内容：

- 信息状态胶囊。
- 系统代理开关。
- 增强模式开关入口。
- 网络名称。
- 当前配置名。
- 出站模式。
- 外部 IP。
- 策略组选择。
- 代理节点选择。
- Internet / DNS / 路由延迟。
- 上传 / 下载速率。
- 活动连接数。
- 进程数。
- 总流量。

验收标准：

- 在 Tauri 环境读取真实数据。
- 没有真实数据时显示明确占位，不假装正常。
- 导入节点后代理下拉使用真实节点列表。
- 页面适配 900x600 以上窗口，不出现关键内容遮挡。

### 9.7 代理配置页

需求编号：UI-002  
优先级：P0

代理配置页只承载配置，不承载实时流量监控。

MVP 内容：

- 出站模式切换入口。
- 节点卡片。
- 策略组卡片。
- 导入订阅按钮。
- 测试全部按钮。

验收标准：

- 节点来自真实 sing-box config。
- `direct`、`block`、`selector` 等系统 outbound 不作为普通节点展示。
- 导入订阅后刷新节点。
- 测试全部按钮不能是空操作。

### 9.8 进程与连接

需求编号：OBS-001  
优先级：P1

Helio 需要展示当前进程连接情况。

MVP 实现：

- 通过系统命令读取进程和连接。
- 展示进程名、连接数、估算流量。

长期实现：

- 接入 `sing-box-for-apple` / Libbox command server。
- 支持真实连接归属、出站、规则、域名、流量统计。

### 9.9 规则页

需求编号：RULE-001  
优先级：P1

规则页展示当前配置中的 route rules。

MVP 内容：

- 规则 ID。
- 类型。
- 值。
- 策略。
- 命中次数占位。
- 搜索过滤。

长期目标：

- 规则编辑。
- 规则排序。
- 规则测试。
- 命中计数。
- 规则集更新。

### 9.10 HTTP 捕获

需求编号：HTTP-001  
优先级：P2

HTTP 捕获页用于查看 HTTP/HTTPS 请求元数据。

MVP 状态：

- 页面和表格结构保留。
- 真实捕获能力不作为 MVP 承诺。

长期目标：

- 最近请求列表。
- 活动连接。
- DNS 请求。
- 进程筛选。
- 主机名筛选。
- 导出请求日志。
- 与 MITM 配合展示 HTTPS 内容。

### 9.11 HTTPS 解密

需求编号：MITM-001  
优先级：P2

HTTPS 解密页用于 CA 证书和 MITM 主机名管理。

MVP 状态：

- 页面入口和配置结构保留。
- 真实证书安装、信任和解密链路不作为 MVP 承诺。

长期目标：

- 生成 CA 证书。
- 安装到系统钥匙串。
- 引导用户信任证书。
- MITM 主机名白名单。
- QUIC 自动降级。
- HTTP/2 MITM。

### 9.12 重写与映射

需求编号：REWRITE-001  
优先级：P2

重写页用于 URL、Header、Body、Mock 等规则管理。

MVP 状态：

- 卡片入口保留。
- 不执行真实重写规则。

长期目标：

- URL 重定向。
- Header 重写。
- Body 重写。
- Mock 响应。
- 脚本扩展。
- 模块化配置。

### 9.13 设备与网关模式

需求编号：DEVICE-001  
优先级：P2

设备页用于局域网代理、网关模式、DHCP 设备和设备流量观测。

MVP 状态：

- 空设备状态。
- 网关模式入口。

长期目标：

- 局域网 HTTP/SOCKS5 代理。
- 网关模式。
- DHCP 设备列表。
- 设备级规则和流量。

### 9.14 设置与更新

需求编号：SETTINGS-001  
优先级：P1

更多页用于管理通用、外观、DNS、模块、配置、授权与更新。

MVP 内容：

- 设置入口展示。
- DNS、模块、配置等作为后续入口。

长期目标：

- DNS 配置。
- 远程订阅自动更新。
- 配置导入导出。
- App 自动更新。
- 许可证和开源声明。

## 10. 非功能需求

### 10.1 性能

- App 首屏加载时间 <= 2 秒。
- 活动页数据轮询默认不低于 3 秒间隔。
- 大量连接情况下 UI 不阻塞。
- 日志和请求列表需要分页或窗口化。

### 10.2 稳定性

- `sing-box` 崩溃时 UI 显示错误。
- 重启内核前先停止旧进程。
- 系统代理开启失败时回滚。
- App 退出时停止 sidecar 或明确处理后台运行策略。

### 10.3 安全

- 不记录订阅链接中的敏感凭证到普通日志。
- 不默认开启 MITM。
- 安装 CA 证书前必须提示用户。
- 修改系统代理前需要明确状态反馈。
- 后续接入 Network Extension 时必须遵守 Apple 权限要求。

### 10.4 可维护性

- 前端页面按 `src/pages` 拆分。
- 共享 UI 放在 `src/components`。
- Tauri 命令按 `network/process/proxy/singbox/subscription/traffic` 拆分。
- 订阅解析必须有测试。
- 与平台权限相关逻辑必须独立封装。

## 11. 技术方案

### 11.1 当前阶段架构

```text
React 前端
  |
  | Tauri invoke
  v
Rust/Tauri 适配层
  |
  | sidecar / system commands
  v
sing-box binary + macOS networksetup/lsof/netstat
```

当前阶段用于保留前端和快速形成可运行 MVP。

### 11.2 中期目标架构

```text
React 前端
  |
  | Tauri invoke / native bridge
  v
Helio Native Adapter
  |
  | reuse concepts/code from sing-box-for-apple
  v
Network Extension / Libbox / Profile Manager / Command Server
  |
  v
sing-box core
```

中期目标是减少手写系统层，复用 `sing-box-for-apple` 的成熟 Apple 平台能力。

### 11.3 技术取舍

| 方案 | 优点 | 缺点 | 结论 |
| --- | --- | --- | --- |
| 继续纯 Tauri sidecar | 保留前端最快，工程成本低 | TUN、权限、连接观测能力弱 | 适合作为 MVP |
| 直接 fork sing-box-for-apple 改 SwiftUI | 平台能力完整 | 当前 React 前端需重写 | 不符合“保留前端” |
| 保留 React 前端 + 接入 sing-box-for-apple 能力 | UI 保留，平台能力逐步增强 | 桥接复杂，需要设计 native adapter | 推荐中期路线 |

## 12. 数据模型

### 12.1 Profile

```text
id
name
type: local | remote
remoteURL
content
lastUpdatedAt
autoUpdate
```

### 12.2 ProxyNode

```text
id
tag
type
server
serverPort
transport
tls
latency
status
raw
```

### 12.3 PolicyGroup

```text
id
name
type: selector | urltest | fallback
members
selected
latencyStrategy
```

### 12.4 Rule

```text
id
type
value
outbound
hits
source
enabled
```

### 12.5 TrafficSnapshot

```text
timestamp
uploadKbps
downloadKbps
totalUploadMb
totalDownloadMb
connectionCount
processCount
history
```

### 12.6 Connection

```text
id
timestamp
process
status
rule
outbound
upload
download
duration
method
remote
```

## 13. 用户流程

### 13.1 首次启动

1. 用户打开 Helio。
2. App 初始化默认配置。
3. App 启动 `sing-box` sidecar。
4. 活动页显示本机网络状态。
5. 用户进入代理配置页导入订阅。
6. 导入成功后自动重启内核。
7. 用户打开系统代理。
8. 活动页显示代理状态。

### 13.2 导入订阅

1. 用户点击“导入订阅”。
2. 输入订阅 URL 或 URI/base64 节点内容。
3. 后端拉取或解析内容。
4. 如果是 YAML，提示暂不支持。
5. 如果解析不到节点，提示失败。
6. 如果解析成功，生成 sing-box 配置。
7. 校验配置并写入本地。
8. 重启内核。
9. 刷新节点列表。

### 13.3 开启系统代理

1. 用户点击“系统代理”。
2. App 确保内核已启动。
3. 后端识别默认网络服务。
4. 设置 HTTP/HTTPS/SOCKS 代理。
5. 读取系统状态。
6. 前端更新开关。

## 14. 权限与发布

### 14.1 MVP

- 使用 sidecar 和 `networksetup`。
- App 为 ad-hoc 签名。
- 本地开发和手动安装。

### 14.2 正式发布

需要补齐：

- Apple Developer ID。
- 代码签名。
- DMG 签名。
- Notarization 公证。
- Network Extension 权限。
- System Extension 或 Helper 权限。
- 自动更新签名。

## 15. 开源与许可证

Helio 计划基于以下开源项目：

- `sing-box`
- `sing-box-for-apple`

注意：

- `sing-box-for-apple` 使用 GPL-3.0。
- 如果复用其代码，Helio 需要保持 GPL-3.0 兼容开源策略。
- 产品名称、图标和品牌需要保持独立。
- About / License 页面必须展示依赖项目和许可证。

## 16. 风险

| 风险 | 等级 | 说明 | 应对 |
| --- | --- | --- | --- |
| Network Extension 接入复杂 | 高 | Apple 权限、签名、公证门槛高 | MVP 先 sidecar，中期接入 |
| 订阅格式复杂 | 高 | Clash YAML、sing-box JSON、URI 差异大 | 分阶段支持，失败明确提示 |
| 系统代理残留 | 高 | 关闭失败会影响用户网络 | 强制状态读取和回滚 |
| UI 先于能力 | 中 | 用户看到入口但不可用 | 不可用功能标注阶段状态 |
| GPL 约束 | 中 | 复用 sing-box-for-apple 影响许可证 | 明确开源路线 |
| 真实连接统计不准 | 中 | 系统命令解析有限 | 后续接 Libbox command server |

## 17. 里程碑

### M0：当前原型整理

- 保留前端。
- 完成页面拆分。
- 清理假按钮。
- 完成开发情况文档和 PRD。

### M1：可用 sidecar MVP

- 默认配置可启动。
- 系统代理可开关。
- URI/base64 订阅可导入。
- 节点列表真实展示。
- 活动页基础数据真实展示。
- Mac App / DMG 可构建。

### M2：配置和订阅产品化

- Profile 管理。
- 远程订阅更新。
- Clash/Mihomo YAML 转换。
- sing-box JSON 导入。
- 配置校验和错误定位。

### M3：接入 sing-box-for-apple 平台能力

- Network Extension。
- TUN / 增强模式。
- Libbox command server。
- 真实连接和日志。
- 更稳定的进程/设备归因。

### M4：Surge 级高级功能

- HTTP 捕获。
- HTTPS MITM。
- 重写与脚本。
- 规则命中统计。
- 网关模式。
- 模块系统。

### M5：正式发布

- 代码签名。
- 公证。
- GitHub Release。
- 自动更新。
- 完整许可证页面。

## 18. 验收清单

### MVP 必须通过

- [ ] `npm run build` 通过。
- [ ] `cargo test` 通过。
- [ ] `npx tauri build` 通过。
- [ ] 默认 sing-box config 通过 `sing-box check`。
- [ ] `Helio.app` 可以打开。
- [ ] 导入 URI/base64 节点后显示真实节点。
- [ ] 打开系统代理后 macOS 代理状态真实开启。
- [ ] 关闭系统代理后 macOS 代理状态真实关闭。
- [ ] 无节点、导入失败、启动失败都有明确错误。
- [ ] 捕获、MITM、重写等未完成能力不伪装成已可用。

### 后续增强验收

- [ ] Profile 列表可新增、编辑、删除。
- [ ] 远程订阅可更新。
- [ ] TUN 可启停。
- [ ] 活动连接来自真实内核数据。
- [ ] 规则命中数真实更新。
- [ ] HTTP 捕获可筛选和导出。
- [ ] MITM 证书流程完整。

## 19. 当前完成度判断

当前 Helio 处于“可构建 macOS App + 保留前端 + sidecar 代理链路初步可用化”的阶段。

已经具备：

- Surge/Material 风格前端。
- Tauri macOS 壳。
- 内置 `sing-box` sidecar。
- 基础订阅 URI 解析。
- 基础 sing-box 配置生成。
- 系统代理设置命令修正。
- Mac App / DMG 构建。

尚未具备：

- 完整 Network Extension / TUN。
- 完整订阅生态转换。
- 真实 Surge 级连接观测。
- HTTP 捕获与 MITM。
- 重写脚本运行时。
- 正式签名、公证、更新。

一句话结论：

Helio 的前端可以保留，接下来不应继续把核心网络能力手写到底，而应以当前 Tauri sidecar 作为 MVP 过渡，并逐步接入 `sing-box-for-apple` 的 Apple 平台能力，最终形成真正可用的 macOS 代理/VPN 可视化客户端。

# Helio 开发情况文档

更新时间：2026-06-09  
项目路径：`/Users/karl/apps/vpn/surge-material-prototype`  
GitHub 仓库：`https://github.com/kam74515-boop/helio-macos-vpn`

## 1. 项目定位

Helio 是一个面向 macOS 的开源代理与 VPN 客户端原型。产品目标是以 Surge 的信息架构为参照，覆盖活动监控、代理配置、规则、HTTP 捕获、HTTPS 解密、重写映射、进程/设备维度分析等能力，同时使用更扁平的 Google / Material 视觉语言和 macOS 原生窗口骨架。

当前阶段重点是完成可运行的桌面应用原型与主要页面信息结构，不是完整商业级网络内核产品。协议和内核方向以 `sing-box` 侧车为基础，后续可继续扩展全协议配置、订阅解析、真实连接监控与系统代理/TUN 能力。

## 2. 当前交付物

- Web 原型：React + Vite，可通过 `npm run dev` 在 `localhost` 本机预览。
- macOS 桌面壳：Tauri 2，产品名 `Helio`，bundle id 为 `com.kam74515.helio`。
- 本地 macOS 构建产物：
  - `.app`：`src-tauri/target/release/bundle/macos/Helio.app`
  - `.dmg`：`src-tauri/target/release/bundle/dmg/Helio_0.1.0_aarch64.dmg`
- 已初始化 Git 仓库并推送过初始版本到 GitHub。
- 当前本地工作区存在后续开发改动，尚未整体提交到远程。

## 3. 技术栈

- 前端：React 19、Vite 6、MUI Icons。
- 样式：单文件 CSS，重点控制 macOS 工具型界面的密度、栅格、卡片、表格和响应式窗口适配。
- 桌面端：Tauri 2、Rust 2021。
- 内核侧车：`sing-box-aarch64-apple-darwin`，通过 Tauri shell sidecar 启动。
- macOS 系统能力：使用 `networksetup`、`lsof`、`netstat`、`ping`、`dig`、`ipconfig` 等系统命令读取或设置部分网络状态。

## 4. UI 与页面完成情况

已实现的主导航页面：

| 页面 | 当前状态 | 说明 |
| --- | --- | --- |
| 活动 | 已完成高保真原型 | 展示状态胶囊、网络摘要、策略组/代理选择、延迟、上传下载、活动连接、流量统计和总计。 |
| 概览 | 已完成原型 | 系统代理、增强模式、局域网代理、网关模式、远程连接等配置卡片。 |
| 进程 | 已完成原型 + 部分动态 | 左侧进程列表，右侧进程详情面板。 |
| 设备 | 已完成原型 | 空设备状态、网关模式、设备接入步骤面板。 |
| 代理配置 | 已完成原型 | 出站模式、节点配置、策略组配置。代理相关内容只做配置，不承载实时监控。 |
| 规则 | 已完成原型 | 类 Surge 表格，支持搜索过滤。 |
| 捕获 | 已完成原型 | 请求列表、客户端筛选、捕获启动/停止按钮。 |
| HTTPS 解密 | 已完成原型 | CA 证书、MitM 主机名、相关选项。 |
| 重写 & 映射 | 已完成原型 | URL 重定向、Header 重写、Mock、Body 重写。 |
| 更多/设置 | 已完成原型 | 通用、外观、DNS、模块、配置、授权更新、脚本入口。 |

本轮 UI 修正重点：

- 隐藏 Tauri 环境下重复的 Web 假窗口红黄绿按钮，避免和 macOS 原生窗口控件叠加。
- 收紧侧边栏、标题、状态胶囊、卡片、表格、按钮和图表密度，避免页面像展示稿。
- 活动页在桌面宽度下保持仪表盘栅格，不再过早拆成单列。
- 修复流量卡、规则表和捕获表格在小窗口下的裁切/横向溢出。
- 本机预览只使用 `localhost`，不再按局域网 host 暴露开发服务。

## 5. Tauri / 原生能力进展

Tauri 后端目前已从最初的壳层扩展到一批原型级系统能力：

- `start_engine` / `stop_engine`：启动或停止 `sing-box` 侧车进程。
- `set_system_proxy`：通过 `networksetup` 切换 Wi-Fi 的 HTTP、HTTPS、SOCKS 代理。
- `get_system_snapshot`：聚合进程、连接、流量、延迟、网络和代理状态。
- `get_processes`：基于 `lsof` / `ps` / `sysinfo` 生成进程列表。
- `get_connections`：读取活动连接并映射到进程、地址、协议和流量字段。
- `get_traffic_stats`：读取网络接口统计并维护流量历史。
- `get_latency`：通过 ping / dig 估算 Internet、DNS、路由延迟。
- `get_proxy_state`：读取系统代理状态。
- `get_network_info`：读取当前网络、IP、网关和外部 IP。
- `get_singbox_config` / `update_singbox_config`：读取、解析和更新 sing-box 配置。
- `run_speed_test` / `run_speed_test_all`：节点测速占位接口。
- `start_monitoring` / `stop_monitoring`：周期性推送流量更新事件。

这些能力已经为后续真实数据接入奠定基础，但仍需继续做错误处理、权限提示、Network Extension/TUN 集成、订阅格式解析和 UI 数据绑定。

## 6. 构建与验证

已验证命令：

```bash
npm run build
npx tauri build
```

当前产物状态：

- `Helio.app` 已生成，主二进制约 11 MB。
- `Helio_0.1.0_aarch64.dmg` 已生成，约 20 MB。
- 目标架构为 Apple Silicon / arm64。

浏览器视觉 QA：

- 通过应用内浏览器检查过主页面。
- 重点窗口尺寸包括默认桌面窗口与较小窗口。
- 检查项包括：无页面级横向溢出、无不可滚动裁切、活动页状态条不换行、侧边栏布局稳定、主页面内容可访问。

已有 QA 记录：

- `design-qa.md`
- `qa-browser-results.json`
- `qa-policy-page.png`

## 7. Git 状态

远程仓库：

```text
origin https://github.com/kam74515-boop/helio-macos-vpn.git
```

已推送的初始提交：

```text
d3e4db5 Initial Helio macOS VPN prototype
```

当前本地存在未提交改动，主要涉及：

- UI 密度与窗口适配进一步调整。
- README、AGENTS、design QA 文案中文化。
- Tauri 后端能力扩展。
- Tauri 窗口尺寸调整。

在继续推送前，建议先做一次完整 review 和构建验证，再将这批改动作为第二个提交推送。

## 8. 当前风险与未完成项

- 网络内核仍处于原型接入阶段：`sing-box` 已可作为 sidecar 启动，但配置管理、订阅导入和真实路由策略尚未产品化。
- 系统代理当前默认操作 Wi-Fi 服务，尚未做网络服务自动识别、权限提示和失败回滚。
- 进程、连接、流量统计依赖系统命令解析，需继续强化兼容性、性能和异常处理。
- HTTP 捕获、MitM、重写与脚本能力目前主要是 UI 和配置入口，尚未具备 Surge 级完整功能。
- 代码体量已经开始增大：`src/App.jsx`、`src/styles.css`、`src-tauri/src/lib.rs` 后续需要拆分模块。
- macOS 正式分发还缺少代码签名、公证、更新机制和发布流程。

## 9. 推荐下一步

1. 提交当前未提交改动，形成第二个 GitHub 版本。
2. 将前端拆分为页面组件、数据 hooks 和共享组件，降低 `App.jsx` 复杂度。
3. 将 Rust 后端拆分为 `network`、`process`、`traffic`、`singbox`、`proxy` 等模块。
4. 做真实数据绑定：活动页、进程页、网络摘要和系统代理状态优先接入 Tauri 命令。
5. 增加订阅导入与 sing-box 配置生成流程，覆盖 VLESS、Hysteria2、TUIC、VMess、AnyTLS 等协议。
6. 为 macOS 系统代理/TUN/权限操作增加明确的用户确认和失败恢复。
7. 建立发布流程：代码签名、公证、GitHub Release、DMG 上传和版本更新。

## 10. 一句话总结

Helio 当前已经完成 Surge 信息结构 + Material 扁平化风格 + Tauri macOS 壳层的可运行原型，并产出了本地 `.app/.dmg`。下一阶段的核心不是继续堆 UI，而是把当前原型拆分工程结构、接入真实内核数据，并补齐 macOS 代理/VPN 产品所需的权限、配置、订阅和发布链路。

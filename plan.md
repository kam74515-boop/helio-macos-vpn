# Helio 多问题修复计划

## 问题清单

1. **进程获取不全面** — 当前 process.rs 已改用 sysinfo 获取所有进程，但：
   - `process.name()` 返回截断名（15字符），导致应用识别和图标匹配失败
   - `get_app_icon_base64` 对每个进程单独调用 `lsof -p`，100个进程=100次外部命令，性能极差
   - 系统进程过滤列表可能不够完善

2. **活动页数据显示有问题** — ActivityPage.jsx：
   - `displayProcs` 使用 `p.icon_key`，但 Tauri 序列化后字段名可能是 camelCase (`iconKey`)
   - 未使用 `p.icon_base64`（或 `iconBase64`）显示真实图标
   - `get_system_snapshot` 返回的数据可能不准确（traffic.rs 使用 `ipkts * 1500` 估算）

3. **无法导入订阅** — 需要验证 subscription.rs 的完整导入链路：
   - `looks_like_subscription_body` 对 URL 的判断
   - HTTP GET 请求和 base64 解码
   - `parse_proxy_uri` 对各种协议的支持
   - 前端调用方式

4. **前端顶部白条** — tauri.conf.json `titleBarStyle: "Transparent"` + CSS `.is-tauri .sidebar { padding-top: 64px; }` 在隐藏 traffic-lights 后造成顶部空白

5. **窗口缩放后UI不正常** — CSS 中多处使用固定高度计算如 `height: calc(100vh - 14px * 2 - 20px - 210px)`，窗口缩小时布局崩坏

## 任务分配

### Stage 1 — 并行修复（Agent 1 + Agent 2 同时执行）

**Agent 1: Rust后端修复专家**
- 文件范围：`src-tauri/src/commands/process.rs`, `src-tauri/src/utils.rs`, `src-tauri/src/commands/subscription.rs`, `src-tauri/src/commands/network.rs`
- 任务：
  1. 优化进程获取：使用 `sysinfo` 的 `exe()` 获取真实路径，批量调用 `lsof` 获取所有进程路径（一次命令），避免每个进程单独调用
  2. 修复 `get_app_icon_base64` 性能：批量获取路径，或改用 `sysinfo::Process::exe()`
  3. 检查 `guess_icon` 对截断名称的匹配（如 "Google Chrome" 截断为 "Google" 或 "Chrome"）
  4. 检查 subscription.rs 导入逻辑，确保 URL 订阅能正确下载、解码、解析
  5. 检查 `get_system_snapshot` 数据准确性

**Agent 2: 前端UI修复专家**
- 文件范围：`src/styles.css`, `src/pages/ActivityPage.jsx`, `src/pages/ProcessesPage.jsx`, `src-tauri/tauri.conf.json`
- 任务：
  1. 修复顶部白条：调整 `.is-tauri .sidebar` 的 padding-top，或调整 titleBarStyle
  2. 修复窗口缩放响应式：将固定 height: calc(...) 改为 minmax/auto/flex 布局
  3. 修复 ActivityPage 的 icon 映射：确认 Tauri 序列化字段名（icon_base64 vs iconBase64），统一使用真实图标
  4. 检查 ProcessesPage 的 icon 映射一致性

### Stage 2 — 整合验证
- 主 Agent 合并所有修改
- 执行 `cargo check` + `npm run build` + `npx tauri build`
- 提交最终代码

## 关键上下文

- 项目路径：`/Users/karl/apps/vpn/surge-material-prototype`
- Tauri v2，Rust 1.96.0 刚安装
- 前端：React 19 + Vite 6，单文件 CSS `src/styles.css`
- 窗口配置：`titleBarStyle: "Transparent"`, `hiddenTitle: true`, `transparent: true`
- 进程图标：后端新增 `icon_base64: Option<String>`，通过 `lsof -Fn` + `sips` 提取真实 macOS 图标
- 订阅测试 URL：`https://user.vipservers202611.cc/s/tX80deST`（base64 编码的 vless Reality URI）

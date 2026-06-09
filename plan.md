# Helio MVP 继续开发计划

## 目标
完成 M1 阶段（可用 sidecar MVP）剩余核心功能，使 App 达到"可导入订阅、切换模式、测速、展示真实数据"的可用标准。

## 当前基线
- 仓库：`/Users/karl/apps/vpn/surge-material-prototype`
- 分支：`main`（已提交 94c5955）
- 前端：React 19 + Vite 6，10 个页面组件
- 后端：Tauri 2 + Rust，模块化命令
- 已具备：UI 原型、sidecar 启停、系统代理、订阅导入、基础配置生成

## Stage 1 — Rust 后端增强（并行）

### Worker A: 节点测速 + 代理模式切换
**所有权**：`src-tauri/src/commands/network.rs`、`src-tauri/src/commands/singbox.rs`、`src-tauri/src/lib.rs`

#### 1.1 真实节点测速
- 新增命令 `test_node_latency(node_tag: String, server: String, server_port: u16) -> SpeedTestResult`
- 实现：通过 `std::net::TcpStream::connect_timeout` 测试节点端口连通性
- 超时 3 秒，成功则记录耗时，失败则标记为超时
- 修改 `run_speed_test`：读取当前 config 中的节点，对每个节点调用 `test_node_latency`
- 修改 `run_speed_test_all`：测试所有非系统 outbound 节点

#### 1.2 代理模式切换
- 新增命令 `set_proxy_mode(mode: String) -> Result<(), String>`
- mode 取值：`"direct"` | `"global"` | `"rule"`
- 实现逻辑：
  - `"direct"`：route.final = "direct"，auto_detect_interface = true，移除 selector 默认
  - `"global"`：route.final = "Proxy"，auto_detect_interface = false
  - `"rule"`：route.final = "Proxy"，auto_detect_interface = true
- 修改配置后自动写入 config.json 并重启 sidecar
- 新增命令 `get_proxy_mode() -> String`

### Worker B: 配置信息完善
**所有权**：`src-tauri/src/commands/singbox.rs`、`src-tauri/src/types.rs`

#### 1.3 配置元数据
- `SingboxConfig` 新增字段：`config_name: String`（从文件路径或默认"Default"）
- `get_singbox_config` 返回真实配置名和当前模式

## Stage 2 — 前端增强（并行）

### Worker C: 错误处理 + Toast 系统
**所有权**：`src/components/ui.jsx`、`src/App.jsx`、各页面文件

#### 2.1 Toast 通知系统
- 新增 `ToastProvider` 和 `useToast` hook
- 支持 success / error / info 三种类型
- 自动 3 秒后消失
- 替换所有 `alert()` 调用

#### 2.2 活动页配置名真实显示
- 从 `get_singbox_config` 读取 `config_name` 和 `mode`
- 替换写死的 "Default" 和 "全局代理"

#### 2.3 代理配置页模式切换联动
- 模式切换调用 `set_proxy_mode`
- 切换后刷新配置并显示 toast

### Worker D: 规则页 + 进程页完善
**所有权**：`src/pages/RulesPage.jsx`、`src/pages/ProcessesPage.jsx`

#### 2.4 规则页
- 已接入真实数据，保持现状
- 命中计数显示 "-"（MVP 占位）

#### 2.5 进程页
- 已接入真实数据，保持现状
- 详情面板使用真实进程数据填充

## Stage 3 — 整合与验证

### 主代理执行
1. 合并所有 worker 改动
2. 运行 `cargo test`（订阅解析测试）
3. 运行 `npm run build`
4. 运行 `npx tauri build`
5. Git 提交

## 接口契约

### 新增 Tauri 命令
```rust
// 节点测速
test_node_latency(node_tag: String, server: String, server_port: u16) -> SpeedTestResult

// 代理模式
set_proxy_mode(mode: String) -> Result<(), String>
get_proxy_mode() -> String
```

### 修改的数据结构
```rust
struct SingboxConfig {
    mode: String,           // "direct" | "global" | "rule"
    config_name: String,    // "Default" 或自定义名
    outbounds: Vec<SingboxOutbound>,
    rules: Vec<SingboxRule>,
    policy_groups: Vec<serde_json::Value>,
}
```

## 验证清单
- [ ] `cargo test` 通过（订阅解析测试）
- [ ] `npm run build` 通过
- [ ] `npx tauri build` 通过
- [ ] 导入订阅后节点列表真实展示
- [ ] 节点测速返回真实延迟
- [ ] 模式切换后配置正确写入
- [ ] 活动页显示真实配置名和模式
- [ ] Toast 替代所有 alert

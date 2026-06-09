# M4+M5 开发计划

> 目标：推进 MITM CA 管理、主机名列表、LAN 设备发现。对于不可实现的功能保持明确占位状态。

## 当前状态

- MitmPage 前端已调用 `get_ca_status`、`generate_ca`、`install_ca`，但后端命令不存在
- DevicesPage 前端已调用 `get_devices`，但后端命令可能不存在或只是占位
- Cargo.toml 没有 rcgen/openssl 依赖
- Profile 中没有 mitm.hostname_list 的持久化前端交互

## 可行范围

### M4 可行

1. **CA 证书生成** — 使用 `rcgen` 库（纯 Rust，轻量）生成自签名 CA
2. **CA 证书导出** — 导出为 PEM 格式到用户指定路径
3. **证书状态检测** — 检查证书文件是否存在、是否过期
4. **MITM 主机名列表 CRUD** — 存储在 Profile 中，前端支持增删改

### M4 不可行（保持占位）

- 系统 CA 信任（需要管理员权限 + 安全钥匙串操作，风险高）
- CaptureService（需要额外 HTTP 代理实现，超出 sing-box 能力）
- Rewrite engine（需要请求拦截修改）
- Mock response（同上）

### M5 可行

1. **LAN 设备发现** — 通过 `arp -a` 获取局域网设备（IP、MAC、名称）

### M5 不可行（保持占位）

- Gateway/DHCP（需要管理员权限配置 DHCP 服务器）
- Developer ID 签名/公证（需要 Apple Developer 账号）
- 自动更新（需要签名 + 更新服务器）

## 阶段划分

### Stage 1 — CA 管理 + MITM 主机名（并行）

**Worker A — CA 管理工程师**
- 后端：`generate_ca`、`get_ca_status`、`export_ca` 命令
- 前端：MitmPage 证书状态真实显示、生成/导出按钮真实功能
- 系统信任按钮保持占位但明确提示原因

**Worker B — MITM 主机名 + LAN 设备工程师**
- 后端：`add_mitm_hostname`、`remove_mitm_hostname`、`get_mitm_hostnames`、`get_lan_devices`
- 前端：MitmPage 主机名列表增删改、DevicesPage 设备列表真实显示

### Stage 2 — 整合构建

- `cargo check` + `npm run build` + `npx tauri build`

## 验收标准

- 所有按钮有真实反馈或明确不可用原因
- 证书生成/导出/状态检测可用
- 主机名列表可增删改
- LAN 设备可从 ARP 表显示
- 不可行功能明确标注原因

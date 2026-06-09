# Helio

Helio 是一个 macOS 代理与 VPN 仪表盘原型，借鉴 Surge 的信息架构，采用更扁平的 Material 风格视觉语言。

## 当前范围

- 类似 Surge 的活动、概览、进程、设备、代理配置、规则、捕获、MITM、重写和设置页面。
- 基于 `localhost` 的本地 Vite 开发服务器。
- Tauri macOS 壳层，内嵌 `sing-box` 侧车程序占位。

## 开发

```bash
npm install
npm run dev
```

## 构建

```bash
npm run build
npx tauri build
```

构建产物生成在 `src-tauri/target/release/bundle/` 目录下。

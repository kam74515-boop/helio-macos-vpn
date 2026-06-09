import { Icon } from "../components/ui";
import { useToast } from "../components/Toast";

export function MorePage() {
  const { addToast } = useToast();

  const settings = [
    ["toggle_on", "通用", "可以在这里找到大部分基础配置。", () => addToast("通用设置功能开发中", "info")],
    ["palette", "外观", "菜单栏图标、Dock 图标和通知相关设置。", () => addToast("外观设置功能开发中", "info")],
    ["explore", "DNS", "本地 DNS 映射和 DNS 相关设置。", () => addToast("DNS 设置功能开发中，请前往配置页面", "info")],
    ["deployed_code", "模块", "模块是一系列设置的集合，可用于覆盖当前配置的部分设定。", () => addToast("模块功能开发中", "info")],
    ["description", "配置", "管理所有配置式、订阅和远程配置。", () => addToast("配置管理功能开发中", "info")],
    ["sync", "授权 & 更新", "管理授权、更新和开源版本信息。", () => addToast("授权与更新功能开发中", "info")],
    ["experiment", "脚本", "使用 JavaScript 扩展 HTTP 与规则处理能力。", () => addToast("脚本功能开发中", "info")],
  ];

  return (
    <div className="page more-page">
      <h1>设置</h1>
      <div className="settings-grid">
        {settings.map(([icon, title, text, onClick]) => (
          <button className="setting-tile" key={title} onClick={onClick}>
            <span className="setting-icon"><Icon name={icon} /></span>
            <strong>{title}</strong>
            <p>{text}</p>
          </button>
        ))}
      </div>
    </div>
  );
}

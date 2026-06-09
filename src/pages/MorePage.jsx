import { Icon } from "../components/ui";

export function MorePage() {
  const settings = [
    ["toggle_on", "通用", "可以在这里找到大部分基础配置。"],
    ["palette", "外观", "菜单栏图标、Dock 图标和通知相关设置。"],
    ["explore", "DNS", "本地 DNS 映射和 DNS 相关设置。"],
    ["deployed_code", "模块", "模块是一系列设置的集合，可用于覆盖当前配置的部分设定。"],
    ["description", "配置", "管理所有配置式、订阅和远程配置。"],
    ["sync", "授权 & 更新", "管理授权、更新和开源版本信息。"],
    ["experiment", "脚本", "使用 JavaScript 扩展 HTTP 与规则处理能力。"],
  ];
  return (
    <div className="page more-page">
      <h1>设置</h1>
      <div className="settings-grid">
        {settings.map(([icon, title, text]) => (
          <button className="setting-tile" key={title}>
            <span className="setting-icon"><Icon name={icon} /></span>
            <strong>{title}</strong>
            <p>{text}</p>
          </button>
        ))}
      </div>
    </div>
  );
}

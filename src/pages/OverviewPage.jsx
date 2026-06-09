import { OverviewCard } from "../components/ui";

export function OverviewPage({ systemProxy, enhanced, setSystemProxy, setEnhanced }) {
  const cards = [
    ["系统代理", "大多数应用的流量可以通过将 Helio 设置为系统代理接管，具有最佳的兼容性和性能。", "Helio 当前被设置为系统代理", systemProxy, setSystemProxy, "green"],
    ["增强模式", "部分应用可能不遵循系统代理设置。使用增强模式可以让所有应用由 Helio 处理。", "增强模式已激活", enhanced, setEnhanced, "green"],
    ["HTTP & SOCKS5 代理", "Helio 可以被其他设备用作标准的 HTTP 和 SOCKS5 代理服务器。", "已禁用", false, () => {}, "orange"],
    ["网关模式", "将 Helio 用作局域网的 DHCP 服务器，并使用网关模式接管其他设备的网络。", "未配置", false, () => {}, "orange"],
    ["Helio Ponte", "在运行 macOS 和 iOS 的设备之间创建私有代理网络。", "已禁用", false, () => {}, "orange"],
  ];
  return (
    <div className="page airy overview-page">
      <h1>概览</h1>
      <h2 className="section-title magenta">网络接管</h2>
      <div className="overview-grid">
        {cards.slice(0, 2).map(([title, text, status, checked, setChecked, color]) => (
          <OverviewCard key={title} title={title} text={text} status={status} checked={checked} onChange={setChecked} color={color} />
        ))}
      </div>
      <h2 className="section-title indigo">局域网设备接管</h2>
      <div className="overview-grid">
        {cards.slice(2, 4).map(([title, text, status, checked, setChecked, color]) => (
          <OverviewCard key={title} title={title} text={text} status={status} checked={checked} onChange={setChecked} color={color} />
        ))}
      </div>
      <h2 className="section-title blue">远程连接</h2>
      <div className="overview-grid single">
        <OverviewCard title={cards[4][0]} text={cards[4][1]} status={cards[4][2]} checked={false} onChange={() => {}} color="orange" />
      </div>
    </div>
  );
}

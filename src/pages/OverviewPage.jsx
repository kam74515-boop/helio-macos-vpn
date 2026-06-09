import { useEffect, useState } from "react";
import { OverviewCard } from "../components/ui";
import { canUseTauri, safeInvoke } from "../utils/tauri";
import { useTauriPoll } from "../hooks/tauri";
import { useToast } from "../components/Toast";

export function OverviewPage({ systemProxy, enhanced, setSystemProxy, setEnhanced }) {
  const { addToast } = useToast();
  const [proxyConfig, setProxyConfig] = useState(null);
  const [tunBusy, setTunBusy] = useState(false);
  const [permStatus, setPermStatus] = useState(null);

  const { data: tunStatus, loading: tunLoading } = useTauriPoll("get_tun_status", null, 3000);

  useEffect(() => {
    if (!canUseTauri()) return;
    safeInvoke("get_proxy_config").then(setProxyConfig).catch(() => {});
  }, []);

  const httpStatus = proxyConfig?.http_port ? `HTTP: ${proxyConfig.http_listen || "127.0.0.1"}:${proxyConfig.http_port}` : "未配置";
  const socksStatus = proxyConfig?.socks_port ? `SOCKS5: ${proxyConfig.socks_listen || "127.0.0.1"}:${proxyConfig.socks_port}` : "未配置";
  const proxyStatus = proxyConfig ? `${httpStatus} / ${socksStatus}` : "未配置";

  const handleProxyToggle = () => {
    addToast("代理服务器配置请前往设置页面", "info");
  };

  const handleGatewayToggle = () => {
    addToast("网关模式需要管理员权限，功能开发中", "info");
  };

  const handlePonteToggle = () => {
    addToast("远程连接功能开发中", "info");
  };

  const handleEnhancedToggle = async (enable) => {
    if (!canUseTauri()) {
      addToast("Tauri 环境不可用", "error");
      return;
    }
    try {
      setTunBusy(true);
      const status = await safeInvoke("toggle_enhanced_mode", { enable });
      if (status?.active) {
        addToast("增强模式已激活", "success");
      } else if (status?.error) {
        addToast(status.error, "error");
      } else {
        addToast("增强模式已关闭", "info");
      }
      setEnhanced(enable);
    } catch (e) {
      addToast("增强模式切换失败: " + e, "error");
    } finally {
      setTunBusy(false);
    }
  };

  const handlePermissionCheck = async () => {
    if (!canUseTauri()) {
      addToast("Tauri 环境不可用", "error");
      return;
    }
    try {
      const status = await safeInvoke("get_permission_status");
      setPermStatus(status);
      if (status?.error) {
        addToast(status.error, "error");
      } else {
        addToast("权限诊断完成", "success");
      }
    } catch (e) {
      addToast("权限诊断失败: " + e, "error");
    }
  };

  const isTunActive = tunStatus?.active;
  const isTunEnabled = tunStatus?.enabled;
  const enhancedStatus = isTunActive
    ? "增强模式已激活"
    : isTunEnabled
      ? "增强模式启用中，等待接口..."
      : "增强模式未启用";
  const enhancedColor = isTunActive ? "green" : "orange";
  const enhancedChecked = isTunActive || enhanced;

  const cards = [
    ["系统代理", "大多数应用的流量可以通过将 Helio 设置为系统代理接管，具有最佳的兼容性和性能。", "Helio 当前被设置为系统代理", systemProxy, setSystemProxy, "green"],
    ["增强模式", "部分应用可能不遵循系统代理设置。使用增强模式可以让所有应用由 Helio 处理。", enhancedStatus, enhancedChecked, handleEnhancedToggle, enhancedColor],
    ["HTTP & SOCKS5 代理", "Helio 可以被其他设备用作标准的 HTTP 和 SOCKS5 代理服务器。", proxyStatus, false, handleProxyToggle, "orange"],
    ["网关模式", "将 Helio 用作局域网的 DHCP 服务器，并使用网关模式接管其他设备的网络。", "需要管理员权限，功能开发中", false, handleGatewayToggle, "orange"],
    ["Helio Ponte", "在运行 macOS 和 iOS 的设备之间创建私有代理网络。", "远程连接功能开发中", false, handlePonteToggle, "orange"],
  ];

  const permSummary = (() => {
    if (!canUseTauri()) return "Tauri 环境不可用";
    if (tunLoading && !permStatus) return "加载中...";
    if (!permStatus) return "点击检查权限状态";
    const parts = [];
    if (permStatus.is_admin != null) parts.push(`管理员权限: ${permStatus.is_admin ? "是" : "否"}`);
    if (permStatus.has_sudo != null) parts.push(`sudo: ${permStatus.has_sudo ? "是" : "否"}`);
    if (permStatus.has_ne_entitlement != null) parts.push(`NE entitlement: ${permStatus.has_ne_entitlement ? "是" : "否"}`);
    return parts.join(" | ") || "权限状态正常";
  })();

  return (
    <div className="page airy overview-page">
      <h1>概览</h1>
      <h2 className="section-title magenta">网络接管</h2>
      <div className="overview-grid">
        {cards.slice(0, 2).map(([title, text, status, checked, setChecked, color]) => (
          <OverviewCard key={title} title={title} text={text} status={status} checked={checked} onChange={setChecked} color={color} />
        ))}
      </div>
      <div className="permission-diagnostics" style={{ marginTop: "8px", padding: "0 4px" }}>
        <button
          className="soft-button"
          onClick={handlePermissionCheck}
          disabled={tunBusy}
          style={{ fontSize: "12px", opacity: 0.7 }}
        >
          {permSummary}
        </button>
      </div>
      <h2 className="section-title indigo">局域网设备接管</h2>
      <div className="overview-grid">
        {cards.slice(2, 4).map(([title, text, status, checked, setChecked, color]) => (
          <OverviewCard key={title} title={title} text={text} status={status} checked={checked} onChange={setChecked} color={color} />
        ))}
      </div>
      <h2 className="section-title blue">远程连接</h2>
      <div className="overview-grid single">
        <OverviewCard title={cards[4][0]} text={cards[4][1]} status={cards[4][2]} checked={false} onChange={cards[4][4]} color="orange" />
      </div>
    </div>
  );
}

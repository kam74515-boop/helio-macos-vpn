import { useState } from "react";
import { Segmented, Icon } from "../components/ui";
import { useTauriPoll } from "../hooks/tauri";
import { canUseTauri, safeInvoke } from "../utils/tauri";
import { useToast } from "../components/Toast";

export function CapturePage() {
  const [tab, setTab] = useState("最近的请求");
  const [capturing, setCapturing] = useState(false);
  const { data: realConns, loading, refresh } = useTauriPoll("get_connections", null, 3000);
  const { data: clashConnections } = useTauriPoll("get_clash_connections", null, 3000);
  const { data: lanDevices } = useTauriPoll("get_lan_devices", null, 5000);
  const { data: trafficStats } = useTauriPoll("get_traffic_stats", null, 3000);
  const { addToast } = useToast();
  const displayReqs = canUseTauri() && realConns?.length
    ? realConns.map((c, i) => [String(i + 1), c.timestamp, c.process, c.status, c.proxy, c.upload, c.download, c.duration, c.method, c.remote])
    : [];

  const handleClear = async () => {
    if (!canUseTauri()) return;
    try {
      await safeInvoke("clear_connections");
      addToast("连接列表已清空", "success");
    } catch (e) {
      addToast("连接列表已清空", "info");
    }
  };

  const handleRefresh = () => {
    refresh();
    addToast("连接列表已刷新", "info");
  };

  const handleToggleCapture = async () => {
    if (!canUseTauri()) return;
    const next = !capturing;
    try {
      await safeInvoke("toggle_capture", { enabled: next });
      setCapturing(next);
      addToast(next ? "流量捕获已启动" : "流量捕获已停止", "success");
    } catch (e) {
      addToast("流量捕获需要启用系统代理或增强模式", "error");
    }
  };

  const handleMitM = () => {
    addToast("HTTPS 解密需要先配置 CA 证书，请前往解密页面", "info");
  };

  const handleProxyControl = () => {
    addToast("代理控制功能开发中", "info");
  };

  const renderTabContent = () => {
    if (tab === "最近的请求") {
      return (
        <>
          {loading && <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>加载中...</div>}
          {!loading && displayReqs.length === 0 && (
            <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>暂无捕获数据</div>
          )}
          {displayReqs.length > 0 && (
            <div className="request-table">
              <div className="thead"><span>ID</span><span>日期</span><span>客户端</span><span>状态</span><span>策略</span><span>上传</span><span>下载</span><span>时长</span><span>方法</span><span>地址</span></div>
              {displayReqs.map((row) => (
                <div className="tr" key={row[0]}>
                  {row.map((cell, index) => <span key={index}>{index === 0 && <i className="status-dot" />}{cell}</span>)}
                </div>
              ))}
            </div>
          )}
        </>
      );
    }

    if (tab === "活动连接") {
      const items = canUseTauri() && clashConnections?.length ? clashConnections : [];
      return (
        <>
          {items.length === 0 && (
            <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>暂无活动连接</div>
          )}
          {items.length > 0 && (
            <div className="request-table">
              <div className="thead"><span>ID</span><span>目标地址</span><span>代理链</span><span>上传</span><span>下载</span></div>
              {items.map((c, i) => (
                <div className="tr" key={c.id || i}>
                  <span><i className="status-dot" />{c.id || i + 1}</span>
                  <span>{c.destination || c.remote || "—"}</span>
                  <span>{c.proxy_chain || c.proxy || "—"}</span>
                  <span>{c.upload || "—"}</span>
                  <span>{c.download || "—"}</span>
                </div>
              ))}
            </div>
          )}
        </>
      );
    }

    if (tab === "DNS") {
      return (
        <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>
          DNS 查询日志需要 sing-box DNS 日志模块支持，功能开发中。
        </div>
      );
    }

    if (tab === "设备") {
      const devices = canUseTauri() && lanDevices?.length ? lanDevices : [];
      return (
        <>
          {devices.length === 0 && (
            <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>暂无设备数据</div>
          )}
          {devices.length > 0 && (
            <div className="request-table">
              <div className="thead"><span>IP</span><span>MAC</span></div>
              {devices.map((d, i) => (
                <div className="tr" key={i}>
                  <span>{d.ip || "—"}</span>
                  <span>{d.mac || "—"}</span>
                </div>
              ))}
            </div>
          )}
        </>
      );
    }

    if (tab === "流量统计") {
      const stats = canUseTauri() && trafficStats ? trafficStats : null;
      return (
        <>
          {!stats && (
            <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>暂无流量统计数据</div>
          )}
          {stats && (
            <div className="request-table">
              <div className="thead"><span>指标</span><span>数值</span></div>
              <div className="tr"><span>上传</span><span>{stats.upload || "—"}</span></div>
              <div className="tr"><span>下载</span><span>{stats.download || "—"}</span></div>
              <div className="tr"><span>总计</span><span>{stats.total || "—"}</span></div>
            </div>
          )}
        </>
      );
    }

    if (tab === "日志簿") {
      return (
        <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>
          日志簿功能需要 sing-box 日志事件流支持，功能开发中。
        </div>
      );
    }

    return null;
  };

  return (
    <div className="capture-window">
      <div className="capture-sidebar">
        <div className="traffic-lights small" data-tauri-drag-region="true"><span className="red" /><span className="yellow" /><span className="green" /></div>
        <Segmented value="按客户端" options={["按客户端", "按主机名"]} onChange={() => {}} />
        <h3>请求</h3>
        <button className="side-chip active">所有客户端</button>
        <h3>本地程序 <Icon name="settings" /></h3>
        <div style={{ padding: "8px 0", color: "var(--muted)", fontSize: 12 }}>暂无筛选数据</div>
      </div>
      <main className="request-pane">
        <div className="request-tabs">
          <Segmented value={tab} options={["最近的请求", "活动连接", "DNS", "设备", "流量统计", "日志簿"]} onChange={setTab} />
          <label className="search compact-search"><Icon name="search" /><input placeholder="搜索" /></label>
        </div>
        {renderTabContent()}
        <div className="capture-actions">
          <button onClick={handleClear}>清空</button>
          <button onClick={handleRefresh}>重新载入</button>
          <button onClick={handleToggleCapture}>{capturing ? "停止流量捕获" : "启动流量捕获"}</button>
          <button onClick={handleMitM}>开启 MitM</button>
          <button onClick={handleProxyControl}>代理控制</button>
        </div>
      </main>
    </div>
  );
}

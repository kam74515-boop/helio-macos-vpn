import { useEffect, useState, useRef } from "react";
import { StatusPills, MetricCard, MiniLine, MenuSelect, Icon, ProcessRank, Segmented } from "../components/ui";
import { useTauriPoll } from "../hooks/tauri";
import { canUseTauri, safeInvoke } from "../utils/tauri";
import { useToast } from "../components/Toast";

export function ActivityPage({ systemProxy, enhanced, setSystemProxy, setEnhanced, selectedProxy, setSelectedProxy, selectedGroup, setSelectedGroup }) {
  const [scope, setScope] = useState("全部");
  const [trafficTab, setTrafficTab] = useState("进程与设备");
  const { addToast } = useToast();

  // Existing system-level polls
  const { data: snap, loading: snapLoading } = useTauriPoll("get_system_snapshot", null, 3000);
  const { data: realProcs } = useTauriPoll("get_processes", null, 5000);
  const { data: config } = useTauriPoll("get_singbox_config", null, 5000);
  const { data: tunStatus } = useTauriPoll("get_tun_status", null, 3000);

  // New Clash API polls
  const { data: clashProxiesRaw } = useTauriPoll("get_clash_proxies", null, 5000);
  const { data: clashConnectionsRaw } = useTauriPoll("get_clash_connections", null, 3000);
  const { data: clashTraffic } = useTauriPoll("get_clash_traffic", null, 3000);

  // Local traffic history built from clash traffic (down KB/s)
  const [trafficHistory, setTrafficHistory] = useState([]);
  const historyRef = useRef([]);

  useEffect(() => {
    if (clashTraffic && typeof clashTraffic.down === "number") {
      const downKbps = Math.round(clashTraffic.down / 1024);
      const newHistory = [...historyRef.current, downKbps].slice(-24);
      historyRef.current = newHistory;
      setTrafficHistory(newHistory);
    }
  }, [clashTraffic]);

  // Normalize clash proxies to an object keyed by name
  const clashProxies = (() => {
    if (!clashProxiesRaw) return {};
    if (Array.isArray(clashProxiesRaw)) {
      return Object.fromEntries(clashProxiesRaw.map((p) => [p.name || p.tag, p]));
    }
    if (clashProxiesRaw.proxies) return clashProxiesRaw.proxies;
    return clashProxiesRaw;
  })();

  const proxyEntries = Object.values(clashProxies || {});

  // Policy groups: Selector or URLTest
  const policyGroups = proxyEntries.filter((p) => {
    const type = p.proxy_type || p.type;
    return type === "Selector" || type === "URLTest";
  });

  const groupOptions = policyGroups.length
    ? policyGroups.map((g) => g.name || g.tag || "Proxy")
    : canUseTauri() && config
      ? ["Proxy"]
      : [];

  // Node options: non-Selector/URLTest/Direct/Block/Reject
  const nodeOptions = proxyEntries
    .filter((p) => {
      const type = p.proxy_type || p.type;
      return !["Selector", "URLTest", "Direct", "Block", "Reject"].includes(type);
    })
    .map((p) => p.name || p.tag);

  // Active group and its selectable proxies
  const activeGroup = policyGroups.find((g) => (g.name || g.tag || "Proxy") === selectedGroup);

  const proxyOptions = (() => {
    if (activeGroup?.all?.length) return activeGroup.all;
    if (activeGroup?.now) return [activeGroup.now];
    return nodeOptions.length ? nodeOptions : ["direct"];
  })();

  // Sync selected group / proxy when options change
  useEffect(() => {
    if (groupOptions.length && !groupOptions.includes(selectedGroup)) {
      setSelectedGroup(groupOptions[0]);
      return;
    }
    const groupNow = activeGroup?.now;
    if (groupNow && groupNow !== selectedProxy) {
      setSelectedProxy(groupNow);
    } else if (proxyOptions.length && !proxyOptions.includes(selectedProxy)) {
      setSelectedProxy(proxyOptions[0]);
    }
  }, [proxyOptions.join("|"), groupOptions.join("|"), activeGroup?.now, selectedProxy, selectedGroup, setSelectedProxy, setSelectedGroup]);

  const isReal = canUseTauri() && snap;
  const loading = snapLoading;

  // System snapshot fields
  const ssid = isReal ? snap.ssid : "--";
  const externalIp = isReal ? snap.external_ip : "--";
  const latencyMs = isReal ? snap.internet_latency_ms : null;
  const dnsMs = isReal ? snap.dns_latency_ms : null;
  const routerMs = isReal ? snap.router_latency_ms : null;
  const processCount = isReal ? snap.processes_with_connections : 0;
  const deviceCount = isReal ? (snap.devices_total > 0 ? snap.devices_total : "-") : "-";
  const dhcpCount = isReal ? (snap.dhcp_devices_total > 0 ? snap.dhcp_devices_total : "-") : "-";

  // Real connection data from Clash API
  const connectionsData = (() => {
    if (!clashConnectionsRaw) return null;
    if (Array.isArray(clashConnectionsRaw)) return clashConnectionsRaw;
    if (clashConnectionsRaw.connections) return clashConnectionsRaw.connections;
    return [];
  })();

  const connectionsCount = connectionsData != null ? connectionsData.length : isReal ? snap.connections_total : 0;
  const connTotalUp = connectionsData ? connectionsData.reduce((sum, c) => sum + (c.upload || 0), 0) : 0;
  const connTotalDown = connectionsData ? connectionsData.reduce((sum, c) => sum + (c.download || 0), 0) : 0;

  // Real traffic from Clash API (bytes/s → KB/s)
  const upKbps = clashTraffic && typeof clashTraffic.up === "number" ? Math.round(clashTraffic.up / 1024) : isReal ? snap.upload_kbps : 0;
  const downKbps = clashTraffic && typeof clashTraffic.down === "number" ? Math.round(clashTraffic.down / 1024) : isReal ? snap.download_kbps : 0;

  // History: prefer clash-derived history, fallback to system snapshot
  const history = trafficHistory.length ? trafficHistory : isReal && snap.traffic_history?.length ? snap.traffic_history : [];

  // Totals (still from system snapshot until clash provides cumulative totals)
  const totalDown = isReal ? Math.round(snap.total_download_mb) : 0;
  const totalUp = isReal ? Math.round(snap.total_upload_mb) : 0;

  // Config info
  const configName = config?.config_name || "Default";
  const modeLabel = config?.mode === "direct" ? "直接连接" : config?.mode === "global" ? "全局代理" : "规则判定";

  // Proxy node latency from clash proxy history
  const selectedProxyData = clashProxies[selectedProxy];
  const proxyLatency = (() => {
    if (!selectedProxyData?.history?.length) return null;
    const latest = selectedProxyData.history[selectedProxyData.history.length - 1];
    const delay = latest?.delay ?? latest?.latency;
    if (delay === -1 || delay === 0) return null;
    return delay;
  })();

  // Process list for display
  const displayProcs = canUseTauri() && realProcs?.length
    ? realProcs.map((p) => ({
        icon: p.iconBase64 || p.icon_base64 || p.iconKey || p.icon_key,
        app: p.name,
        speed: `${p.connections} 连接`,
        total: `${((p.download_bytes + p.upload_bytes) / 1048576).toFixed(1)} MB`,
      }))
    : [];

  // Connection list for display (strategy tab)
  const displayConnections = canUseTauri() && connectionsData?.length
    ? connectionsData.slice(0, 5).map((c) => ({
        icon: "language",
        app: c.metadata?.host || c.metadata?.destinationIP || c.metadata?.destinationIp || "Unknown",
        speed: (c.chains || []).join(" → ") || "-",
        total: c.rule || "RULE",
      }))
    : [];

  const handleSpeedTest = async () => {
    try {
      addToast("开始测速...", "info");
      await Promise.all(
        nodeOptions.map((proxyName) =>
          safeInvoke("test_proxy_latency", { proxyName }).catch((e) => {
            console.error(`测速 ${proxyName} 失败:`, e);
          })
        )
      );
      addToast("测速请求已发送", "success");
    } catch (e) {
      addToast("测速失败: " + e, "error");
    }
  };

  const handleEnhancedToggle = async (enable) => {
    if (!canUseTauri()) {
      addToast("Tauri 环境不可用", "error");
      return;
    }
    try {
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
    }
  };

  const handleGroupChange = (group) => {
    setSelectedGroup(group);
    const nextGroup = policyGroups.find((g) => (g.name || g.tag || "Proxy") === group);
    if (nextGroup?.now) setSelectedProxy(nextGroup.now);
  };

  const handleProxyChange = async (proxy) => {
    setSelectedProxy(proxy);
    if (!canUseTauri()) return;
    try {
      await safeInvoke("test_proxy_latency", { proxyName: proxy });
    } catch (e) {
      console.error("延迟测试失败:", e);
    }
    try {
      await safeInvoke("set_selector_default", { groupTag: selectedGroup || "Proxy", targetTag: proxy });
      addToast(`已切换 ${selectedGroup || "Proxy"} 到 ${proxy}`, "success");
    } catch (e) {
      try {
        await safeInvoke("switch_proxy", { group: selectedGroup || "Proxy", proxy });
        addToast(`已切换 ${selectedGroup || "Proxy"} 到 ${proxy}`, "success");
      } catch (e2) {
        addToast("代理切换失败: " + (e2.message || e2), "error");
      }
    }
  };

  const handleNetworkDiagnostics = async () => {
    if (!canUseTauri()) {
      addToast("网络诊断功能开发中", "info");
      return;
    }
    try {
      await safeInvoke("run_network_diagnostics");
      addToast("网络诊断完成", "success");
    } catch (e) {
      addToast("网络诊断功能开发中", "info");
    }
  };

  return (
    <div className="page">
      <StatusPills systemProxy={systemProxy} enhanced={tunStatus?.active || enhanced} setSystemProxy={setSystemProxy} setEnhanced={handleEnhancedToggle} />
      {tunStatus?.error && (
        <div className="tun-error-banner" style={{ marginTop: "8px", padding: "8px 12px", background: "rgba(211, 47, 47, 0.08)", borderRadius: "6px", color: "#d32f2f", fontSize: "13px" }}>
          增强模式错误: {tunStatus.error}
        </div>
      )}
      <header className="page-title">
        <h1>活动</h1>
        <div className="title-stats">
          <div><span>网络</span><strong>{ssid}</strong></div>
          <div><span>配置</span><strong>{configName}</strong></div>
          <div><span>出站模式</span><strong>{modeLabel}</strong></div>
          <div><span>外部 IP</span><strong>{externalIp}</strong></div>
        </div>
        <div className="activity-selectors">
          <MenuSelect label="策略组" value={selectedGroup} options={groupOptions} onChange={handleGroupChange} />
          <MenuSelect label="代理" value={selectedProxy} options={proxyOptions.length ? proxyOptions : ["direct"]} onChange={handleProxyChange} />
          <button className="soft-button test-button" onClick={handleSpeedTest}><Icon name="refresh" />测速</button>
        </div>
      </header>
      <div className="activity-grid">
        <section className="card latency">
          <div className="card-toolbar">
            <div><span className="card-label">INTERNET 延迟</span><Icon name="refresh" className="soft-icon" /></div>
            <button className="soft-button" onClick={handleNetworkDiagnostics}>网络诊断</button>
          </div>
          <div className="latency-main">{loading ? "--" : latencyMs != null ? Math.round(latencyMs) : "--"}<span>ms</span></div>
          <div className="latency-sub">
            <div><span>路由</span><strong>{loading ? "--" : routerMs != null ? `${Math.round(routerMs)} ms` : "--"}</strong></div>
            <div><span>DNS</span><strong>{loading ? "--" : dnsMs != null ? `${Math.round(dnsMs)} ms` : "--"}</strong></div>
            <div><span>代理节点</span><strong>{loading ? "--" : proxyLatency != null ? `${Math.round(proxyLatency)} ms` : "--"}</strong></div>
          </div>
        </section>
        <MetricCard label="上传" value={loading ? "--" : Math.round(upKbps)} unit="KB/s" accent="purple">
          <MiniLine color="purple" values={history.length ? history.map((v) => v * 0.8) : [4, 4, 4, 4, 4, 4, 4, 4]} />
        </MetricCard>
        <MetricCard label="下载" value={loading ? "--" : Math.round(downKbps)} unit="KB/s" accent="cyan">
          <MiniLine color="cyan" values={history.length ? history : [4, 4, 4, 4, 4, 4, 4, 4]} />
        </MetricCard>
        <section className="card connections">
          <span className="live-dot" />
          <div className="card-label">活动连接</div>
          <div className="big-number">{loading ? "--" : connectionsCount}</div>
          <div className="latency-sub">
            <div><strong>{loading ? "--" : processCount}</strong><span>进程</span></div>
            <div><strong>{loading ? "--" : deviceCount}</strong><span>设备</span></div>
            <div><strong>{loading ? "--" : dhcpCount}</strong><span>DHCP 设备</span></div>
          </div>
        </section>
        <section className="card traffic">
          <div className="traffic-head">
            <span className="card-label">流量</span>
            <Segmented value={scope} options={["全部", "仅代理"]} onChange={setScope} />
          </div>
          <div className="bar-chart">
            {history.length ? history.map((value, index) => <span key={index} style={{ height: `${Math.min(value, 100)}%` }} />) : <span style={{ height: "4%" }} />}
          </div>
          <div className="time-axis"><span>12AM</span><span>6AM</span><span>12PM</span><span>6PM</span></div>
          <Segmented value={trafficTab} options={["进程与设备", "域名", "策略"]} onChange={setTrafficTab} />
          <ProcessRank compact items={trafficTab === "策略" ? (displayConnections.length ? displayConnections : undefined) : (displayProcs.length ? displayProcs : undefined)} />
        </section>
        <section className="card total">
          <div className="traffic-head">
            <span className="card-label">总计</span>
            <Segmented value="今日" options={["今日", "本月"]} onChange={() => {}} />
          </div>
          <div className="total-main">{loading ? "--" : totalDown + totalUp}<span>MB</span></div>
          <div className="usage-row"><span>DIRECT<br /><strong>{totalDown} MB</strong></span><span>代理<br /><strong>{totalUp} MB</strong></span></div>
          <div className="usage-bar"><span /><b /></div>
        </section>
      </div>
    </div>
  );
}

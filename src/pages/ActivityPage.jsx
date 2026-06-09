import { useEffect, useState } from "react";
import { StatusPills, MetricCard, MiniLine, MenuSelect, Icon, ProcessRank, Segmented } from "../components/ui";
import { useTauriPoll } from "../hooks/tauri";
import { canUseTauri, safeInvoke } from "../utils/tauri";
import { useToast } from "../components/Toast";

export function ActivityPage({ systemProxy, enhanced, setSystemProxy, setEnhanced, selectedProxy, setSelectedProxy, selectedGroup, setSelectedGroup }) {
  const [scope, setScope] = useState("全部");
  const { addToast } = useToast();
  const { data: snap, loading } = useTauriPoll("get_system_snapshot", null, 3000);
  const { data: realProcs } = useTauriPoll("get_processes", null, 5000);
  const { data: config } = useTauriPoll("get_singbox_config", null, 5000);
  const isReal = canUseTauri() && snap;
  const proxyOptions = canUseTauri() && config?.outbounds?.length
    ? config.outbounds
      .filter((outbound) => !["direct", "block", "selector"].includes(outbound.outbound_type))
      .map((outbound) => outbound.tag)
    : [];
  const groupOptions = canUseTauri() && config
    ? ["Proxy"]
    : [];

  useEffect(() => {
    if (proxyOptions.length && !proxyOptions.includes(selectedProxy)) {
      setSelectedProxy(proxyOptions[0]);
    }
    if (groupOptions.length && !groupOptions.includes(selectedGroup)) {
      setSelectedGroup(groupOptions[0]);
    }
  }, [proxyOptions.join("|"), groupOptions.join("|"), selectedProxy, selectedGroup, setSelectedProxy, setSelectedGroup]);

  const ssid = isReal ? snap.ssid : "--";
  const externalIp = isReal ? snap.external_ip : "--";
  const latencyMs = isReal ? snap.internet_latency_ms : null;
  const dnsMs = isReal ? snap.dns_latency_ms : null;
  const routerMs = isReal ? snap.router_latency_ms : null;
  const connectionsCount = isReal ? snap.connections_total : 0;
  const processCount = isReal ? snap.processes_with_connections : 0;
  const upKbps = isReal ? snap.upload_kbps : 0;
  const downKbps = isReal ? snap.download_kbps : 0;
  const totalDown = isReal ? Math.round(snap.total_download_mb) : 0;
  const totalUp = isReal ? Math.round(snap.total_upload_mb) : 0;
  const history = isReal && snap.traffic_history?.length ? snap.traffic_history : [];

  const configName = config?.config_name || "Default";
  const modeLabel = config?.mode === "direct" ? "直接连接" : config?.mode === "global" ? "全局代理" : "规则判定";

  const displayProcs = canUseTauri() && realProcs?.length
    ? realProcs.map(p => ({ icon: p.icon_key, app: p.name, speed: `${p.connections} 连接`, total: `${((p.download_bytes + p.upload_bytes) / 1048576).toFixed(1)} MB` }))
    : [];

  const handleSpeedTest = async () => {
    try {
      addToast("开始测速...", "info");
      const results = await safeInvoke("run_speed_test_all");
      const summary = (results || [])
        .map((item) => `${item.node_name}: ${Math.round(item.latency_ms)} ms`)
        .join("\n");
      addToast(summary || "没有可测速目标", "success");
    } catch (e) {
      addToast("测速失败: " + e, "error");
    }
  };

  return (
    <div className="page">
      <StatusPills systemProxy={systemProxy} enhanced={enhanced} setSystemProxy={setSystemProxy} setEnhanced={setEnhanced} />
      <header className="page-title">
        <h1>活动</h1>
        <div className="title-stats">
          <div><span>网络</span><strong>{ssid}</strong></div>
          <div><span>配置</span><strong>{configName}</strong></div>
          <div><span>出站模式</span><strong>{modeLabel}</strong></div>
          <div><span>外部 IP</span><strong>{externalIp}</strong></div>
        </div>
        <div className="activity-selectors">
          <MenuSelect label="策略组" value={selectedGroup} options={groupOptions} onChange={setSelectedGroup} />
          <MenuSelect label="代理" value={selectedProxy} options={proxyOptions.length ? proxyOptions : ["direct"]} onChange={setSelectedProxy} />
          <button className="soft-button test-button" onClick={handleSpeedTest}><Icon name="refresh" />测速</button>
        </div>
      </header>
      <div className="activity-grid">
        <section className="card latency">
          <div className="card-toolbar">
            <div><span className="card-label">INTERNET 延迟</span><Icon name="refresh" className="soft-icon" /></div>
            <button className="soft-button">网络诊断</button>
          </div>
          <div className="latency-main">{loading ? "--" : latencyMs != null ? Math.round(latencyMs) : "--"}<span>ms</span></div>
          <div className="latency-sub">
            <div><span>路由</span><strong>{loading ? "--" : routerMs != null ? `${Math.round(routerMs)} ms` : "--"}</strong></div>
            <div><span>DNS</span><strong>{loading ? "--" : dnsMs != null ? `${Math.round(dnsMs)} ms` : "--"}</strong></div>
            <div><span>代理节点</span><strong>{loading ? "--" : latencyMs != null ? `${Math.round(latencyMs)} ms` : "--"}</strong></div>
          </div>
        </section>
        <MetricCard label="上传" value={loading ? "--" : Math.round(upKbps)} unit="KB/s" accent="purple">
          <MiniLine color="purple" values={history.length ? history.map(v => v * 0.8) : [4,4,4,4,4,4,4,4]} />
        </MetricCard>
        <MetricCard label="下载" value={loading ? "--" : Math.round(downKbps)} unit="KB/s" accent="cyan">
          <MiniLine color="cyan" values={history.length ? history : [4,4,4,4,4,4,4,4]} />
        </MetricCard>
        <section className="card connections">
          <span className="live-dot" />
          <div className="card-label">活动连接</div>
          <div className="big-number">{loading ? "--" : connectionsCount}</div>
          <div className="latency-sub">
            <div><strong>{loading ? "--" : processCount}</strong><span>进程</span></div>
            <div><strong>0</strong><span>设备</span></div>
            <div><strong>0</strong><span>DHCP 设备</span></div>
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
          <Segmented value="进程与设备" options={["进程与设备", "域名", "策略"]} onChange={() => {}} />
          <ProcessRank compact items={displayProcs.length ? displayProcs : undefined} />
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

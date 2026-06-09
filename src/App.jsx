import { useMemo, useState, useEffect } from "react";
import * as MuiIcons from "@mui/icons-material";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const navGroups = [
  {
    items: [
      { id: "activity", label: "活动", icon: "monitor_heart" },
      { id: "overview", label: "概览", icon: "grid_view" },
    ],
  },
  {
    label: "客户端",
    items: [
      { id: "processes", label: "进程", icon: "terminal" },
      { id: "devices", label: "设备", icon: "router" },
    ],
  },
  {
    label: "代理",
    items: [
      { id: "policy", label: "策略", icon: "alt_route" },
      { id: "rules", label: "规则", icon: "checklist" },
    ],
  },
  {
    label: "HTTP",
    items: [
      { id: "capture", label: "捕获", icon: "capture" },
      { id: "mitm", label: "解密", icon: "lock_open" },
      { id: "rewrite", label: "重写", icon: "edit_note" },
    ],
  },
  {
    items: [{ id: "more", label: "更多", icon: "tune" }],
    pinned: true,
  },
];

const nodes = [
  { type: "AnyTLS", name: "anytls-VM-0-11-ubuntu", ping: "失败", state: "error" },
  { type: "Hysteria 2", name: "hy2-VM-0-11-ubuntu", ping: "209 ms", state: "ok" },
  { type: "TUIC v5", name: "tu5-VM-0-11-ubuntu", ping: "216 ms", state: "selected" },
  { type: "VLESS Reality", name: "vl-reality-VM-0-11-ubuntu", ping: "195 ms", state: "ok" },
  { type: "VMess WS", name: "vm-ws-VM-0-11-ubuntu", ping: "205 ms", state: "ok" },
];

const policyGroups = [
  { name: "Proxy", mode: "手动选择策略组", members: 5 },
  { name: "Auto", mode: "延迟最低", members: 4 },
  { name: "Fallback", mode: "故障转移", members: 4 },
  { name: "Streaming", mode: "流媒体", members: 3 },
];

const processes = [
  { icon: "build", app: "System Services", speed: "0 B/s", total: "8.1 MB" },
  { icon: "language", app: "Google Chrome", speed: "14 KB/s", total: "4.0 MB" },
  { icon: "terminal", app: "xray", speed: "0 B/s", total: "3.4 MB" },
  { icon: "memory", app: "TRAE SOLO CN", speed: "15 KB/s", total: "2.1 MB" },
  { icon: "send", app: "飞书", speed: "0 B/s", total: "1.3 MB" },
  { icon: "deployed_code", app: "Codex", speed: "0 B/s", total: "1.1 MB" },
  { icon: "cloud", app: "夸克", speed: "46 B/s", total: "836 KB" },
  { icon: "chat", app: "微信", speed: "0 B/s", total: "353 KB" },
  { icon: "deployed_code", app: "Cursor", speed: "0 B/s", total: "325 KB" },
  { icon: "explore", app: "Antigravity", speed: "0 B/s", total: "105 KB" },
];

const requests = [
  ["2316", "12:04:07", "Cursor Helper", "活跃", "tu5-VM-0-11...", "2 KB", "4 KB", "9 s", "HTTPS", "api3.cursor.sh:443"],
  ["2315", "12:04:07", "syspolicyd", "活跃", "tu5-VM-0-11...", "2 KB", "8 KB", "10 s", "HTTPS", "api.apple-cloudkit.com:443"],
  ["2314", "12:04:06", "WeChat", "已完成", "tu5-VM-0-11...", "725 B", "347 B", "1 s", "POST", "http://183.60.8.150/mmtls/750028"],
  ["2313", "12:04:06", "Cursor Helper", "已完成", "tu5-VM-0-11...", "4 KB", "2 KB", "2 s", "HTTPS", "api3.cursor.sh:443"],
  ["2312", "12:04:05", "xray", "已完成", "tu5-VM-0-11...", "6 KB", "10 KB", "3 s", "TCP", "ipel.zheshe002.com:60004"],
  ["2311", "12:04:05", "xray", "已完成", "tu5-VM-0-11...", "6 KB", "11 KB", "3 s", "TCP", "ipel.zheshe002.com:60004"],
  ["2310", "12:04:05", "TRAE SOLO CN", "已完成", "tu5-VM-0-11...", "44 KB", "1 KB", "3 s", "HTTPS", "mon.zijieapi.com:443"],
  ["2309", "12:04:04", "DingTalk", "活跃", "tu5-VM-0-11...", "2 KB", "5 KB", "14 s", "HTTPS", "h-adashx.dingtalkapps.com:443"],
];

const rules = [
  ["0", "RULE-SET", "SYSTEM (no-resolve)", "DIRECT", "0"],
  ["1", "DOMAIN-SUFFIX", "apple.com", "DIRECT", "42"],
  ["2", "DOMAIN-KEYWORD", "cursor", "Proxy", "81"],
  ["3", "GEOIP", "CN", "DIRECT", "23"],
  ["4", "FINAL", "", "Proxy", "161"],
];

const trafficBars = [4, 5, 4, 4, 5, 4, 4, 5, 6, 38, 65, 92, 6, 5, 5, 4, 5, 5, 4, 4, 5, 4, 5, 6];

const iconMap = {
  monitor_heart: MuiIcons.MonitorHeartRounded,
  grid_view: MuiIcons.GridViewRounded,
  terminal: MuiIcons.TerminalRounded,
  router: MuiIcons.RouterRounded,
  alt_route: MuiIcons.AltRouteRounded,
  checklist: MuiIcons.ChecklistRounded,
  capture: MuiIcons.CenterFocusStrongRounded,
  lock_open: MuiIcons.LockOpenRounded,
  edit_note: MuiIcons.EditNoteRounded,
  tune: MuiIcons.TuneRounded,
  speed: MuiIcons.SpeedRounded,
  open_in_new: MuiIcons.OpenInNewRounded,
  refresh: MuiIcons.RefreshRounded,
  more_horiz: MuiIcons.MoreHorizRounded,
  add: MuiIcons.AddRounded,
  unfold_more: MuiIcons.UnfoldMoreRounded,
  settings: MuiIcons.SettingsRounded,
  search: MuiIcons.SearchRounded,
  add_circle: MuiIcons.AddCircleRounded,
  check_box: MuiIcons.CheckBoxRounded,
  check_box_outline_blank: MuiIcons.CheckBoxOutlineBlankRounded,
  workspace_premium: MuiIcons.WorkspacePremiumRounded,
  build: MuiIcons.BuildRounded,
  language: MuiIcons.LanguageRounded,
  memory: MuiIcons.MemoryRounded,
  send: MuiIcons.SendRounded,
  deployed_code: MuiIcons.AppsRounded,
  cloud: MuiIcons.CloudRounded,
  chat: MuiIcons.ChatRounded,
  palette: MuiIcons.PaletteRounded,
  explore: MuiIcons.ExploreRounded,
  description: MuiIcons.DescriptionRounded,
  sync: MuiIcons.SyncRounded,
  experiment: MuiIcons.ScienceRounded,
  toggle_on: MuiIcons.ToggleOnRounded,
};

function Icon({ name, className = "" }) {
  const Component = iconMap[name] || MuiIcons.RadioButtonUncheckedRounded;
  return <Component className={`icon ${className}`} fontSize="inherit" aria-hidden="true" />;
}

function canUseTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function safeInvoke(command, args) {
  if (!canUseTauri()) return null;
  return invoke(command, args);
}

function useTauriPoll(command, args = null, interval = 3000, defaultValue = null) {
  const [data, setData] = useState(defaultValue);
  const [loading, setLoading] = useState(canUseTauri());
  useEffect(() => {
    if (!canUseTauri()) { setLoading(false); return; }
    let active = true;
    const fetch = async () => {
      try {
        const result = await invoke(command, args || {});
        if (active) { setData(result); setLoading(false); }
      } catch (e) { console.error(`${command}:`, e); if (active) setLoading(false); }
    };
    fetch();
    const timer = setInterval(fetch, interval);
    return () => { active = false; clearInterval(timer); };
  }, [command, interval]);
  return { data, loading };
}

function useTauriData(command, defaultValue = null) {
  const [data, setData] = useState(defaultValue);
  const [loading, setLoading] = useState(canUseTauri());
  useEffect(() => {
    if (!canUseTauri()) { setLoading(false); return; }
    let active = true;
    invoke(command).then(r => { if (active) { setData(r); setLoading(false); } }).catch(e => { console.error(e); if (active) setLoading(false); });
    return () => { active = false; };
  }, [command]);
  return { data, loading };
}

function Toggle({ checked, onChange }) {
  return (
    <button className={`toggle ${checked ? "is-on" : ""}`} onClick={() => onChange(!checked)} aria-label="toggle">
      <span />
    </button>
  );
}

function Segmented({ value, options, onChange }) {
  return (
    <div className="segmented">
      {options.map((option) => (
        <button className={value === option ? "active" : ""} key={option} onClick={() => onChange(option)}>
          {option}
        </button>
      ))}
    </div>
  );
}

function MenuSelect({ label, value, options, onChange }) {
  return (
    <label className="menu-select">
      <span>{label}</span>
      <select value={value} onChange={(event) => onChange(event.target.value)}>
        {options.map((option) => (
          <option key={option} value={option}>{option}</option>
        ))}
      </select>
      <Icon name="unfold_more" />
    </label>
  );
}

function Sidebar({ active, onNavigate }) {
  return (
    <aside className="sidebar">
      <div className="traffic-lights" aria-hidden="true" data-tauri-drag-region="true">
        <span className="red" />
        <span className="yellow" />
        <span className="green" />
      </div>
      <nav className="nav">
        <div className="nav-scroll">
          {navGroups.filter((group) => !group.pinned).map((group, index) => (
            <div className="nav-group" key={index}>
              {group.label && <div className="nav-heading">{group.label}</div>}
              {group.items.map((item) => (
                <button
                  className={`nav-item ${active === item.id ? "active" : ""}`}
                  key={item.id}
                  data-nav-id={item.id}
                  aria-label={item.label}
                  onClick={() => onNavigate(item.id)}
                >
                  <Icon name={item.icon} />
                  <span>{item.label}</span>
                </button>
              ))}
            </div>
          ))}
        </div>
        <div className="nav-bottom">
          <button className={`nav-item ${active === "more" ? "active" : ""}`} data-nav-id="more" aria-label="更多" onClick={() => onNavigate("more")}>
            <Icon name="tune" />
            <span>更多</span>
          </button>
          <button className="nav-item panel-link">
            <Icon name="speed" />
            <span>面板</span>
            <Icon name="open_in_new" className="mini" />
          </button>
        </div>
      </nav>
    </aside>
  );
}

function StatusPills({ systemProxy, enhanced, setSystemProxy, setEnhanced }) {
  return (
    <div className="status-row">
      <div className="pill status">
        <span className="dot green" />
        <strong>信息</strong>
        <span>超时（代理握手）</span>
      </div>
      <button className={`pill switch ${systemProxy ? "active" : ""}`} onClick={() => setSystemProxy(!systemProxy)}>
        <span className="dot green" />
        <strong>系统代理</strong>
      </button>
      <button className={`pill switch ${enhanced ? "active" : ""}`} onClick={() => setEnhanced(!enhanced)}>
        <span className="dot green" />
        <strong>增强模式</strong>
      </button>
    </div>
  );
}

function MetricCard({ label, value, unit, accent = "blue", children }) {
  return (
    <section className={`card metric ${accent}`}>
      <div className="card-label">{label}</div>
      <div className="metric-value">
        {value}
        {unit && <span>{unit}</span>}
      </div>
      {children}
    </section>
  );
}

function MiniLine({ color = "blue", values = [40, 40, 35, 36, 62, 70, 38, 32, 52, 40] }) {
  const points = values.map((value, index) => `${index * 22},${88 - value}`).join(" ");
  return (
    <svg className="mini-line" viewBox="0 0 198 96" role="img" aria-label="traffic chart">
      <defs>
        <linearGradient id={`fill-${color}`} x1="0" x2="0" y1="0" y2="1">
          <stop stopColor={`var(--${color})`} stopOpacity="0.22" />
          <stop stopColor={`var(--${color})`} stopOpacity="0" />
        </linearGradient>
      </defs>
      <polyline points={`0,96 ${points} 198,96`} fill={`url(#fill-${color})`} stroke="none" />
      <polyline points={points} fill="none" stroke={`var(--${color})`} strokeWidth="4" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function ActivityPage({ systemProxy, enhanced, setSystemProxy, setEnhanced, selectedProxy, setSelectedProxy, selectedGroup, setSelectedGroup }) {
  const [scope, setScope] = useState("全部");
  const { data: snap, loading } = useTauriPoll("get_system_snapshot", null, 3000);
  const isReal = canUseTauri() && snap;

  const ssid = isReal ? snap.ssid : "Wi-Fi";
  const externalIp = isReal ? snap.external_ip : "...";
  const latencyMs = isReal ? snap.internet_latency_ms : 57;
  const dnsMs = isReal ? snap.dns_latency_ms : 36;
  const routerMs = isReal ? snap.router_latency_ms : 4;
  const connectionsCount = isReal ? snap.connections_total : 73;
  const processCount = isReal ? snap.processes_with_connections : 15;
  const upKbps = isReal ? snap.upload_kbps : 11;
  const downKbps = isReal ? snap.download_kbps : 47;
  const totalDown = isReal ? Math.round(snap.total_download_mb) : 583;
  const totalUp = isReal ? Math.round(snap.total_upload_mb) : 20.5;
  const history = isReal && snap.traffic_history?.length ? snap.traffic_history : trafficBars;

  return (
    <div className="page">
      <StatusPills systemProxy={systemProxy} enhanced={enhanced} setSystemProxy={setSystemProxy} setEnhanced={setEnhanced} />
      <header className="page-title">
        <h1>活动</h1>
        <div className="title-stats">
          <div><span>网络</span><strong>{ssid}</strong></div>
          <div><span>配置</span><strong>Default</strong></div>
          <div><span>出站模式</span><strong>全局代理</strong></div>
          <div><span>外部 IP</span><strong>{externalIp}</strong></div>
        </div>
        <div className="activity-selectors">
          <MenuSelect label="策略组" value={selectedGroup} options={policyGroups.map((group) => group.name)} onChange={setSelectedGroup} />
          <MenuSelect label="代理" value={selectedProxy} options={nodes.map((node) => node.name)} onChange={setSelectedProxy} />
          <button className="soft-button test-button" onClick={async () => { await safeInvoke("run_speed_test_all"); }}><Icon name="refresh" />测速</button>
        </div>
      </header>
      <div className="activity-grid">
        <section className="card latency">
          <div className="card-toolbar">
            <div><span className="card-label">INTERNET 延迟</span><Icon name="refresh" className="soft-icon" /></div>
            <button className="soft-button">网络诊断</button>
          </div>
          <div className="latency-main">{loading ? "--" : Math.round(latencyMs)}<span>ms</span></div>
          <div className="latency-sub">
            <div><span>路由</span><strong>{loading ? "--" : Math.round(routerMs)} ms</strong></div>
            <div><span>DNS</span><strong>{loading ? "--" : Math.round(dnsMs)} ms</strong></div>
            <div><span>代理节点</span><strong>{loading ? "--" : latencyMs ? `${Math.round(latencyMs)} ms` : "失败"}</strong></div>
          </div>
        </section>
        <MetricCard label="上传" value={loading ? "--" : Math.round(upKbps)} unit="KB/s" accent="purple">
          <MiniLine color="purple" values={history.map(v => v * 0.8)} />
        </MetricCard>
        <MetricCard label="下载" value={loading ? "--" : Math.round(downKbps)} unit="KB/s" accent="cyan">
          <MiniLine color="cyan" values={history} />
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
            {history.map((value, index) => <span key={index} style={{ height: `${Math.min(value, 100)}%` }} />)}
          </div>
          <div className="time-axis"><span>12AM</span><span>6AM</span><span>12PM</span><span>6PM</span></div>
          <Segmented value="进程与设备" options={["进程与设备", "域名", "策略"]} onChange={() => {}} />
          <ProcessRank compact />
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

function OverviewPage({ systemProxy, enhanced, setSystemProxy, setEnhanced }) {
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

function OverviewCard({ title, text, status, checked, onChange, color }) {
  return (
    <section className="card overview-card">
      <div className="overview-head">
        <h3>{title}</h3>
        <Toggle checked={checked} onChange={onChange} />
      </div>
      <p>{text}</p>
      <div className="card-status"><span className={`dot ${color}`} />{status}<Icon name="more_horiz" /></div>
    </section>
  );
}

function ProcessRank({ compact = false, selectedApp = "", onSelect, items }) {
  const list = items || processes;
  return (
    <div className={`process-rank ${compact ? "compact" : ""}`}>
      {list.slice(0, compact ? 5 : list.length).map((item, index) => (
        <button
          className={`rank-row ${selectedApp === item.app ? "selected" : ""}`}
          key={item.app}
          onClick={() => onSelect?.(item)}
        >
          <span className={`app-icon tone-${index % 6}`}><Icon name={item.icon} /></span>
          <strong>{item.app}</strong>
          <em>{compact ? ["276.4 MB", "187.8 MB", "41.9 MB", "41.3 MB", "24.8 MB"][index] : item.speed}</em>
          {!compact && <small>总计 {item.total}</small>}
          {compact && <div className="mini-track"><span style={{ width: `${90 - index * 15}%` }} /></div>}
        </button>
      ))}
    </div>
  );
}

function ProcessesPage() {
  const [metered, setMetered] = useState(false);
  const { data: realProcs } = useTauriPoll("get_processes", null, 5000);
  const displayProcs = canUseTauri() && realProcs?.length
    ? realProcs.map(p => ({ icon: p.icon_key, app: p.name, speed: `${p.connections} 连接`, total: `${((p.download_bytes + p.upload_bytes) / 1048576).toFixed(1)} MB` }))
    : processes;
  const [selectedProcess, setSelectedProcess] = useState(displayProcs[1]);
  useEffect(() => { if (displayProcs?.length && !displayProcs.find(p => p.app === selectedProcess?.app)) setSelectedProcess(displayProcs[1]); }, [displayProcs]);

  return (
    <SplitPage title="进程" control="按流量排序">
      <div className="list-pane">
        <ProcessRank selectedApp={selectedProcess.app} onSelect={setSelectedProcess} items={displayProcs} />
        <div className="metered-row">
          <div><strong>计费网络模式</strong><p>启动后所有进程将默认禁止使用网络，适用于按流量计费的网络。</p></div>
          <Toggle checked={metered} onChange={setMetered} />
          <Icon name="settings" />
        </div>
      </div>
      <ProcessDetail process={selectedProcess} />
    </SplitPage>
  );
}

function DevicesPage() {
  const [gateway, setGateway] = useState(false);
  return (
    <SplitPage title="设备" control="按 IP 排序">
      <div className="empty-pane">
        <span>无设备</span>
        <div className="metered-row bottom">
          <div>
            <strong>网关模式</strong>
            <p>可以使用 Helio 作为局域网的 DHCP 服务器，接管其他设备的网络。</p>
          </div>
          <Toggle checked={gateway} onChange={setGateway} />
          <Icon name="settings" />
        </div>
      </div>
      <DeviceDetail gateway={gateway} />
    </SplitPage>
  );
}

function SplitPage({ title, control, children }) {
  return (
    <div className="page card-shell">
      <div className="split-header">
        <h1>{title}</h1>
        <button className="sort-button">{control}<Icon name="unfold_more" /></button>
      </div>
      <div className="split-grid">{children}</div>
    </div>
  );
}

function ProcessDetail({ process }) {
  const rows = [
    ["当前速度", process.speed],
    ["累计流量", process.total],
    ["活动连接", process.app === "Google Chrome" ? "6" : "2"],
    ["命中策略", process.app === "System Services" ? "DIRECT" : "Proxy"],
    ["最近地址", process.app === "Google Chrome" ? "api3.cursor.sh:443" : "ipel.zheshe002.com:60004"],
    ["DNS", "system-resolver"],
  ];
  return (
    <aside className="detail-panel">
      <div className="detail-hero">
        <span className="app-icon tone-1"><Icon name={process.icon} /></span>
        <div>
          <h2>{process.app}</h2>
          <p>按进程查看实时连接、规则命中和流量统计。</p>
        </div>
      </div>
      <div className="detail-stats">
        {rows.map(([label, value]) => (
          <div key={label}>
            <span>{label}</span>
            <strong>{value}</strong>
          </div>
        ))}
      </div>
      <div className="detail-chart">
        <MiniLine color="cyan" values={[25, 31, 28, 44, 66, 52, 60, 35, 39, 48]} />
      </div>
    </aside>
  );
}

function DeviceDetail({ gateway }) {
  return (
    <aside className="detail-panel device-detail">
      <div className="detail-hero">
        <span className={`status-orb ${gateway ? "active" : ""}`} />
        <div>
          <h2>{gateway ? "网关待发现" : "设备列表为空"}</h2>
          <p>{gateway ? "网关模式开启后，新设备会显示在这里并可单独分配策略。" : "开启网关模式后，局域网设备会出现在此面板。"}</p>
        </div>
      </div>
      <div className="device-steps">
        <div><Icon name="router" /><span>启用 DHCP 网关</span></div>
        <div><Icon name="alt_route" /><span>选择默认策略组</span></div>
        <div><Icon name="checklist" /><span>为设备绑定规则</span></div>
      </div>
    </aside>
  );
}

function PolicyPage({ selectedProxy, setSelectedProxy, selectedGroup, setSelectedGroup }) {
  const [mode, setMode] = useState("全局代理");
  const { data: config } = useTauriData("get_singbox_config");
  const isReal = canUseTauri() && config;
  const displayNodes = isReal && config.outbounds?.length
    ? config.outbounds.map(o => ({ type: o.outbound_type, name: o.tag, ping: o.ping || "-", state: o.state || "ok" }))
    : nodes;
  const displayGroups = isReal && config.rules?.length
    ? [{ name: "Proxy", mode: "手动选择策略组", members: config.outbounds?.length || 5 }]
    : policyGroups;
  return (
    <div className="page airy policy-page">
      <h1>代理配置</h1>
      <Segmented value={mode} options={["直接连接", "全局代理", "规则判定"]} onChange={setMode} />
      <p className="muted">{mode === "全局代理" ? "这里仅配置出站模式、节点和策略组；实时流量与连接状态集中在活动页查看。" : "当前模式会作为配置写入内核路由策略。"}</p>
      <div className="policy-head">
        <h2 className="section-title magenta">节点配置</h2>
        <button className="ghost-button">测试全部</button>
      </div>
      <div className="node-grid">
        {displayNodes.map((node) => (
          <button className={`node-card ${selectedProxy === node.name ? "selected" : ""}`} key={node.name} onClick={() => setSelectedProxy(node.name)}>
            <span>{node.type}</span>
            <strong>{node.name}</strong>
            <em className={node.state}>{node.ping}</em>
          </button>
        ))}
        <button className="node-card add-card"><Icon name="add" /></button>
      </div>
      <h2 className="section-title cyan">策略组</h2>
      <div className="node-grid group-row">
        {displayGroups.slice(0, 3).map((group) => (
          <button className={`node-card ${selectedGroup === group.name ? "selected" : ""}`} key={group.name} onClick={() => setSelectedGroup(group.name)}>
            <span>{group.mode}</span>
            <strong>{group.name}</strong>
            <small>{group.members} 个节点 · 当前 {selectedProxy}</small>
          </button>
        ))}
        <button className="node-card add-card"><Icon name="add" /></button>
      </div>
    </div>
  );
}

function RulesPage() {
  const [query, setQuery] = useState("");
  const { data: config } = useTauriData("get_singbox_config");
  const isReal = canUseTauri() && config;
  const displayRules = isReal && config.rules?.length
    ? config.rules.map(r => [r.id, r.rule_type, r.value, r.action, r.hits])
    : rules;
  const visibleRules = useMemo(() => displayRules.filter((rule) => rule.join(" ").toLowerCase().includes(query.toLowerCase())), [query, displayRules]);
  return (
    <div className="page card-shell rules-page">
      <div className="split-header">
        <div>
          <h1>规则</h1>
          <p className="muted">规则将按照从上至下的顺序进行测试。</p>
        </div>
        <label className="search"><Icon name="search" /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="搜索" /></label>
      </div>
      <div className="rule-table">
        <div className="thead"><span /><span>ID</span><span>类型</span><span>值</span><span>策略</span><span>使用</span></div>
        {visibleRules.map((rule, index) => (
          <div className={`tr ${index === 0 ? "selected" : ""}`} key={`${rule[0]}-${rule[1]}`}>
            <span className="row-dot" /><span>{rule[0]}</span><span>{rule[1]}</span><span>{rule[2]}</span><span>{rule[3]}</span><span>{rule[4]}</span>
          </div>
        ))}
      </div>
      <div className="table-actions">
        <button>+</button><button>-</button><button>重置计数器</button>
      </div>
    </div>
  );
}

function CapturePage() {
  const [tab, setTab] = useState("最近的请求");
  const [capturing, setCapturing] = useState(false);
  const { data: realConns } = useTauriPoll("get_connections", null, 3000);
  const displayReqs = canUseTauri() && realConns?.length
    ? realConns.map((c, i) => [String(i + 1), c.timestamp, c.process, c.status, c.proxy, c.upload, c.download, c.duration, c.method, c.remote])
    : requests;
  return (
    <div className="capture-window">
      <div className="capture-sidebar">
        <div className="traffic-lights small" data-tauri-drag-region="true"><span className="red" /><span className="yellow" /><span className="green" /></div>
        <Segmented value="按客户端" options={["按客户端", "按主机名"]} onChange={() => {}} />
        <h3>请求</h3>
        <button className="side-chip active">所有客户端</button>
        <h3>本地程序 <Icon name="settings" /></h3>
        {processes.slice(5).concat(processes.slice(0, 5)).map((item, index) => (
          <button className="app-filter" key={`${item.app}-${index}`}>
            <span className={`app-icon tone-${index % 6}`}><Icon name={item.icon} /></span>{item.app}
          </button>
        ))}
      </div>
      <main className="request-pane">
        <div className="request-tabs">
          <Segmented value={tab} options={["最近的请求", "活动连接", "DNS", "设备", "流量统计", "日志簿"]} onChange={setTab} />
          <label className="search compact-search"><Icon name="search" /><input placeholder="搜索" /></label>
        </div>
        <div className="request-table">
          <div className="thead"><span>ID</span><span>日期</span><span>客户端</span><span>状态</span><span>策略</span><span>上传</span><span>下载</span><span>时长</span><span>方法</span><span>地址</span></div>
          {displayReqs.map((row) => (
            <div className="tr" key={row[0]}>
              {row.map((cell, index) => <span key={index}>{index === 0 && <i className="status-dot" />}{cell}</span>)}
            </div>
          ))}
        </div>
        <div className="capture-actions">
          <button>清空</button>
          <button>重新载入</button>
          <button onClick={() => setCapturing(!capturing)}>{capturing ? "停止流量捕获" : "启动流量捕获"}</button>
          <button disabled>开启 MitM</button>
          <button>代理控制</button>
        </div>
      </main>
    </div>
  );
}

function MitmPage() {
  const [enabled, setEnabled] = useState(false);
  return (
    <div className="page settings-page">
      <div className="headline-toggle"><h1>HTTPS 解密</h1><Toggle checked={enabled} onChange={setEnabled} /></div>
      <p className="muted">通过中间人攻击（MitM, man-in-the-middle）的方式解密 HTTPS 流量。</p>
      <div className="two-col">
        <section>
          <h2 className="section-title orange">CA 证书</h2>
          <div className="cert-row"><Icon name="workspace_premium" /><div><strong>配置中没有 CA 证书</strong><span>CA 证书不被系统信任</span></div></div>
          <div className="form-actions"><span>操作:</span><button>生成新证书</button><button disabled>将证书安装到系统</button><button>从 PKCS #12 文件导入证书</button><button disabled>为 iOS 模拟器导出证书</button></div>
          <h2 className="section-title green">选项</h2>
          <CheckItem label="进行 MitM 时跳过服务端证书验证" disabled />
          <CheckItem label="自动屏蔽 QUIC" disabled />
          <CheckItem label="MitM over HTTP/2" />
        </section>
        <section>
          <div className="host-box"><div>MitM 主机名 <Icon name="add_circle" /></div></div>
          <p className="muted">只有该列表中的域名才会被 Helio 进行解密，允许使用通配符。</p>
        </section>
      </div>
    </div>
  );
}

function CheckItem({ label, disabled = false }) {
  const [checked, setChecked] = useState(false);
  return (
    <button className={`check-item ${checked ? "checked" : ""}`} disabled={disabled} onClick={() => setChecked(!checked)}>
      <Icon name={checked ? "check_box" : "check_box_outline_blank"} />{label}
    </button>
  );
}

function RewritePage() {
  const [enabled, setEnabled] = useState(true);
  const rewrites = [
    ["URL 重定向", "重定向 HTTP 请求，也称为 Map Remote。", "编辑 URL 重定向规则..."],
    ["Header 重写", "修改发送到服务器的 HTTP Header，以及修改返回响应的 Header。", "编辑 Header 重写规则..."],
    ["Mock", "模拟 API 服务器并返回静态响应。", "编辑 Mock 规则..."],
    ["Body 重写", "重写 HTTP 请求或响应的 Body，用正则表达式替换原始内容。", "编辑 Body 重写规则..."],
  ];
  return (
    <div className="page airy">
      <div className="headline-toggle"><h1>重写 & 映射</h1><Toggle checked={enabled} onChange={setEnabled} /></div>
      <div className="rewrite-grid">
        {rewrites.map(([title, text, action]) => (
          <section className="card rewrite-card" key={title}>
            <h3>{title}</h3>
            <p>{text}</p>
            <div><span>0 条规则生效中</span><button>{action}</button></div>
          </section>
        ))}
      </div>
      <p className="muted">以上计数包含了模块中的规则。</p>
    </div>
  );
}

function MorePage() {
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

export function App() {
  const [active, setActive] = useState("activity");
  const [systemProxy, setSystemProxyState] = useState(false);
  const [enhanced, setEnhanced] = useState(false);
  const [selectedProxy, setSelectedProxy] = useState("tu5-VM-0-11-ubuntu");
  const [selectedGroup, setSelectedGroup] = useState("Proxy");

  const setSystemProxy = async (enable) => {
    setSystemProxyState(enable);
    try {
      await safeInvoke("set_system_proxy", { enable });
    } catch (e) {
      console.error("Failed to set system proxy", e);
    }
  };

  useEffect(() => {
    if (!canUseTauri()) return undefined;
    // Start sing-box engine
    const dummyConfig = JSON.stringify({
      log: { level: "info" },
      inbounds: [{ type: "mixed", tag: "mixed-in", listen: "127.0.0.1", listen_port: 6152 }]
    });
    safeInvoke("start_engine", { config: dummyConfig }).catch(console.error);
    // Start background monitoring
    safeInvoke("start_monitoring").catch(console.error);
    const unlisten = listen("traffic-update", (_event) => {
      // Real-time traffic events supplement polling
    }).catch(() => {});
    return () => {
      safeInvoke("stop_engine").catch(console.error);
      safeInvoke("stop_monitoring").catch(console.error);
      unlisten.then?.(fn => fn()).catch(() => {});
    };
  }, []);

  const pageProps = { systemProxy, enhanced, setSystemProxy, setEnhanced, selectedProxy, setSelectedProxy, selectedGroup, setSelectedGroup };
  const pages = {
    activity: <ActivityPage {...pageProps} />,
    overview: <OverviewPage {...pageProps} />,
    processes: <ProcessesPage />,
    devices: <DevicesPage />,
    policy: <PolicyPage {...pageProps} />,
    rules: <RulesPage />,
    capture: <CapturePage />,
    mitm: <MitmPage />,
    rewrite: <RewritePage />,
    more: <MorePage />,
  };

  return (
    <div className={`app-shell ${canUseTauri() ? "is-tauri" : "is-web"}`}>
      <Sidebar active={active} onNavigate={setActive} />
      <main className="content">{pages[active]}</main>
    </div>
  );
}

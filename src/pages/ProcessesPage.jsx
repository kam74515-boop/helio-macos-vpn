import { useState, useEffect } from "react";
import { SplitPage, ProcessRank, Toggle, Icon, MiniLine } from "../components/ui";
import { useTauriPoll } from "../hooks/tauri";
import { canUseTauri } from "../utils/tauri";

export function ProcessDetail({ process }) {
  const rows = [
    ["当前速度", process.speed || "--"],
    ["累计流量", process.total || "--"],
    ["活动连接", process.connections != null ? String(process.connections) : "--"],
    ["命中策略", "--"],
    ["最近地址", "--"],
    ["DNS", "system-resolver"],
  ];
  return (
    <aside className="detail-panel">
      <div className="detail-hero">
        <span className="app-icon tone-1">
          {typeof process.icon === "string" && process.icon.startsWith("data:")
            ? <img src={process.icon} alt="" style={{ width: "100%", height: "100%", objectFit: "contain", borderRadius: "inherit" }} />
            : <Icon name={process.icon || "memory"} />}
        </span>
        <div>
          <h2>{process.app || "--"}</h2>
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
        <MiniLine color="cyan" values={[4, 5, 4, 6, 5, 4, 5, 4, 5, 6]} />
      </div>
    </aside>
  );
}

export function ProcessesPage() {
  const [metered, setMetered] = useState(false);
  const { data: realProcs, loading } = useTauriPoll("get_processes", null, 5000);
  const displayProcs = canUseTauri() && realProcs?.length
    ? realProcs.map(p => ({
        icon: p.iconBase64 || p.icon_base64 || p.iconKey || p.icon_key,
        app: p.name,
        speed: `${p.connections} 连接`,
        total: `${((p.download_bytes + p.upload_bytes) / 1048576).toFixed(1)} MB`,
        connections: p.connections
      }))
    : [];
  const [selectedProcess, setSelectedProcess] = useState(displayProcs[0] || { icon: "memory", app: "--", speed: "--", total: "--", connections: 0 });
  useEffect(() => { if (displayProcs?.length && !displayProcs.find(p => p.app === selectedProcess?.app)) setSelectedProcess(displayProcs[0]); }, [displayProcs]);

  return (
    <SplitPage title="进程" control="按流量排序">
      <div className="list-pane">
        {loading && <div style={{ padding: 16, color: "var(--muted)", textAlign: "center" }}>加载中...</div>}
        {!loading && displayProcs.length === 0 && (
          <div style={{ padding: 16, color: "var(--muted)", textAlign: "center" }}>暂无活动进程</div>
        )}
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

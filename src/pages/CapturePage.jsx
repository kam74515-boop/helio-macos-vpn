import { useState } from "react";
import { Segmented, Icon } from "../components/ui";
import { useTauriPoll } from "../hooks/tauri";
import { canUseTauri } from "../utils/tauri";

export function CapturePage() {
  const [tab, setTab] = useState("最近的请求");
  const [capturing, setCapturing] = useState(false);
  const { data: realConns, loading } = useTauriPoll("get_connections", null, 3000);
  const displayReqs = canUseTauri() && realConns?.length
    ? realConns.map((c, i) => [String(i + 1), c.timestamp, c.process, c.status, c.proxy, c.upload, c.download, c.duration, c.method, c.remote])
    : [];
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

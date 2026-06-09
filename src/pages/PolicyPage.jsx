import { useEffect, useState } from "react";
import { Segmented, Icon } from "../components/ui";
import { useTauriData } from "../hooks/tauri";
import { canUseTauri, safeInvoke } from "../utils/tauri";
import { useToast } from "../components/Toast";
import { nodes, policyGroups } from "../data/mock";

export function PolicyPage({ selectedProxy, setSelectedProxy, selectedGroup, setSelectedGroup }) {
  const [mode, setMode] = useState("全局代理");
  const [config, setConfig] = useState(null);
  const [busy, setBusy] = useState(false);
  const { addToast } = useToast();
  const { data: initialConfig } = useTauriData("get_singbox_config");
  useEffect(() => {
    if (initialConfig) {
      setConfig(initialConfig);
      const m = initialConfig.mode;
      if (m === "direct") setMode("直接连接");
      else if (m === "global") setMode("全局代理");
      else if (m === "rule") setMode("规则判定");
    }
  }, [initialConfig]);
  const isReal = canUseTauri() && config;
  const displayNodes = isReal && config.outbounds?.length
    ? config.outbounds
      .filter((o) => !["direct", "block", "selector"].includes(o.outbound_type))
      .map(o => ({ type: o.outbound_type, name: o.tag, ping: o.ping || "-", state: o.state || "ok" }))
    : nodes;
  const displayGroups = isReal
    ? [{ name: "Proxy", mode: "手动选择策略组", members: displayNodes.length }]
    : policyGroups;

  const handleModeChange = async (newMode) => {
    setMode(newMode);
    if (!canUseTauri()) return;
    const modeMap = { "直接连接": "direct", "全局代理": "global", "规则判定": "rule" };
    const modeKey = modeMap[newMode];
    if (!modeKey) return;
    try {
      setBusy(true);
      await safeInvoke("set_proxy_mode", { mode: modeKey });
      const latest = await safeInvoke("get_singbox_config");
      if (latest) setConfig(latest);
      addToast(`已切换为 ${newMode} 模式`, "success");
    } catch (e) {
      addToast("模式切换失败: " + e, "error");
    } finally {
      setBusy(false);
    }
  };

  const handleImport = async () => {
    const url = window.prompt("输入订阅链接或 URI/base64 节点内容");
    if (!url) return;
    try {
      setBusy(true);
      const res = await safeInvoke("import_subscription", { url });
      const latest = await safeInvoke("get_singbox_config");
      if (latest) setConfig(latest);
      addToast(`${res.message}: ${res.imported_nodes} 个节点`, "success");
    } catch (e) {
      addToast("导入失败: " + e, "error");
    } finally {
      setBusy(false);
    }
  };

  const handleSpeedTest = async () => {
    try {
      setBusy(true);
      addToast("开始测速...", "info");
      const results = await safeInvoke("run_speed_test_all");
      const summary = (results || [])
        .map((item) => `${item.node_name}: ${Math.round(item.latency_ms)} ms`)
        .join("\n");
      addToast(summary || "没有可测速目标", "success");
    } catch (e) {
      addToast("测速失败: " + e, "error");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="page airy policy-page">
      <h1>代理配置</h1>
      <Segmented value={mode} options={["直接连接", "全局代理", "规则判定"]} onChange={handleModeChange} />
      <p className="muted">{mode === "全局代理" ? "这里仅配置出站模式、节点和策略组；实时流量与连接状态集中在活动页查看。" : "当前模式会作为配置写入内核路由策略。"}</p>
      <div className="policy-head">
        <h2 className="section-title magenta">节点配置</h2>
        <div style={{ display: 'flex', gap: '8px' }}>
          <button className="ghost-button" onClick={handleImport} disabled={busy}>{busy ? "处理中" : "导入订阅"}</button>
          <button className="ghost-button" onClick={handleSpeedTest} disabled={busy}>测试全部</button>
        </div>
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

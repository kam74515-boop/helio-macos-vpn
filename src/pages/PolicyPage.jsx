import { useEffect, useMemo, useState } from "react";
import { Segmented, Icon } from "../components/ui";
import { useTauriData } from "../hooks/tauri";
import { canUseTauri, safeInvoke } from "../utils/tauri";
import { useToast } from "../components/Toast";

const NODE_TYPES = [
  ["vless", "VLESS"],
  ["vmess", "VMess"],
  ["trojan", "Trojan"],
  ["hysteria2", "Hysteria 2"],
  ["tuic", "TUIC"],
  ["anytls", "AnyTLS"],
  ["shadowsocks", "Shadowsocks"],
];

function normalizeNode(outbound) {
  const raw = outbound.raw || {};
  return {
    type: outbound.outbound_type,
    name: outbound.tag,
    server: outbound.server || raw.server || "-",
    serverPort: outbound.server_port || raw.server_port || 0,
    ping: outbound.ping || "-",
    state: outbound.state || "ok",
    raw,
  };
}

function nodeToForm(node) {
  const raw = node?.raw || {};
  const tls = raw.tls || {};
  return {
    tag: node?.name || raw.tag || "",
    outboundType: node?.type || raw.type || "vless",
    server: node?.server !== "-" ? node?.server || raw.server || "" : raw.server || "",
    serverPort: node?.serverPort || raw.server_port || "",
    uuid: raw.uuid || "",
    password: raw.password || "",
    method: raw.method || "",
    security: tls.reality?.enabled ? "reality" : tls.enabled ? "tls" : "",
    sni: tls.server_name || "",
    rawJson: node ? JSON.stringify(raw, null, 2) : "",
  };
}

function Modal({ title, subtitle, children, onClose }) {
  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
      <section className="modal-panel" role="dialog" aria-modal="true" aria-label={title} onMouseDown={(event) => event.stopPropagation()}>
        <div className="modal-head">
          <div>
            <h2>{title}</h2>
            {subtitle && <p>{subtitle}</p>}
          </div>
          <button className="icon-button" onClick={onClose} aria-label="关闭"><Icon name="close" /></button>
        </div>
        {children}
      </section>
    </div>
  );
}

function ImportDialog({ busy, onClose, onSubmit }) {
  const [value, setValue] = useState("");
  return (
    <Modal title="导入订阅" subtitle="支持订阅链接、URI 列表或 base64 编码的节点内容。" onClose={onClose}>
      <form className="modal-form" onSubmit={(event) => {
        event.preventDefault();
        onSubmit(value);
      }}>
        <label className="field full">
          <span>订阅链接 / 节点内容</span>
          <textarea value={value} onChange={(event) => setValue(event.target.value)} placeholder="https://example.com/sub 或 vless://..." autoFocus />
        </label>
        <div className="modal-actions">
          <button type="button" className="ghost-button" onClick={onClose}>取消</button>
          <button type="submit" className="primary-button" disabled={busy || !value.trim()}>{busy ? "导入中" : "导入并重启内核"}</button>
        </div>
      </form>
    </Modal>
  );
}

function NodeDialog({ node, busy, onClose, onSubmit }) {
  const [form, setForm] = useState(() => nodeToForm(node));
  const [advancedOpen, setAdvancedOpen] = useState(Boolean(node));
  const update = (key, value) => setForm((current) => ({ ...current, [key]: value }));
  const isPasswordNode = ["trojan", "hysteria2", "tuic", "anytls", "shadowsocks"].includes(form.outboundType);
  const isUuidNode = ["vless", "vmess", "tuic"].includes(form.outboundType);

  return (
    <Modal
      title={node ? "编辑代理节点" : "新增代理节点"}
      subtitle="基础字段会生成 sing-box outbound；复杂传输、Reality public key 等可在高级 JSON 中保留。"
      onClose={onClose}
    >
      <form className="modal-form" onSubmit={(event) => {
        event.preventDefault();
        onSubmit({
          ...form,
          serverPort: form.serverPort ? Number(form.serverPort) : null,
          rawJson: form.rawJson.trim() || null,
        });
      }}>
        <label className="field">
          <span>名称</span>
          <input value={form.tag} onChange={(event) => update("tag", event.target.value)} placeholder="Proxy Node" autoFocus />
        </label>
        <label className="field">
          <span>协议</span>
          <select value={form.outboundType} onChange={(event) => update("outboundType", event.target.value)}>
            {NODE_TYPES.map(([value, label]) => <option key={value} value={value}>{label}</option>)}
          </select>
        </label>
        <label className="field">
          <span>服务器</span>
          <input value={form.server} onChange={(event) => update("server", event.target.value)} placeholder="example.com" />
        </label>
        <label className="field">
          <span>端口</span>
          <input value={form.serverPort} onChange={(event) => update("serverPort", event.target.value)} inputMode="numeric" placeholder="443" />
        </label>
        {isUuidNode && (
          <label className="field full">
            <span>UUID</span>
            <input value={form.uuid} onChange={(event) => update("uuid", event.target.value)} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" />
          </label>
        )}
        {isPasswordNode && (
          <label className="field full">
            <span>密码 / 密钥</span>
            <input value={form.password} onChange={(event) => update("password", event.target.value)} placeholder="password" />
          </label>
        )}
        {form.outboundType === "shadowsocks" && (
          <label className="field full">
            <span>加密方法</span>
            <input value={form.method} onChange={(event) => update("method", event.target.value)} placeholder="2022-blake3-aes-128-gcm" />
          </label>
        )}
        <label className="field">
          <span>TLS</span>
          <select value={form.security} onChange={(event) => update("security", event.target.value)}>
            <option value="">关闭</option>
            <option value="tls">TLS</option>
            <option value="reality">Reality</option>
          </select>
        </label>
        <label className="field">
          <span>SNI</span>
          <input value={form.sni} onChange={(event) => update("sni", event.target.value)} placeholder="www.apple.com" />
        </label>
        <button type="button" className="inline-toggle full" onClick={() => setAdvancedOpen(!advancedOpen)}>
          <Icon name={advancedOpen ? "expand_less" : "expand_more"} />
          高级 JSON outbound
        </button>
        {advancedOpen && (
          <label className="field full">
            <span>高级 JSON</span>
            <textarea value={form.rawJson} onChange={(event) => update("rawJson", event.target.value)} placeholder='{"type":"vless","tag":"Node","server":"example.com","server_port":443}' />
          </label>
        )}
        <div className="modal-actions">
          <button type="button" className="ghost-button" onClick={onClose}>取消</button>
          <button type="submit" className="primary-button" disabled={busy || !form.tag.trim()}>{busy ? "保存中" : "保存并重启内核"}</button>
        </div>
      </form>
    </Modal>
  );
}

function GroupDialog({ nodes, busy, onClose, onSubmit }) {
  const [tag, setTag] = useState("Auto");
  const [members, setMembers] = useState(() => nodes.map((node) => node.name));
  const [defaultTag, setDefaultTag] = useState(nodes[0]?.name || "direct");
  const toggleMember = (name) => {
    setMembers((current) => current.includes(name)
      ? current.filter((item) => item !== name)
      : [...current, name]);
  };

  return (
    <Modal title="新增策略组" subtitle="策略组会写入 sing-box selector outbound，可在活动页和代理页选择。" onClose={onClose}>
      <form className="modal-form" onSubmit={(event) => {
        event.preventDefault();
        onSubmit({ tag, members, default: defaultTag });
      }}>
        <label className="field full">
          <span>策略组名称</span>
          <input value={tag} onChange={(event) => setTag(event.target.value)} autoFocus />
        </label>
        <div className="field full">
          <span>成员节点</span>
          <div className="member-list">
            {nodes.map((node) => (
              <button type="button" className={members.includes(node.name) ? "member-pill active" : "member-pill"} key={node.name} onClick={() => toggleMember(node.name)}>
                <Icon name={members.includes(node.name) ? "check_circle" : "radio_button_unchecked"} />
                {node.name}
              </button>
            ))}
            <button type="button" className={members.includes("direct") ? "member-pill active" : "member-pill"} onClick={() => toggleMember("direct")}>
              <Icon name={members.includes("direct") ? "check_circle" : "radio_button_unchecked"} />
              direct
            </button>
          </div>
        </div>
        <label className="field full">
          <span>默认节点</span>
          <select value={defaultTag} onChange={(event) => setDefaultTag(event.target.value)}>
            {[...members, "direct"].filter((item, index, arr) => arr.indexOf(item) === index).map((item) => (
              <option key={item} value={item}>{item}</option>
            ))}
          </select>
        </label>
        <div className="modal-actions">
          <button type="button" className="ghost-button" onClick={onClose}>取消</button>
          <button type="submit" className="primary-button" disabled={busy || !tag.trim()}>{busy ? "保存中" : "保存策略组"}</button>
        </div>
      </form>
    </Modal>
  );
}

export function PolicyPage({ selectedProxy, setSelectedProxy, selectedGroup, setSelectedGroup }) {
  const [mode, setMode] = useState("全局代理");
  const [config, setConfig] = useState(null);
  const [busy, setBusy] = useState(false);
  const [dialog, setDialog] = useState(null);
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
  const displayNodes = useMemo(() => {
    if (!isReal || !config.outbounds?.length) return [];
    return config.outbounds
      .filter((outbound) => !["direct", "block", "selector"].includes(outbound.outbound_type))
      .map(normalizeNode);
  }, [isReal, config]);

  const displayGroups = useMemo(() => {
    const groups = isReal && config.policy_groups?.length
      ? config.policy_groups.map((group) => ({
        name: group.name || "Proxy",
        mode: group.mode || "手动选择策略组",
        members: group.members || group.memberTags?.length || 0,
        selected: group.selected || "direct",
      }))
      : [];
    return groups.length ? groups : [{ name: "Proxy", mode: "手动选择策略组", members: displayNodes.length, selected: selectedProxy || "direct" }];
  }, [isReal, config, displayNodes.length, selectedProxy]);

  const applyLatestConfig = (latest) => {
    if (!latest) return;
    setConfig(latest);
    if (latest.mode === "direct") setMode("直接连接");
    if (latest.mode === "global") setMode("全局代理");
    if (latest.mode === "rule") setMode("规则判定");
  };

  const handleModeChange = async (newMode) => {
    setMode(newMode);
    if (!canUseTauri()) return;
    const modeMap = { "直接连接": "direct", "全局代理": "global", "规则判定": "rule" };
    const modeKey = modeMap[newMode];
    if (!modeKey) return;
    try {
      setBusy(true);
      await safeInvoke("set_proxy_mode", { mode: modeKey });
      applyLatestConfig(await safeInvoke("get_singbox_config"));
      addToast(`已切换为 ${newMode} 模式`, "success");
    } catch (e) {
      addToast("模式切换失败: " + e, "error");
    } finally {
      setBusy(false);
    }
  };

  const handleImportSubmit = async (value) => {
    if (!value.trim()) return;
    try {
      setBusy(true);
      const res = await safeInvoke("import_subscription", { url: value.trim() });
      applyLatestConfig(await safeInvoke("get_singbox_config"));
      setDialog(null);
      addToast(`${res.message}: ${res.imported_nodes} 个节点`, "success");
    } catch (e) {
      addToast("导入失败: " + e, "error");
    } finally {
      setBusy(false);
    }
  };

  const handleSaveNode = async (payload) => {
    try {
      setBusy(true);
      const latest = await safeInvoke("save_outbound", { outbound: payload });
      applyLatestConfig(latest);
      setSelectedProxy(payload.tag);
      setDialog(null);
      addToast("节点已写入 sing-box 配置", "success");
    } catch (e) {
      addToast("保存节点失败: " + e, "error");
    } finally {
      setBusy(false);
    }
  };

  const handleDeleteNode = async (node) => {
    if (!window.confirm(`删除节点「${node.name}」？`)) return;
    try {
      setBusy(true);
      const latest = await safeInvoke("delete_outbound", { tag: node.name });
      applyLatestConfig(latest);
      if (selectedProxy === node.name) {
        setSelectedProxy(latest?.policy_groups?.[0]?.selected || "direct");
      }
      addToast("节点已删除并同步策略组", "success");
    } catch (e) {
      addToast("删除节点失败: " + e, "error");
    } finally {
      setBusy(false);
    }
  };

  const handleSaveGroup = async (payload) => {
    try {
      setBusy(true);
      const latest = await safeInvoke("save_selector_group", { group: payload });
      applyLatestConfig(latest);
      setSelectedGroup(payload.tag);
      setDialog(null);
      addToast("策略组已写入 selector outbound", "success");
    } catch (e) {
      addToast("保存策略组失败: " + e, "error");
    } finally {
      setBusy(false);
    }
  };

  const handleSelectProxy = async (nodeName) => {
    setSelectedProxy(nodeName);
    if (!canUseTauri()) return;
    try {
      setBusy(true);
      const latest = await safeInvoke("set_selector_default", { groupTag: selectedGroup || "Proxy", targetTag: nodeName });
      applyLatestConfig(latest);
      addToast(`已将 ${selectedGroup || "Proxy"} 默认节点设为 ${nodeName}`, "success");
    } catch (e) {
      addToast("选择节点失败: " + e, "error");
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
      <p className="muted">{mode === "全局代理" ? "这里只管理 sing-box 出站节点、selector 策略组和配置落盘；实时流量与连接状态集中在活动页查看。" : "当前模式会写入内核路由策略并重启 sing-box。"}</p>

      <div className="policy-head">
        <h2 className="section-title magenta">节点配置</h2>
        <div className="policy-actions">
          <button className="ghost-button" onClick={() => setDialog({ type: "import" })} disabled={busy}>{busy ? "处理中" : "导入订阅"}</button>
          <button className="ghost-button" onClick={handleSpeedTest} disabled={busy}>测试全部</button>
        </div>
      </div>

      {displayNodes.length === 0 && (
        <div className="empty-state compact-empty">
          <p>暂无节点，请导入订阅或手动添加节点。</p>
        </div>
      )}
      <div className="node-grid">
        {displayNodes.map((node) => (
          <div
            className={`node-card node-card--interactive ${selectedProxy === node.name ? "selected" : ""}`}
            key={node.name}
            role="button"
            tabIndex={0}
            onClick={() => handleSelectProxy(node.name)}
            onKeyDown={(event) => event.key === "Enter" && handleSelectProxy(node.name)}
          >
            <div className="node-actions">
              <button type="button" className="mini-action" onClick={(event) => {
                event.stopPropagation();
                setDialog({ type: "node", node });
              }}><Icon name="edit" /></button>
              <button type="button" className="mini-action danger" onClick={(event) => {
                event.stopPropagation();
                handleDeleteNode(node);
              }}><Icon name="delete" /></button>
            </div>
            <span>{node.type}</span>
            <strong>{node.name}</strong>
            <small>{node.server}:{node.serverPort || "-"}</small>
            <em className={node.state}>{node.ping}</em>
          </div>
        ))}
        <button className="node-card add-card" onClick={() => setDialog({ type: "node" })}><Icon name="add" /></button>
      </div>

      <h2 className="section-title cyan">策略组</h2>
      <div className="node-grid group-row">
        {displayGroups.slice(0, 6).map((group) => (
          <button className={`node-card ${selectedGroup === group.name ? "selected" : ""}`} key={group.name} onClick={() => {
            setSelectedGroup(group.name);
            if (group.selected) setSelectedProxy(group.selected);
          }}>
            <span>{group.mode}</span>
            <strong>{group.name}</strong>
            <small>{group.members} 个成员 · 默认 {group.selected}</small>
          </button>
        ))}
        <button className="node-card add-card" onClick={() => setDialog({ type: "group" })}><Icon name="add" /></button>
      </div>

      {dialog?.type === "import" && <ImportDialog busy={busy} onClose={() => setDialog(null)} onSubmit={handleImportSubmit} />}
      {dialog?.type === "node" && <NodeDialog node={dialog.node} busy={busy} onClose={() => setDialog(null)} onSubmit={handleSaveNode} />}
      {dialog?.type === "group" && <GroupDialog nodes={displayNodes} busy={busy} onClose={() => setDialog(null)} onSubmit={handleSaveGroup} />}
    </div>
  );
}

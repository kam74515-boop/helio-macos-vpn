import { useState } from "react";
import { Toggle, Icon, CheckItem } from "../components/ui";
import { useTauriPoll } from "../hooks/tauri";
import { canUseTauri, safeInvoke } from "../utils/tauri";
import { useToast } from "../components/Toast";

export function MitmPage() {
  const [enabled, setEnabled] = useState(false);
  const [showInput, setShowInput] = useState(false);
  const [newHostname, setNewHostname] = useState("");
  const { addToast } = useToast();
  const { data: certStatus, refresh: refreshStatus } = useTauriPoll("get_ca_status", null, 30000);
  const { data: hostnames, refresh: refreshHostnames } = useTauriPoll("get_mitm_hostnames", null, 3000);
  const hasCert = canUseTauri() && certStatus?.has_cert;
  const hostnameList = canUseTauri() && hostnames ? hostnames : [];

  const handleToggle = async (val) => {
    if (!canUseTauri()) return;
    const prev = enabled;
    setEnabled(val);
    try {
      await safeInvoke("set_mitm_enabled", { enabled: val });
      addToast(val ? "HTTPS 解密已开启" : "HTTPS 解密已关闭", "success");
    } catch (e) {
      addToast(`设置失败: ${e}`, "error");
      setEnabled(prev);
    }
  };

  const handleGenerate = async () => {
    if (!canUseTauri()) return;
    try {
      await safeInvoke("generate_ca");
      addToast("CA 证书生成成功", "success");
      refreshStatus();
    } catch (e) {
      addToast(`CA 证书生成失败: ${e}`, "error");
    }
  };

  const handleInstall = async () => {
    if (!canUseTauri()) return;
    if (!hasCert) {
      addToast("请先生成 CA 证书", "info");
      return;
    }
    try {
      await safeInvoke("install_ca");
      addToast("CA 证书已安装到系统", "success");
    } catch (e) {
      addToast(`${e}`, "error", 8000);
    }
  };

  const handleImport = () => {
    addToast("PKCS #12 导入功能开发中", "info");
  };

  const handleExport = async () => {
    if (!canUseTauri()) return;
    if (!hasCert) {
      addToast("请先生成 CA 证书", "info");
      return;
    }
    try {
      const path = await safeInvoke("export_ca", { format: "pem" });
      addToast(`证书已导出到: ${path}`, "success");
    } catch (e) {
      addToast(`导出失败: ${e}`, "error");
    }
  };

  const handleAdd = async () => {
    if (!canUseTauri()) return;
    const trimmed = newHostname.trim();
    if (!trimmed) {
      addToast("主机名不能为空", "info");
      return;
    }
    try {
      await safeInvoke("add_mitm_hostname", { hostname: trimmed });
      addToast("主机名已添加", "success");
      setNewHostname("");
      setShowInput(false);
      refreshHostnames();
    } catch (e) {
      addToast(`添加失败: ${e}`, "error");
    }
  };

  const handleRemove = async (hostname) => {
    if (!canUseTauri()) return;
    try {
      await safeInvoke("remove_mitm_hostname", { hostname });
      addToast("主机名已移除", "success");
      refreshHostnames();
    } catch (e) {
      addToast(`移除失败: ${e}`, "error");
    }
  };

  const handleKeyDown = (e) => {
    if (e.key === "Enter") {
      handleAdd();
    } else if (e.key === "Escape") {
      setShowInput(false);
      setNewHostname("");
    }
  };

  return (
    <div className="page settings-page">
      <div className="headline-toggle"><h1>HTTPS 解密</h1><Toggle checked={enabled} onChange={handleToggle} /></div>
      <p className="muted">通过中间人攻击（MitM, man-in-the-middle）的方式解密 HTTPS 流量。</p>
      <div className="two-col">
        <section>
          <h2 className="section-title orange">CA 证书</h2>
          <div className="cert-row">
            <Icon name="workspace_premium" />
            <div>
              <strong>{hasCert ? "配置中已有 CA 证书" : "配置中没有 CA 证书"}</strong>
              <span>{hasCert ? (certStatus?.is_trusted ? "CA 证书已被系统信任" : "CA 证书未被系统信任") : "CA 证书不被系统信任"}</span>
              {certStatus?.expires_at && <span>过期时间: {certStatus.expires_at}</span>}
              {certStatus?.cert_path && <span className="muted">路径: {certStatus.cert_path}</span>}
            </div>
          </div>
          <div className="form-actions">
            <span>操作:</span>
            <button onClick={handleGenerate}>生成新证书</button>
            <button onClick={handleInstall} disabled={!hasCert}>将证书安装到系统</button>
            <button onClick={handleImport}>从 PKCS #12 文件导入证书</button>
            <button onClick={handleExport} disabled={!hasCert}>为 iOS 模拟器导出证书</button>
          </div>
          <h2 className="section-title green">选项</h2>
          <CheckItem label="进行 MitM 时跳过服务端证书验证" disabled />
          <CheckItem label="自动屏蔽 QUIC" disabled />
          <CheckItem label="MitM over HTTP/2" />
        </section>
        <section>
          <div className="host-box">
            <div>
              MitM 主机名
              <button
                className="icon-button"
                onClick={() => setShowInput((s) => !s)}
                aria-label="添加主机名"
                style={{ background: "transparent" }}
              >
                <Icon name="add_circle" />
              </button>
            </div>
            {showInput && (
              <div style={{ display: "flex", gap: 8, padding: "8px 12px", borderBottom: "1px solid rgba(39,45,54,0.08)" }}>
                <input
                  type="text"
                  value={newHostname}
                  onChange={(e) => setNewHostname(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="输入主机名，如 *.example.com"
                  style={{ flex: 1, minHeight: 32, borderRadius: 8, border: "1px solid rgba(36,42,50,0.12)", padding: "0 10px", fontSize: 13 }}
                  autoFocus
                />
                <button className="primary-button" onClick={handleAdd} disabled={!newHostname.trim()}>
                  添加
                </button>
              </div>
            )}
            <div style={{ padding: "8px 12px", overflow: "auto", maxHeight: "calc(100% - 44px)" }}>
              {hostnameList.length === 0 ? (
                <p className="muted" style={{ textAlign: "center", margin: "12px 0" }}>暂无主机名，点击 + 添加</p>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                  {hostnameList.map((hostname) => (
                    <div key={hostname} className="side-chip" style={{ justifyContent: "space-between" }}>
                      <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{hostname}</span>
                      <button
                        className="mini-action danger"
                        onClick={() => handleRemove(hostname)}
                        aria-label="删除"
                        style={{ flex: "0 0 auto" }}
                      >
                        <Icon name="close" />
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
          <p className="muted">只有该列表中的域名才会被 Helio 进行解密，允许使用通配符。</p>
        </section>
      </div>
    </div>
  );
}

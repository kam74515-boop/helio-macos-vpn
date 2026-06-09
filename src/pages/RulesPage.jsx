import { useState, useMemo } from "react";
import { Icon } from "../components/ui";
import { useTauriData } from "../hooks/tauri";
import { canUseTauri } from "../utils/tauri";

export function RulesPage() {
  const [query, setQuery] = useState("");
  const { data: config, loading } = useTauriData("get_singbox_config");
  const isReal = canUseTauri() && config;
  const displayRules = isReal && config.rules?.length
    ? config.rules.map(r => [r.id, r.rule_type, r.value, r.action, r.hits])
    : [];
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
      {loading && <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>加载中...</div>}
      {!loading && displayRules.length === 0 && (
        <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>暂无规则</div>
      )}
      {displayRules.length > 0 && (
        <div className="rule-table">
          <div className="thead"><span /><span>ID</span><span>类型</span><span>值</span><span>策略</span><span>使用</span></div>
          {visibleRules.map((rule, index) => (
            <div className={`tr ${index === 0 ? "selected" : ""}`} key={`${rule[0]}-${rule[1]}`}>
              <span className="row-dot" /><span>{rule[0]}</span><span>{rule[1]}</span><span>{rule[2]}</span><span>{rule[3]}</span><span>{rule[4]}</span>
            </div>
          ))}
        </div>
      )}
      <div className="table-actions">
        <button>+</button><button>-</button><button>重置计数器</button>
      </div>
    </div>
  );
}

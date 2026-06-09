import { useState, useMemo } from "react";
import { Icon } from "../components/ui";
import { useTauriPoll } from "../hooks/tauri";
import { canUseTauri, safeInvoke } from "../utils/tauri";
import { useToast } from "../components/Toast";

const RULE_TYPES = [
  "DOMAIN",
  "DOMAIN-SUFFIX",
  "DOMAIN-KEYWORD",
  "GEOSITE",
  "GEOIP",
  "IP-CIDR",
  "PROTOCOL",
];

const DEFAULT_ACTIONS = ["direct", "Proxy", "block"];

function RuleModal({ open, title, initial, onClose, onSave }) {
  const [ruleType, setRuleType] = useState(initial?.rule_type || "DOMAIN");
  const [value, setValue] = useState(
    Array.isArray(initial?.value) ? initial.value.join(", ") : initial?.value || ""
  );
  const [action, setAction] = useState(initial?.action || "direct");
  const [customAction, setCustomAction] = useState(
    DEFAULT_ACTIONS.includes(initial?.action) ? "" : initial?.action || ""
  );

  if (!open) return null;

  const finalAction = customAction.trim() || action;

  const handleSave = () => {
    onSave({ rule_type: ruleType, value: value.trim(), action: finalAction });
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal-panel" onClick={(e) => e.stopPropagation()}>
        <div className="modal-head">
          <div>
            <h2>{title}</h2>
            <p>规则将按照从上至下的顺序进行匹配。</p>
          </div>
          <button className="icon-button" onClick={onClose}>
            <Icon name="close" />
          </button>
        </div>
        <div className="modal-form">
          <div className="field">
            <span>规则类型</span>
            <select value={ruleType} onChange={(e) => setRuleType(e.target.value)}>
              {RULE_TYPES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </div>
          <div className="field">
            <span>策略</span>
            <select value={action} onChange={(e) => setAction(e.target.value)}>
              {DEFAULT_ACTIONS.map((a) => (
                <option key={a} value={a}>
                  {a}
                </option>
              ))}
            </select>
          </div>
          <div className="field full">
            <span>值（支持逗号分隔多个值）</span>
            <input
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder="例如: example.com, google.com"
            />
          </div>
          <div className="field full">
            <span>自定义策略（可选，覆盖下拉选择）</span>
            <input
              value={customAction}
              onChange={(e) => setCustomAction(e.target.value)}
              placeholder="输入自定义 outbound tag"
            />
          </div>
          <div className="modal-actions">
            <button className="ghost-button" onClick={onClose}>
              取消
            </button>
            <button className="primary-button" onClick={handleSave}>
              保存
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export function RulesPage() {
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState(null);
  const { data: rules, loading, refresh } = useTauriPoll("get_rules", null, 3000);
  const { addToast } = useToast();

  const [modalOpen, setModalOpen] = useState(false);
  const [editingRule, setEditingRule] = useState(null);

  const isReal = canUseTauri() && rules;
  const displayRules = useMemo(() => {
    if (!isReal || !rules?.length) return [];
    return rules.map((r) => ({
      id: r.id,
      rule_type: r.rule_type,
      value: Array.isArray(r.value) ? r.value.join(", ") : r.value,
      action: r.action,
      hits: r.hits || "0",
    }));
  }, [isReal, rules]);

  const visibleRules = useMemo(
    () =>
      displayRules.filter((rule) =>
        `${rule.rule_type} ${rule.value} ${rule.action} ${rule.hits}`
          .toLowerCase()
          .includes(query.toLowerCase())
      ),
    [query, displayRules]
  );

  const handleAdd = () => {
    setEditingRule(null);
    setModalOpen(true);
  };

  const handleEdit = (rule) => {
    setEditingRule(rule);
    setModalOpen(true);
  };

  const handleModalSave = async (form) => {
    if (!canUseTauri()) return;
    try {
      if (editingRule) {
        await safeInvoke("edit_rule", {
          id: editingRule.id,
          rule_type: form.rule_type,
          value: form.value,
          action: form.action,
        });
        addToast("规则修改成功", "success");
      } else {
        await safeInvoke("add_rule", {
          rule_type: form.rule_type,
          value: form.value,
          action: form.action,
        });
        addToast("规则添加成功", "success");
      }
      refresh();
      setModalOpen(false);
      setEditingRule(null);
    } catch (e) {
      addToast(e?.message || e || "操作失败", "error");
    }
  };

  const handleDelete = async () => {
    if (!canUseTauri()) return;
    if (!selectedId) {
      addToast("请先选择要删除的规则", "info");
      return;
    }
    if (!window.confirm("确定要删除选中的规则吗？")) return;
    try {
      await safeInvoke("delete_rule", { id: selectedId });
      addToast("规则删除成功", "success");
      setSelectedId(null);
      refresh();
    } catch (e) {
      addToast(e?.message || e || "删除失败", "error");
    }
  };

  const handleReset = async () => {
    if (!canUseTauri()) return;
    try {
      await safeInvoke("reset_rule_counters");
      addToast("规则计数器已重置", "success");
      refresh();
    } catch (e) {
      addToast(e?.message || e || "重置失败", "error");
    }
  };

  const moveRule = async (id, direction) => {
    if (!canUseTauri() || !rules) return;
    const ids = rules.map((r) => r.id);
    const idx = ids.indexOf(id);
    if (idx === -1) return;
    const newIdx = direction === "up" ? idx - 1 : idx + 1;
    if (newIdx < 0 || newIdx >= ids.length) return;
    const newIds = [...ids];
    const [moved] = newIds.splice(idx, 1);
    newIds.splice(newIdx, 0, moved);
    try {
      await safeInvoke("reorder_rules", { ids: newIds });
      addToast("排序已更新", "success");
      refresh();
    } catch (e) {
      addToast(e?.message || e || "排序失败", "error");
    }
  };

  return (
    <div className="page card-shell rules-page">
      <div className="split-header">
        <div>
          <h1>规则</h1>
          <p className="muted">规则将按照从上至下的顺序进行测试。</p>
        </div>
        <label className="search">
          <Icon name="search" />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="搜索"
          />
        </label>
      </div>
      {loading && (
        <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>
          加载中...
        </div>
      )}
      {!loading && displayRules.length === 0 && (
        <div style={{ padding: 24, color: "var(--muted)", textAlign: "center" }}>
          暂无规则
        </div>
      )}
      {displayRules.length > 0 && (
        <div className="rule-table">
          <div className="thead">
            <span />
            <span>ID</span>
            <span>类型</span>
            <span>值</span>
            <span>策略</span>
            <span>使用</span>
            <span />
          </div>
          {visibleRules.map((rule, index) => (
            <div
              className={`tr ${selectedId === rule.id ? "selected" : ""}`}
              key={rule.id}
              onClick={() => setSelectedId(rule.id)}
            >
              <span className="row-dot" />
              <span>{rule.id}</span>
              <span>{rule.rule_type}</span>
              <span>{rule.value}</span>
              <span>{rule.action}</span>
              <span>{rule.hits}</span>
              <span className="row-actions">
                <button
                  className="mini-action"
                  title="上移"
                  disabled={index === 0}
                  onClick={(e) => {
                    e.stopPropagation();
                    moveRule(rule.id, "up");
                  }}
                >
                  <Icon name="expand_less" />
                </button>
                <button
                  className="mini-action"
                  title="下移"
                  disabled={index === visibleRules.length - 1}
                  onClick={(e) => {
                    e.stopPropagation();
                    moveRule(rule.id, "down");
                  }}
                >
                  <Icon name="expand_more" />
                </button>
                <button
                  className="mini-action"
                  title="编辑"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleEdit(rule);
                  }}
                >
                  <Icon name="edit" />
                </button>
              </span>
            </div>
          ))}
        </div>
      )}
      <div className="table-actions">
        <button onClick={handleAdd} disabled={!canUseTauri()}>
          +
        </button>
        <button onClick={handleDelete} disabled={!canUseTauri()}>
          -
        </button>
        <button onClick={handleReset} disabled={!canUseTauri()}>
          重置计数器
        </button>
      </div>

      <RuleModal
        open={modalOpen}
        title={editingRule ? "编辑规则" : "添加规则"}
        initial={editingRule}
        onClose={() => {
          setModalOpen(false);
          setEditingRule(null);
        }}
        onSave={handleModalSave}
      />
    </div>
  );
}

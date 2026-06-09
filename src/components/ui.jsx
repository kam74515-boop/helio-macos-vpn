import { useState } from "react";
import * as MuiIcons from "@mui/icons-material";
import { iconMap, navGroups, processes } from "../data/mock";

export function Icon({ name, className = "" }) {
  const Component = iconMap[name] || MuiIcons.RadioButtonUncheckedRounded;
  return <Component className={`icon ${className}`} fontSize="inherit" aria-hidden="true" />;
}

export function Toggle({ checked, onChange }) {
  return (
    <button className={`toggle ${checked ? "is-on" : ""}`} onClick={() => onChange(!checked)} aria-label="toggle">
      <span />
    </button>
  );
}

export function Segmented({ value, options, onChange }) {
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

export function MenuSelect({ label, value, options, onChange }) {
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

export function Sidebar({ active, onNavigate }) {
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

export function StatusPills({ systemProxy, enhanced, setSystemProxy, setEnhanced }) {
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

export function MetricCard({ label, value, unit, accent = "blue", children }) {
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

export function MiniLine({ color = "blue", values = [40, 40, 35, 36, 62, 70, 38, 32, 52, 40] }) {
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

export function OverviewCard({ title, text, status, checked, onChange, color }) {
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

export function ProcessRank({ compact = false, selectedApp = "", onSelect, items }) {
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

export function SplitPage({ title, control, children }) {
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

export function CheckItem({ label, disabled = false }) {
  const [checked, setChecked] = useState(false);
  return (
    <button className={`check-item ${checked ? "checked" : ""}`} disabled={disabled} onClick={() => setChecked(!checked)}>
      <Icon name={checked ? "check_box" : "check_box_outline_blank"} />{label}
    </button>
  );
}

import * as MuiIcons from "@mui/icons-material";

export const navGroups = [
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

export const nodes = [
  { type: "AnyTLS", name: "anytls-VM-0-11-ubuntu", ping: "失败", state: "error" },
  { type: "Hysteria 2", name: "hy2-VM-0-11-ubuntu", ping: "209 ms", state: "ok" },
  { type: "TUIC v5", name: "tu5-VM-0-11-ubuntu", ping: "216 ms", state: "selected" },
  { type: "VLESS Reality", name: "vl-reality-VM-0-11-ubuntu", ping: "195 ms", state: "ok" },
  { type: "VMess WS", name: "vm-ws-VM-0-11-ubuntu", ping: "205 ms", state: "ok" },
];

export const policyGroups = [
  { name: "Proxy", mode: "手动选择策略组", members: 5 },
  { name: "Auto", mode: "延迟最低", members: 4 },
  { name: "Fallback", mode: "故障转移", members: 4 },
  { name: "Streaming", mode: "流媒体", members: 3 },
];

export const processes = [
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

export const requests = [
  ["2316", "12:04:07", "Cursor Helper", "活跃", "tu5-VM-0-11...", "2 KB", "4 KB", "9 s", "HTTPS", "api3.cursor.sh:443"],
  ["2315", "12:04:07", "syspolicyd", "活跃", "tu5-VM-0-11...", "2 KB", "8 KB", "10 s", "HTTPS", "api.apple-cloudkit.com:443"],
  ["2314", "12:04:06", "WeChat", "已完成", "tu5-VM-0-11...", "725 B", "347 B", "1 s", "POST", "http://183.60.8.150/mmtls/750028"],
  ["2313", "12:04:06", "Cursor Helper", "已完成", "tu5-VM-0-11...", "4 KB", "2 KB", "2 s", "HTTPS", "api3.cursor.sh:443"],
  ["2312", "12:04:05", "xray", "已完成", "tu5-VM-0-11...", "6 KB", "10 KB", "3 s", "TCP", "ipel.zheshe002.com:60004"],
  ["2311", "12:04:05", "xray", "已完成", "tu5-VM-0-11...", "6 KB", "11 KB", "3 s", "TCP", "ipel.zheshe002.com:60004"],
  ["2310", "12:04:05", "TRAE SOLO CN", "已完成", "tu5-VM-0-11...", "44 KB", "1 KB", "3 s", "HTTPS", "mon.zijieapi.com:443"],
  ["2309", "12:04:04", "DingTalk", "活跃", "tu5-VM-0-11...", "2 KB", "5 KB", "14 s", "HTTPS", "h-adashx.dingtalkapps.com:443"],
];

export const rules = [
  ["0", "RULE-SET", "SYSTEM (no-resolve)", "DIRECT", "0"],
  ["1", "DOMAIN-SUFFIX", "apple.com", "DIRECT", "42"],
  ["2", "DOMAIN-KEYWORD", "cursor", "Proxy", "81"],
  ["3", "GEOIP", "CN", "DIRECT", "23"],
  ["4", "FINAL", "", "Proxy", "161"],
];

export const trafficBars = [4, 5, 4, 4, 5, 4, 4, 5, 6, 38, 65, 92, 6, 5, 5, 4, 5, 5, 4, 4, 5, 4, 5, 6];

export const iconMap = {
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

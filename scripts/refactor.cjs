const fs = require('fs');
const path = require('path');

const content = fs.readFileSync('src/App.jsx', 'utf-8');
const lines = content.split('\n');

function getLines(start, end) {
  // 1-indexed to 0-indexed, end inclusive
  return lines.slice(start - 1, end).join('\n') + '\n';
}

function ensureDir(dir) {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
}

ensureDir('src/pages');
ensureDir('src/components');
ensureDir('src/hooks');
ensureDir('src/data');
ensureDir('src/utils');

// src/data/mock.js
let mockJs = `import * as MuiIcons from "@mui/icons-material";\n\n`;
mockJs += getLines(6, 126).replace(/const /g, 'export const ');
fs.writeFileSync('src/data/mock.js', mockJs);

// src/utils/tauri.js
let tauriJs = `import { invoke } from "@tauri-apps/api/core";\n\n`;
tauriJs += getLines(133, 140).replace(/function /g, 'export function ');
fs.writeFileSync('src/utils/tauri.js', tauriJs);

// src/hooks/tauri.js
let hooksJs = `import { useState, useEffect } from "react";\n`;
hooksJs += `import { invoke } from "@tauri-apps/api/core";\n`;
hooksJs += `import { canUseTauri } from "../utils/tauri";\n\n`;
hooksJs += getLines(142, 171).replace(/function /g, 'export function ');
fs.writeFileSync('src/hooks/tauri.js', hooksJs);

// src/components/ui.jsx
let uiJsx = `import * as MuiIcons from "@mui/icons-material";\n`;
uiJsx += `import { iconMap, navGroups, processes } from "../data/mock";\n\n`;
uiJsx += getLines(128, 131).replace(/function /, 'export function ') + '\n';
uiJsx += getLines(173, 205).replace(/function /g, 'export function ') + '\n';
uiJsx += getLines(207, 249).replace(/function /, 'export function ') + '\n';
uiJsx += getLines(251, 298).replace(/function /g, 'export function ') + '\n';
uiJsx += getLines(421, 453).replace(/function /g, 'export function ') + '\n';
uiJsx += getLines(499, 509).replace(/function /, 'export function ') + '\n';
uiJsx += getLines(710, 717).replace(/function /, 'export function ') + '\n';
fs.writeFileSync('src/components/ui.jsx', uiJsx);

// src/pages/ActivityPage.jsx
let activityJsx = `import { useState } from "react";\n`;
activityJsx += `import { StatusPills, MetricCard, MiniLine, MenuSelect, Icon, ProcessRank, Segmented } from "../components/ui";\n`;
activityJsx += `import { useTauriPoll } from "../hooks/tauri";\n`;
activityJsx += `import { canUseTauri, safeInvoke } from "../utils/tauri";\n`;
activityJsx += `import { policyGroups, nodes, trafficBars } from "../data/mock";\n\n`;
activityJsx += getLines(300, 388).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/ActivityPage.jsx', activityJsx);

// src/pages/OverviewPage.jsx
let overviewJsx = `import { OverviewCard } from "../components/ui";\n\n`;
overviewJsx += getLines(390, 419).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/OverviewPage.jsx', overviewJsx);

// src/pages/ProcessesPage.jsx
let processesJsx = `import { useState, useEffect } from "react";\n`;
processesJsx += `import { SplitPage, ProcessRank, Toggle, Icon, MiniLine } from "../components/ui";\n`;
processesJsx += `import { useTauriPoll } from "../hooks/tauri";\n`;
processesJsx += `import { canUseTauri } from "../utils/tauri";\n`;
processesJsx += `import { processes } from "../data/mock";\n\n`;
processesJsx += getLines(511, 542).replace(/function /, 'export function ') + '\n';
processesJsx += getLines(455, 477).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/ProcessesPage.jsx', processesJsx);

// src/pages/DevicesPage.jsx
let devicesJsx = `import { useState } from "react";\n`;
devicesJsx += `import { SplitPage, Toggle, Icon } from "../components/ui";\n\n`;
devicesJsx += getLines(544, 561).replace(/function /, 'export function ') + '\n';
devicesJsx += getLines(479, 497).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/DevicesPage.jsx', devicesJsx);

// src/pages/PolicyPage.jsx
let policyJsx = `import { useState } from "react";\n`;
policyJsx += `import { Segmented, Icon } from "../components/ui";\n`;
policyJsx += `import { useTauriData } from "../hooks/tauri";\n`;
policyJsx += `import { canUseTauri } from "../utils/tauri";\n`;
policyJsx += `import { nodes, policyGroups } from "../data/mock";\n\n`;
policyJsx += getLines(563, 605).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/PolicyPage.jsx', policyJsx);

// src/pages/RulesPage.jsx
let rulesJsx = `import { useState, useMemo } from "react";\n`;
rulesJsx += `import { Icon } from "../components/ui";\n`;
rulesJsx += `import { useTauriData } from "../hooks/tauri";\n`;
rulesJsx += `import { canUseTauri } from "../utils/tauri";\n`;
rulesJsx += `import { rules } from "../data/mock";\n\n`;
rulesJsx += getLines(607, 637).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/RulesPage.jsx', rulesJsx);

// src/pages/CapturePage.jsx
let captureJsx = `import { useState } from "react";\n`;
captureJsx += `import { Segmented, Icon } from "../components/ui";\n`;
captureJsx += `import { useTauriPoll } from "../hooks/tauri";\n`;
captureJsx += `import { canUseTauri } from "../utils/tauri";\n`;
captureJsx += `import { requests, processes } from "../data/mock";\n\n`;
captureJsx += getLines(639, 683).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/CapturePage.jsx', captureJsx);

// src/pages/MitmPage.jsx
let mitmJsx = `import { useState } from "react";\n`;
mitmJsx += `import { Toggle, Icon, CheckItem } from "../components/ui";\n\n`;
mitmJsx += getLines(685, 708).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/MitmPage.jsx', mitmJsx);

// src/pages/RewritePage.jsx
let rewriteJsx = `import { useState } from "react";\n`;
rewriteJsx += `import { Toggle } from "../components/ui";\n\n`;
rewriteJsx += getLines(719, 742).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/RewritePage.jsx', rewriteJsx);

// src/pages/MorePage.jsx
let moreJsx = `import { Icon } from "../components/ui";\n\n`;
moreJsx += getLines(744, 768).replace(/function /, 'export function ');
fs.writeFileSync('src/pages/MorePage.jsx', moreJsx);

// src/App.jsx
let appJsx = `import { useState, useEffect } from "react";\n`;
appJsx += `import { listen } from "@tauri-apps/api/event";\n`;
appJsx += `import { safeInvoke, canUseTauri } from "./utils/tauri";\n`;
appJsx += `import { Sidebar } from "./components/ui";\n`;
appJsx += `import { ActivityPage } from "./pages/ActivityPage";\n`;
appJsx += `import { OverviewPage } from "./pages/OverviewPage";\n`;
appJsx += `import { ProcessesPage } from "./pages/ProcessesPage";\n`;
appJsx += `import { DevicesPage } from "./pages/DevicesPage";\n`;
appJsx += `import { PolicyPage } from "./pages/PolicyPage";\n`;
appJsx += `import { RulesPage } from "./pages/RulesPage";\n`;
appJsx += `import { CapturePage } from "./pages/CapturePage";\n`;
appJsx += `import { MitmPage } from "./pages/MitmPage";\n`;
appJsx += `import { RewritePage } from "./pages/RewritePage";\n`;
appJsx += `import { MorePage } from "./pages/MorePage";\n\n`;
appJsx += getLines(770, 827);
fs.writeFileSync('src/App.jsx', appJsx);

console.log("Done");

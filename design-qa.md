source visual truth path: user-provided Surge macOS screenshots in conversation
implementation evidence paths:
- /Users/karl/apps/vpn/surge-material-prototype/qa-browser-results.json
- /Users/karl/apps/vpn/surge-material-prototype/qa-policy-page.png
viewport: 1440 x 980
state: all primary sidebar pages checked through the in-app Browser

full-view comparison evidence:
- Activity, overview, processes, devices, proxy configuration, rules, capture, HTTPS decryption, rewrite, and settings pages were opened through the sidebar in the Web preview.
- Browser QA reported no page-level horizontal overflow, no page-level vertical overflow, and no clipped visible content blocks on all ten primary pages.
- Activity page keeps the Surge-like macOS sidebar, status pills, network summary, proxy/group selectors, metric cards, traffic chart, and traffic totals in one window.
- Proxy page is now configuration-only: outbound mode, node configuration, and policy groups. Real-time traffic and connection monitoring stay on Activity/Capture.
- Typography was reduced from oversized/heavy display styling to a lighter tool UI scale: nav 19px/700, H1 about 36px/800, body and controls mostly 14-18px/600-650.

focused region comparison evidence:
- Sidebar: selected state, icon alignment, grouped labels, and bottom actions checked on all pages.
- Activity controls: policy group and proxy selectors are visible and sync with proxy configuration state.
- Proxy configuration grid: all node and policy-group cards are visible in one window at 1440 x 980.
- Process page: right-side detail panel updates when selecting a different process.
- Rules page: search filters the table to the matching rule.
- Capture page: capture button toggles from start to stop.
- MITM and Rewrite pages: headline toggles change state.

findings:
- No P0/P1/P2 issues remain for the requested prototype changes at the verified viewport.

patches made since previous QA pass:
- Corrected the page class assignment so compact proxy styles apply to the proxy configuration page, not overview.
- Added global selected proxy and selected policy group state.
- Added activity-page policy group and proxy selectors.
- Renamed the proxy surface to "代理配置" and kept it configuration-focused.
- Reduced typography scale and font weight across navigation, titles, cards, controls, tables, and settings.
- Compressed all primary pages to fit inside the current app window.
- Added process and device detail panels so client pages are no longer empty placeholders.
- Added structured browser QA evidence at qa-browser-results.json.

build and verification:
- npm run build: passed.
- Browser layout QA at 1440 x 980: passed for all ten primary pages.
- Interaction QA: selector sync, rule search, capture toggle, process detail selection, MITM toggle, and rewrite toggle passed.

follow-up polish:
- P3: capture screenshots intermittently timed out in the in-app browser runtime; structured browser evidence was saved instead.
- P3: future SwiftUI implementation should replace MUI icons with SF Symbols or a local vector icon set.

final result: passed

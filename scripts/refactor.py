import os
import re

def main():
    with open('src/App.jsx', 'r', encoding='utf-8') as f:
        content = f.read()

    # Create directories if not exist
    os.makedirs('src/pages', exist_ok=True)
    os.makedirs('src/components', exist_ok=True)
    os.makedirs('src/hooks', exist_ok=True)
    os.makedirs('src/data', exist_ok=True)
    os.makedirs('src/utils', exist_ok=True)

    # Function to extract a block of code until an empty line or specific pattern
    def extract_between(start_str, end_str=None, include_end=False):
        start_idx = content.find(start_str)
        if start_idx == -1: return ""
        if end_str:
            end_idx = content.find(end_str, start_idx + len(start_str))
            if end_idx == -1: return content[start_idx:]
            return content[start_idx:end_idx + (len(end_str) if include_end else 0)]
        else:
            return ""

    def extract_function(func_name):
        # find 'function func_name' or 'export function func_name'
        match = re.search(r'(export\s+)?function\s+' + func_name + r'\s*\(.*?\)\s*\{', content, re.DOTALL)
        if not match: return ""
        start_idx = match.start()
        # Find matching brace
        brace_count = 0
        in_string = False
        string_char = ''
        for i in range(start_idx, len(content)):
            char = content[i]
            if char in ("'", '"', '`') and content[i-1] != '\\':
                if not in_string:
                    in_string = True
                    string_char = char
                elif string_char == char:
                    in_string = False
            
            if not in_string:
                if char == '{':
                    brace_count += 1
                elif char == '}':
                    brace_count -= 1
                    if brace_count == 0:
                        return content[start_idx:i+1]
        return ""
    
    def extract_const(const_name):
        match = re.search(r'const\s+' + const_name + r'\s*=\s*(?:\[|\{)', content)
        if not match: return ""
        start_idx = match.start()
        # Find matching bracket/brace
        open_char = content[match.end()-1]
        close_char = ']' if open_char == '[' else '}'
        brace_count = 0
        in_string = False
        string_char = ''
        for i in range(match.end()-1, len(content)):
            char = content[i]
            if char in ("'", '"', '`') and content[i-1] != '\\':
                if not in_string:
                    in_string = True
                    string_char = char
                elif string_char == char:
                    in_string = False
            
            if not in_string:
                if char == open_char:
                    brace_count += 1
                elif char == close_char:
                    brace_count -= 1
                    if brace_count == 0:
                        # get trailing semicolon if exists
                        end = i + 1
                        if end < len(content) and content[end] == ';': end += 1
                        return content[start_idx:end]
        return ""

    # 1. Data
    data_vars = ['navGroups', 'nodes', 'policyGroups', 'processes', 'requests', 'rules', 'trafficBars', 'iconMap']
    data_content = 'import * as MuiIcons from "@mui/icons-material";\n\n'
    for var in data_vars:
        c = extract_const(var)
        if c: data_content += f"export {c}\n\n"
    with open('src/data/mock.js', 'w', encoding='utf-8') as f: f.write(data_content.strip() + '\n')

    # 2. Utils/Hooks
    utils_content = 'import { invoke } from "@tauri-apps/api/core";\nimport { useState, useEffect } from "react";\n\n'
    utils_content += extract_function('canUseTauri') + '\n\n'
    utils_content += extract_function('safeInvoke') + '\n\n'
    utils_content += f"export {extract_function('useTauriPoll')}\n\n".replace('function useTauriPoll', 'function useTauriPoll')
    utils_content += f"export {extract_function('useTauriData')}\n\n".replace('function useTauriData', 'function useTauriData')
    # add exports for canUseTauri and safeInvoke
    utils_content = utils_content.replace('function canUseTauri', 'export function canUseTauri')
    utils_content = utils_content.replace('function safeInvoke', 'export function safeInvoke')
    with open('src/utils/tauri.js', 'w', encoding='utf-8') as f: f.write(utils_content.strip() + '\n')

    # 3. Components
    components = ['Icon', 'Toggle', 'Segmented', 'MenuSelect', 'Sidebar', 'StatusPills', 'MetricCard', 'MiniLine', 'OverviewCard', 'ProcessRank', 'SplitPage', 'CheckItem']
    comp_content = 'import { iconMap, navGroups, processes } from "../data/mock";\n\n'
    for comp in components:
        comp_content += f"export {extract_function(comp)}\n\n"
    with open('src/components/ui.jsx', 'w', encoding='utf-8') as f: f.write(comp_content.strip() + '\n')

    # 4. Pages
    pages = {
        'ActivityPage': ['ActivityPage'],
        'OverviewPage': ['OverviewPage'],
        'ProcessesPage': ['ProcessesPage', 'ProcessDetail'],
        'DevicesPage': ['DevicesPage', 'DeviceDetail'],
        'PolicyPage': ['PolicyPage'],
        'RulesPage': ['RulesPage'],
        'CapturePage': ['CapturePage'],
        'MitmPage': ['MitmPage'],
        'RewritePage': ['RewritePage'],
        'MorePage': ['MorePage']
    }
    for page_file, funcs in pages.items():
        page_content = f'import {{ useState, useEffect, useMemo }} from "react";\n'
        page_content += f'import {{ safeInvoke, useTauriPoll, useTauriData, canUseTauri }} from "../utils/tauri";\n'
        page_content += f'import {{ nodes, policyGroups, processes, requests, rules, trafficBars }} from "../data/mock";\n'
        page_content += f'import {{ Icon, Toggle, Segmented, MenuSelect, StatusPills, MetricCard, MiniLine, OverviewCard, ProcessRank, SplitPage, CheckItem }} from "../components/ui";\n\n'
        for func in funcs:
            if func == page_file:
                page_content += f"export {extract_function(func)}\n\n"
            else:
                page_content += f"{extract_function(func)}\n\n"
        with open(f'src/pages/{page_file}.jsx', 'w', encoding='utf-8') as f: f.write(page_content.strip() + '\n')

    # 5. App.jsx
    app_content = '''import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { canUseTauri, safeInvoke } from "./utils/tauri";
import { Sidebar } from "./components/ui";
import { ActivityPage } from "./pages/ActivityPage";
import { OverviewPage } from "./pages/OverviewPage";
import { ProcessesPage } from "./pages/ProcessesPage";
import { DevicesPage } from "./pages/DevicesPage";
import { PolicyPage } from "./pages/PolicyPage";
import { RulesPage } from "./pages/RulesPage";
import { CapturePage } from "./pages/CapturePage";
import { MitmPage } from "./pages/MitmPage";
import { RewritePage } from "./pages/RewritePage";
import { MorePage } from "./pages/MorePage";

'''
    app_content += extract_function('App') + '\n'
    with open('src/App.jsx', 'w', encoding='utf-8') as f: f.write(app_content)

if __name__ == '__main__':
    main()

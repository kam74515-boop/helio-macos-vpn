import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { safeInvoke, canUseTauri } from "./utils/tauri";
import { Sidebar } from "./components/ui";
import { ToastProvider, useToast } from "./components/Toast";
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

export function App() {
  return (
    <ToastProvider>
      <AppContent />
    </ToastProvider>
  );
}

function AppContent() {
  const [active, setActive] = useState("activity");
  const [systemProxy, setSystemProxyState] = useState(false);
  const [enhanced, setEnhanced] = useState(false);
  const [selectedProxy, setSelectedProxy] = useState("tu5-VM-0-11-ubuntu");
  const [selectedGroup, setSelectedGroup] = useState("Proxy");
  const { addToast } = useToast();

  const setSystemProxy = async (enable) => {
    try {
      if (enable) {
        await safeInvoke("start_engine", { config: null });
        await safeInvoke("start_monitoring");
      }
      await safeInvoke("set_system_proxy", { enable });
      if (!enable) {
        await safeInvoke("stop_monitoring");
        await safeInvoke("stop_engine");
      }
      const state = await safeInvoke("get_proxy_state");
      setSystemProxyState(Boolean(state?.system_proxy_enabled));
    } catch (e) {
      console.error("Failed to set system proxy", e);
      setSystemProxyState(false);
      addToast("代理设置失败: " + e, "error");
    }
  };

  useEffect(() => {
    if (!canUseTauri()) return undefined;
    // Sync initial proxy state
    safeInvoke("get_proxy_state").then(state => {
      setSystemProxyState(Boolean(state?.system_proxy_enabled));
    }).catch(console.error);
    safeInvoke("start_engine", { config: null }).catch(console.error);
    // Start background monitoring
    safeInvoke("start_monitoring").catch(console.error);
    const unlisten = listen("traffic-update", (event) => {
      // Real-time traffic events supplement polling
      console.log("Traffic update:", event.payload);
    }).catch(() => {});
    return () => {
      safeInvoke("stop_engine").catch(console.error);
      safeInvoke("stop_monitoring").catch(console.error);
      unlisten.then?.(fn => fn()).catch(() => {});
    };
  }, []);

  const pageProps = { systemProxy, enhanced, setSystemProxy, setEnhanced, selectedProxy, setSelectedProxy, selectedGroup, setSelectedGroup };
  const pages = {
    activity: <ActivityPage {...pageProps} />,
    overview: <OverviewPage {...pageProps} />,
    processes: <ProcessesPage />,
    devices: <DevicesPage />,
    policy: <PolicyPage {...pageProps} />,
    rules: <RulesPage />,
    capture: <CapturePage />,
    mitm: <MitmPage />,
    rewrite: <RewritePage />,
    more: <MorePage />,
  };

  return (
    <ToastProvider>
      <div className={`app-shell ${canUseTauri() ? "is-tauri" : "is-web"}`}>
        <Sidebar active={active} onNavigate={setActive} />
        <main className="content">{pages[active]}</main>
      </div>
    </ToastProvider>
  );
}

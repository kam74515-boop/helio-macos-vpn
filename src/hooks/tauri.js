import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { canUseTauri } from "../utils/tauri";

export function useTauriPoll(command, args = null, interval = 3000, defaultValue = null) {
  const [data, setData] = useState(defaultValue);
  const [loading, setLoading] = useState(canUseTauri());
  useEffect(() => {
    if (!canUseTauri()) { setLoading(false); return; }
    let active = true;
    const fetch = async () => {
      try {
        const result = await invoke(command, args || {});
        if (active) { setData(result); setLoading(false); }
      } catch (e) { console.error(`${command}:`, e); if (active) setLoading(false); }
    };
    fetch();
    const timer = setInterval(fetch, interval);
    return () => { active = false; clearInterval(timer); };
  }, [command, interval]);
  return { data, loading };
}

export function useTauriData(command, defaultValue = null) {
  const [data, setData] = useState(defaultValue);
  const [loading, setLoading] = useState(canUseTauri());
  useEffect(() => {
    if (!canUseTauri()) { setLoading(false); return; }
    let active = true;
    invoke(command).then(r => { if (active) { setData(r); setLoading(false); } }).catch(e => { console.error(e); if (active) setLoading(false); });
    return () => { active = false; };
  }, [command]);
  return { data, loading };
}

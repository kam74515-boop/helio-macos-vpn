import { invoke } from "@tauri-apps/api/core";

export function canUseTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

export async function safeInvoke(command, args) {
  if (!canUseTauri()) return null;
  return invoke(command, args || {});
}

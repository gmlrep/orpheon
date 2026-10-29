import { getCurrentWindow } from "@tauri-apps/api/window";
import { useUiStore } from "../stores/ui";

export function useFullscreen() {
  const ui = useUiStore();

  async function toggleFullscreen() {
    const win = getCurrentWindow();
    const next = !(await win.isFullscreen());
    await win.setFullscreen(next);
    ui.setFullscreen(next);
    return next;
  }

  async function syncFromOs() {
    try {
      const win = getCurrentWindow();
      ui.setFullscreen(await win.isFullscreen());
    } catch {
      /* ignore */
    }
  }

  return { toggleFullscreen, syncFromOs };
}

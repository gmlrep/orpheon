import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  cursorPosition,
  monitorFromPoint,
  getCurrentWindow,
} from "@tauri-apps/api/window";

export const POPUP_LABELS = {
  about: "about",
  shortcuts: "shortcuts",
  customize: "customize",
  context: "context-menu",
} as const;

export const CTX_ACTION_EVENT = "orpheon://context-action";

const CONTEXT_W = 300;
const CONTEXT_H = 860;

async function focusOrCreate(
  label: string,
  options: ConstructorParameters<typeof WebviewWindow>[1],
): Promise<WebviewWindow> {
  const existing = await WebviewWindow.getByLabel(label);
  if (existing) {
    try {
      await existing.setFocus();
      return existing;
    } catch {
      /* recreate below */
    }
  }
  return new WebviewWindow(label, options);
}

export async function openAboutWindow(): Promise<void> {
  await focusOrCreate(POPUP_LABELS.about, {
    url: "/?popup=about",
    title: "About Orpheon",
    width: 560,
    height: 440,
    minWidth: 420,
    minHeight: 320,
    center: true,
    decorations: false,
    resizable: false,
    focus: true,
    skipTaskbar: false,
    visible: true,
  });
}

export async function openShortcutsWindow(): Promise<void> {
  await focusOrCreate(POPUP_LABELS.shortcuts, {
    url: "/?popup=shortcuts",
    title: "Keyboard Shortcuts",
    width: 720,
    height: 640,
    minWidth: 560,
    minHeight: 420,
    center: true,
    decorations: false,
    resizable: true,
    focus: true,
    visible: true,
  });
}

export async function openCustomizeWindow(): Promise<void> {
  await focusOrCreate(POPUP_LABELS.customize, {
    url: "/?popup=customize",
    title: "Customize Player",
    width: 720,
    height: 520,
    minWidth: 560,
    minHeight: 400,
    center: true,
    decorations: false,
    resizable: true,
    focus: true,
    visible: true,
  });
}

export async function closeContextMenuWindow(): Promise<void> {
  const existing = await WebviewWindow.getByLabel(POPUP_LABELS.context);
  if (existing) {
    try {
      await existing.close();
    } catch {
      /* ignore */
    }
  }
}

export async function openContextMenuWindow(state: {
  playing: boolean;
  active: boolean;
  isVideo: boolean;
}): Promise<void> {
  await closeContextMenuWindow();

  const physical = await cursorPosition();
  const mon = await monitorFromPoint(physical.x, physical.y);
  const scale = mon?.scaleFactor ?? (await getCurrentWindow().scaleFactor());
  let x = physical.x / scale;
  let y = physical.y / scale;

  if (mon) {
    const left = mon.position.x / scale;
    const top = mon.position.y / scale;
    const right = left + mon.size.width / scale;
    const bottom = top + mon.size.height / scale;
    x = Math.min(Math.max(left + 4, x), right - CONTEXT_W - 4);
    y = Math.min(Math.max(top + 4, y), bottom - CONTEXT_H - 4);
  }

  const qs = new URLSearchParams({
    popup: "context",
    playing: state.playing ? "1" : "0",
    active: state.active ? "1" : "0",
    isVideo: state.isVideo ? "1" : "0",
  });

  const win = new WebviewWindow(POPUP_LABELS.context, {
    url: `/?${qs.toString()}`,
    title: "Menu",
    width: CONTEXT_W,
    height: CONTEXT_H,
    x: Math.round(x),
    y: Math.round(y),
    decorations: false,
    resizable: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    focus: true,
    visible: true,
    shadow: true,
    transparent: false,
  });

  win.once("tauri://error", (e) => {
    console.error("context menu window failed", e);
  });
}

import { readJSON, writeJSON } from "$lib/prefs";

/**
 * Compare two dotted version strings. Returns >0 if `a` is newer than `b`,
 * <0 if older, 0 if equal. Tolerates a leading "v" and differing segment counts
 * ("1.2" === "1.2.0"). Any pre-release suffix is ignored — Korio only publishes
 * plain x.y.z tags.
 */
export function compareVersions(a: string, b: string): number {
  const parts = (v: string) =>
    v.trim().replace(/^v/i, "").split(/[.\-+]/).map((p) => Number.parseInt(p, 10));
  const pa = parts(a);
  const pb = parts(b);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const x = Number.isFinite(pa[i]) ? pa[i] : 0;
    const y = Number.isFinite(pb[i]) ? pb[i] : 0;
    if (x !== y) return x - y;
  }
  return 0;
}

export interface UpdatePrefs {
  /** Version the user pressed "Skip this version" on. */
  skipped: string | null;
  /** Unix ms before which we stay quiet after "Remind me later". */
  remindUntil: number;
  /** Check on launch. */
  auto: boolean;
}

export const DEFAULT_PREFS: UpdatePrefs = { skipped: null, remindUntil: 0, auto: true };
const KEY = "korio.updates";

export function loadPrefs(): UpdatePrefs {
  const raw = readJSON<Partial<UpdatePrefs>>(KEY, {});
  return {
    skipped: typeof raw?.skipped === "string" ? raw.skipped : null,
    remindUntil: typeof raw?.remindUntil === "number" ? raw.remindUntil : 0,
    auto: raw?.auto !== false,
  };
}
export function savePrefs(p: UpdatePrefs) { writeJSON(KEY, p); }

/**
 * Should a *background* check surface this update? Manual checks bypass this —
 * asking for an update is consent to be told about it, even a skipped one.
 */
export function shouldOfferUpdate(
  current: string, latest: string, prefs: UpdatePrefs, nowMs: number,
): boolean {
  if (compareVersions(latest, current) <= 0) return false;
  if (prefs.skipped !== null && compareVersions(prefs.skipped, latest) >= 0) return false;
  if (prefs.remindUntil > nowMs) return false;
  return true;
}

/** Download progress as a 0–1 fraction; null while the total size is unknown. */
export function progressFraction(downloaded: number, total: number | null): number | null {
  if (!total || total <= 0) return null;
  return Math.min(1, Math.max(0, downloaded / total));
}

/**
 * Tidy release notes for the dialog: strips the markdown noise GitHub release
 * bodies carry (headings, bullets, bold/code marks, links) and drops the
 * download/footer section, leaving readable lines.
 */
export function formatNotes(body: string, maxLines = 14): string[] {
  const lines: string[] = [];
  for (const raw of (body || "").split(/\r?\n/)) {
    let l = raw.trim();
    if (!l) continue;
    if (/^-{3,}$/.test(l)) break;                       // horizontal rule = start of the footer
    if (/^#+\s*(⬇️|:arrow_down:)?\s*Download/i.test(l)) break;
    l = l.replace(/^#+\s*/, "");                        // heading marks
    l = l.replace(/^[-*+]\s+/, "• ");                   // bullets
    l = l.replace(/\[([^\]]+)\]\([^)]*\)/g, "$1");      // links → text
    l = l.replace(/\*\*([^*]+)\*\*/g, "$1");            // bold
    l = l.replace(/`([^`]+)`/g, "$1");                  // code
    if (l) lines.push(l);
    if (lines.length >= maxLines) { lines.push("…"); break; }
  }
  return lines;
}

// ---- Runtime (main window only; no-ops without Tauri) ----
import { writable, get } from "svelte/store";
import { browser } from "$app/environment";
import { isMainWindow } from "$lib/sync";

export type UpdateStatus =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "uptodate"; checkedAt: number }
  | { state: "available"; version: string; date: string | null; notes: string[] }
  | { state: "downloading"; version: string; downloaded: number; total: number | null }
  | { state: "ready"; version: string }
  | { state: "error"; message: string };

export const updateStatus = writable<UpdateStatus>({ state: "idle" });
/** True when the update dialog should be on screen. */
export const updateDialogOpen = writable(false);
export const currentVersion = writable("");

/** The plugin's Update handle for the offered version, kept out of the store. */
let handle: { version: string; downloadAndInstall: (cb?: (e: any) => void) => Promise<void> } | null = null;
let started = false;

async function appVersion(): Promise<string> {
  try {
    const { getVersion } = await import("@tauri-apps/api/app");
    return await getVersion();
  } catch { return ""; }
}

/**
 * Check GitHub for a newer signed release.
 * `manual` surfaces the result either way (including "you're up to date") and
 * ignores a previous Skip/Remind choice.
 */
export async function checkForUpdates(manual: boolean): Promise<void> {
  if (!browser || !isMainWindow()) return;
  updateStatus.set({ state: "checking" });
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    const current = await appVersion();
    currentVersion.set(current);

    if (!update) {
      handle = null;
      updateStatus.set({ state: "uptodate", checkedAt: Date.now() });
      return;
    }

    handle = update as any;
    const status: UpdateStatus = {
      state: "available",
      version: update.version,
      date: update.date ?? null,
      notes: formatNotes(update.body ?? ""),
    };

    if (!manual && !shouldOfferUpdate(current, update.version, loadPrefs(), Date.now())) {
      // A newer build exists but the user asked not to be nagged about it yet.
      updateStatus.set(status);
      return;
    }
    updateStatus.set(status);
    updateDialogOpen.set(true);
  } catch (e) {
    handle = null;
    updateStatus.set({ state: "error", message: String(e) });
    if (manual) updateDialogOpen.set(true);
  }
}

/** Download, install and relaunch into the new version. */
export async function installUpdate(): Promise<void> {
  if (!handle) return;
  const version = handle.version;
  let downloaded = 0;
  let total: number | null = null;
  updateStatus.set({ state: "downloading", version, downloaded, total });
  try {
    await handle.downloadAndInstall((e: any) => {
      if (e.event === "Started") {
        total = e.data?.contentLength ?? null;
        updateStatus.set({ state: "downloading", version, downloaded, total });
      } else if (e.event === "Progress") {
        downloaded += e.data?.chunkLength ?? 0;
        updateStatus.set({ state: "downloading", version, downloaded, total });
      } else if (e.event === "Finished") {
        updateStatus.set({ state: "ready", version });
      }
    });
    // Installing this version clears any earlier "skip"/"remind" choice.
    savePrefs({ ...loadPrefs(), skipped: null, remindUntil: 0 });
    updateStatus.set({ state: "ready", version });
    const { relaunch } = await import("@tauri-apps/plugin-process");
    await relaunch();
  } catch (e) {
    updateStatus.set({ state: "error", message: String(e) });
  }
}

export function skipVersion(version: string) {
  savePrefs({ ...loadPrefs(), skipped: version });
  updateDialogOpen.set(false);
}

/** Stay quiet for `hours`, then offer this version again. */
export function remindLater(hours = 24) {
  savePrefs({ ...loadPrefs(), remindUntil: Date.now() + hours * 3_600_000 });
  updateDialogOpen.set(false);
}

export function setAutoCheck(auto: boolean) { savePrefs({ ...loadPrefs(), auto }); }

/** Background check shortly after launch, then once a day while Korio runs. */
export function initUpdates() {
  if (!browser || started) return;
  started = true;
  void appVersion().then((v) => currentVersion.set(v));
  const run = () => {
    if (!loadPrefs().auto) return;
    if (get(updateDialogOpen)) return;
    void checkForUpdates(false);
  };
  setTimeout(run, 8_000);
  setInterval(run, 24 * 3_600_000);
}

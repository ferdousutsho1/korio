<div align="center">

<img src="docs/assets/logo.svg" alt="Korio" width="140" />

# Korio

### Track your focus. Understand your time. Work smarter, not harder.

A private, 100% offline focus-tracking and productivity app for Windows — lightweight, fast, and built to respect your data.

![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D6)
![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB)
![Frontend](https://img.shields.io/badge/UI-SvelteKit%20%2B%20Svelte%205-FF3E00)
![Storage](https://img.shields.io/badge/storage-SQLite%20(local)-003B57)
![Offline](https://img.shields.io/badge/privacy-100%25%20offline-2F6E4F)
![License](https://img.shields.io/badge/license-MIT-blue)

</div>

---

## About the name

**Korio** is inspired by the Japanese word *kōritsu* (効率), meaning **"efficiency"** or **"productivity."**
The name distills the essence of *kōritsu* into something short, modern, and memorable.

As a focus-tracking app, Korio stands for the pursuit of efficient work, focused effort, and continuous
improvement — helping you understand how you spend your time, optimize your workflow, and achieve more
with less friction. The idea behind the name: productivity isn't about working *harder*, it's about
working *smarter*.

---

## What is Korio?

Korio quietly times how long you *actively* use a chosen set of apps and websites, turns that into clear
visualizations, and gives you gentle tools to stay on track — daily limits, a focus score, reminders, and a
small suite of built-in utilities. Everything lives in a local SQLite file on your machine: **no accounts,
no cloud, no telemetry, no network calls.**

It's built on **Tauri**, so it ships as a tiny native executable with a modern web UI — typically idling
around ~50–90 MB of RAM instead of the hundreds you'd expect from an Electron app.

---

## Features

### 📊 Tracking & insight
- **Automatic focus tracking** — times active use of the apps you choose (counts only while an app is focused *and* you're active; idle time is paused).
- **Categories & Focus Score** — group apps into color-coded categories (Productivity, Entertainment, custom…), each marked productive / neutral / distracting; your Focus Score reflects the balance.
- **Customizable dashboard** — app focus breakdown, site focus breakdown, category pie, focus score, "tracking now," and compact stats/to-do/notes cards you can toggle and drag to reorder.
- **App Stats** — date-range picker, per-app usage bars, a labelled daily-activity heatmap, and an hour-gridded 24-hour timeline ribbon with a per-app key.
- **Auto-tracking** — optionally let Korio add any non-system app you keep focused past a threshold to the Watchlist by itself.

### 🌐 Website tracking (optional)
- **Per-site time tracking** via a companion Chrome/Edge extension over a **local-only** channel (127.0.0.1) — domains only, never full URLs.
- **Per-site daily limits** with warn-or-block enforcement, usage meters, categories, renaming, and a "hide sites under X minutes" filter.
- **Keeps working offline** — closing the browser only pauses new site data; your existing stats stay on screen behind a small "not connected" notice.

### ⏱️ Limits & tools
- **Per-app daily limits** — set a cap and choose *Warn* or *Auto-close* when you go over. The limit alert surfaces over other apps with a (loopable) sound.
- **PIN-locked escape hatches** — optionally require your PIN to *Snooze* or *Ignore* a limit, so blowing past it takes a deliberate act.
- **Built-in tools** — Stopwatch, Countdown Timer, World Clock, and a configurable **Pomodoro** timer with selectable, optionally-looping notification sounds.

### ✅ Productivity suite
- **To-do list** — a per-date list (view and edit previous days).
- **Reminders** — scheduled nudges with repeat rules (once / daily / weekdays / weekly), plus snooze-for-an-hour or a time you pick.
- **Sticky Notes** — color-coded notes with due dates, freely resizable.
- **End of day digest** — a short visual recap of the day (top apps, top sites, where the time went, limits blown through) with commentary. The tab glows when a new one is ready.
- **Global quick-capture** — a system-wide hotkey to jot a task or note without switching windows.
- **Quick-launch** — start any watched app straight from Korio.

### 🔒 Privacy, data & polish
- **100% offline & local** — all data in a local SQLite database (`%APPDATA%\Korio\`); no telemetry, no network. Your data is kept across app updates.
- **Your data is yours** — export to CSV/JSON, plus full backup & restore.
- **Rename anything** — give apps and sites friendlier names; tracking keeps following the underlying exe/domain.
- **Theming** — light/dark, custom accent color, background tints, and a customizable sidebar.
- **Lightweight & portable** — copy the folder to any Windows 10/11 PC and run; no installer required.

---

## Screenshots

_Coming soon._

---

## Tech stack

| Layer | Technology |
|-------|------------|
| Shell / backend | [Tauri 2](https://tauri.app) (Rust) |
| Frontend | [SvelteKit](https://kit.svelte.dev) + Svelte 5 (runes) |
| Storage | SQLite via `rusqlite` (bundled) |
| Platform | Windows 10/11 (64-bit) |

---

## Getting started

### Option A — Download and run (portable)
1. Download the latest `Korio-portable-*.zip` from the [Releases](https://github.com/ferdousutsho1/korio/releases) page.
2. Extract it and double-click **`Korio.exe`** — no installation required.
3. Korio starts in the **system tray**; click the tray icon to open the window.

> Requires the Microsoft Edge **WebView2 Runtime** (preinstalled on Windows 11 and current Windows 10).
> Your data lives in `%APPDATA%\Korio\` and is preserved across updates.

### Option B — Build from source
Prerequisites: [Node.js](https://nodejs.org), [Rust](https://rustup.rs), and the Tauri prerequisites for Windows (MSVC build tools + WebView2).

```bash
git clone https://github.com/ferdousutsho1/korio.git
cd korio
npm install
npm run tauri dev     # run in development
npm run tauri build   # produce a release build + installers
```

### Browser extension (optional)
Only needed if you want **website** tracking:
1. In Korio: **Settings → Track browser sites → On**, and copy the pairing token.
2. In Chrome/Edge: open `chrome://extensions`, enable **Developer mode**, click **Load unpacked**, and select the `extension/` folder.
3. Open the extension's Options page and paste the token. Korio's status flips to **Connected**.

---

## Privacy

Korio is **local-first by design**. There are no accounts, no analytics, and no outbound network requests.
All tracking data is stored in a single SQLite file on your device. The optional browser extension talks
to Korio only over loopback (127.0.0.1), is token-guarded, and transmits **domains only — never full URLs**.

---

## Author

**Korio** is designed and developed by **Ferdous Utsho**.

- GitHub: [@ferdousutsho1](https://github.com/ferdousutsho1)

Issues and pull requests are welcome.

---

## License

Released under the **MIT License** — see [`LICENSE`](LICENSE) for details.

---

<div align="center">
<sub>Built with Tauri • Made by Ferdous Utsho</sub>
</div>

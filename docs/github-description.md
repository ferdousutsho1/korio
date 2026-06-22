<!--
  Korio — GitHub project description / README draft
  Author: Ferdous Utsho
  Drop the logo into the <img> below and this is ready to use as README.md.
-->

<div align="center">

<img src="docs/assets/logo.svg" alt="Korio" width="340" />

### Track your focus. Understand your time. Work smarter, not harder.

A private, 100% offline focus-tracking and productivity app for Windows — lightweight, fast, and built to respect your data.

<!-- Badges: swap USER/REPO for your GitHub path once published. -->
![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D6)
![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB)
![Frontend](https://img.shields.io/badge/UI-SvelteKit%20%2B%20Svelte%205-FF3E00)
![Storage](https://img.shields.io/badge/storage-SQLite%20(local)-003B57)
![Offline](https://img.shields.io/badge/privacy-100%25%20offline-2F6E4F)
<!-- ![License](https://img.shields.io/badge/license-MIT-blue) -->

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
visualizations, and gives you gentle tools to stay on track — daily limits, a focus score, goals, and a
small suite of built-in utilities. Everything lives in a local SQLite file on your machine: **no accounts,
no cloud, no telemetry, no network calls.**

It's built on **Tauri**, so it ships as a tiny native executable with a modern web UI — typically idling
around ~50–90 MB of RAM instead of the hundreds you'd expect from an Electron app.

---

## Features

### 📊 Tracking & insight
- **Automatic focus tracking** — times active use of the apps you choose (counts only while an app is focused *and* you're active; idle time is paused).
- **Categories & Focus Score** — group apps into color-coded categories (Productivity, Entertainment, custom…), each marked productive / neutral / distracting; your Focus Score reflects the balance.
- **Customizable dashboard** — focus breakdown, category pie, focus score, "tracking now," and compact stats/tasks/notes/goals cards you can toggle and drag to reorder.
- **Stats** — date-range picker, per-app usage bars, a GitHub-style daily heatmap, and a 24-hour timeline ribbon.

### 🌐 Website tracking (optional)
- **Per-site time tracking** via a companion Chrome/Edge extension over a **local-only** channel (127.0.0.1) — domains only, never full URLs.
- **Per-site daily limits** with warn-or-block enforcement.

### ⏱️ Limits & tools
- **Per-app daily limits** — set a cap and choose *Warn* or *Auto-close* when you go over.
- **Built-in tools** — Stopwatch, Countdown Timer, World Clock, and a configurable **Pomodoro** timer with **selectable notification sounds**.

### ✅ Productivity suite
- **Tasks** — a per-date task list (view and edit previous days).
- **Sticky Notes** — color-coded notes with due dates.
- **Goals & streaks** — set time goals (productive / per-app / total) and track current and best streaks.
- **End-of-day digest** — an optional once-a-day summary notification.
- **Global quick-capture** — a system-wide hotkey to jot a task or note without switching windows.
- **Quick-launch** — start any watched app straight from Korio.

### 🔒 Privacy, data & polish
- **100% offline & local** — all data in a local SQLite database; no telemetry, no network.
- **Your data is yours** — export to CSV/JSON, plus full backup & restore.
- **App lock** — optional PIN to open Korio.
- **Theming** — light/dark, custom accent color, background tints, and a customizable sidebar.
- **Lightweight & portable** — copy the folder to any Windows 10/11 PC and run; no installer required.

---

## Screenshots

<!-- Add screenshots once you have them, e.g.: -->
<!-- <p align="center"><img src="docs/assets/dashboard.png" width="800" alt="Korio dashboard" /></p> -->

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
1. Download the latest `Korio-portable-*.zip` from the [Releases](#) page.
2. Extract it and double-click **`Korio.exe`** — no installation required.
3. Korio starts in the **system tray**; click the tray icon to open the window.

> Requires the Microsoft Edge **WebView2 Runtime** (preinstalled on Windows 11 and current Windows 10).

### Option B — Build from source
Prerequisites: [Node.js](https://nodejs.org), [Rust](https://rustup.rs), and the Tauri prerequisites for Windows (MSVC build tools + WebView2).

```bash
git clone https://github.com/<your-username>/korio.git
cd korio
npm install
npm run tauri dev     # run in development
npm run tauri build   # produce a release build + installers
```

### Browser extension (optional)
Only needed if you want **website** tracking:
1. In Korio: **Settings → Track browser sites → On**, and copy the pairing token.
2. In Chrome/Edge: open `chrome://extensions`, enable **Developer mode**, click **Load unpacked**, and select the `extension/` (a.k.a. `korio-extension`) folder.
3. Open the extension's Options page and paste the token. Korio's status flips to **Connected**.

---

## Privacy

Korio is **local-first by design**. There are no accounts, no analytics, and no outbound network requests.
All tracking data is stored in a single SQLite file on your device. The optional browser extension talks
to Korio only over loopback (127.0.0.1), is token-guarded, and transmits **domains only — never full URLs**.

---

## Author

**Korio** is designed and developed by **Ferdous Utsho**.

- GitHub: [@your-username](https://github.com/<your-username>)  <!-- add your handle -->
- Contact: anjumferdous1@gmail.com  <!-- remove this line if you'd rather not list it publicly -->

If you build something with Korio or have ideas, issues and pull requests are welcome.

---

## License

<!-- Pick a license and add a LICENSE file. MIT is a common, permissive choice for apps like this. -->
This project is released under the **MIT License** — see [`LICENSE`](LICENSE) for details.
_(Update this section if you choose a different license.)_

---

<div align="center">
<sub>Built with Tauri • Made by Ferdous Utsho</sub>
</div>

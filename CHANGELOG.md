# Changelog

All notable changes to Korio are documented here. This project follows
[Semantic Versioning](https://semver.org/).

## v0.2.0 — 2026-06-24

Focus of this release: smarter app/site limits, a clearer Watchlist, and UI polish.

### Added
- **Watchlist usage meter.** Each app with a daily limit now shows a progress
  meter below its name. When you go over the limit, the meter turns red and a
  warning line appears (e.g. "Over limit — 12m past 30m").
- **5-second close countdown.** Apps/sites set to *Auto-close* no longer close
  instantly. You get a 5-second warning dialog with **Snooze 10 min**,
  **Ignore today**, and **Close now** — and it closes automatically only if you
  do nothing.
- **Seconds on the countdown timer.** The Tools countdown timer now accepts
  minutes *and* seconds.
- **Sites connection gate.** The Sites tab now detects whether the browser
  extension is connected. When it isn't, it shows a step-by-step setup guide
  with an **Open Settings** shortcut and a **Refresh** button; once connected,
  your tracked sites appear.

### Changed
- **Limit dialog colors are now fixed and theme-independent:** Snooze is
  neutral, Ignore is amber, Close is red — your accent color no longer
  overrides them. Auto-close chips are red throughout.
- **Limits re-arm when you change them.** Previously a limit only warned once
  per day; if you changed it after going over, nothing happened. Now changing a
  limit (or snoozing) re-arms the warning, and every configured app/site warns
  independently.
- **Cleaner header.** Removed the redundant page title from the top bar (the
  active tab is already shown in the sidebar). The light/dark toggle moved into
  the sidebar footer next to the version.
- Watchlist now states that limits are daily and `0 = off`.

### Fixed
- The app/site limit warning that would only ever fire once per day.
- Sites view no longer flickers to the "not connected" guide on a transient
  status check.
- Centered the sidebar light/dark toggle glyph.

## v0.1.0 — 2026-06-23

Initial public release.

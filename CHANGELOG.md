# Changelog

All notable changes to Korio are documented here. This project follows
[Semantic Versioning](https://semver.org/).

## v0.3.1 — 2026-08-07

### Fixed
- **The end-of-day digest showed the wrong day.** It always summarised *today*,
  so setting the digest time to a small hour (say 12 AM on 4 August) produced a
  recap of the handful of minutes that had elapsed since midnight — in practice,
  a blank page — instead of the day that had just finished.

### Changed
- **The digest time setting is gone.** A digest now always covers the day that
  just ended and appears at local midnight; the sidebar tab glows until you open
  it. The unread marker is keyed on the day being summarised, so reading one at
  23:00 no longer marks the following night's as already read.
- The notification toggle stays, and now fires for each new digest rather than at
  a set hour. If Korio wasn't running at midnight it sends on the next launch.
- The digest header names the day it covers, and a day with nothing tracked says
  so plainly instead of rendering empty cards.

Existing installs drop the now-unused `digest_time` setting on first launch.

## v0.3.0 — 2026-08-06

Focus of this release: in-app updates, reminders, an end-of-day digest, site
parity with apps, and a serious pass over readability.

### Added
- **In-app updates.** Settings → **Updates** has a **Check for updates** button and
  a "check automatically" toggle (on by default: shortly after launch, then once
  a day). When a newer version exists, a dialog shows the version, release date
  and what's new, with three choices:
  - **Skip this version** — stays quiet about that version, but still tells you
    about anything newer.
  - **Remind me later** — quiet for 24 hours.
  - **Update & restart** — downloads with a progress bar, installs, and relaunches
    Korio on the new version.

  Updates are fetched from Korio's official GitHub releases and their signature is
  verified against a key baked into the app before anything is installed. Manual
  checks always show a result, including "you're on the latest version", and
  ignore an earlier Skip/Remind choice.

  > **Note:** v0.3.0 is the first release that can update itself. Earlier
  > versions have to be upgraded once by hand.
- **Reminders tab.** Schedule nudges with a date and time, repeat them (once /
  every day / weekdays / every week), and handle them when they fire: **Close
  reminder**, **Remind in 1 hour**, or **Choose time…** for a custom delay.
  Overdue reminders are flagged in the list, and each row has quick **+1h** /
  **+1d** buttons.
- **End of day digest tab.** A short visual recap of the day — total focus time,
  focus score, site time, to-dos done, top apps and sites, where the time went
  (category donut), and any limits you blew through — with some commentary about
  how it went. The sidebar tab **glows until you open today's digest**. The time
  it's generated is configurable in Settings.
- **Site usage meters.** Sites with a daily limit now show a fill meter under the
  name, turning red with an "Over limit — Xm past Ym" line, matching the Watchlist.
- **Categories for sites.** Sites can be categorised just like apps, and their
  time now feeds the dashboard's Category breakdown.
- **Site focus breakdown card** for the dashboard (off by default; enable it in
  ⚙ Customize).
- **Renaming.** Apps and sites can both be renamed to whatever you like. Tracking
  keeps following the underlying exe name / domain, so history is never broken.
  Clearing a site's name restores the raw domain.
- **PIN-locked Snooze & Ignore.** Optionally require your Korio PIN to snooze or
  ignore a limit that's been reached. Closing is never gated, and backing out of
  the PIN prompt re-arms the auto-close countdown.
- **Auto-tracking.** Optionally have Korio add any non-system app you keep
  focused past a threshold (default 10 minutes/day) to the Watchlist by itself.
  Windows shell processes and anything under the Windows directory are skipped.

### Changed
- **Sites keep working when the browser is closed.** Previously the whole tab was
  replaced by a "browser not connected" setup screen, hiding your history. Now
  your stats always show, and a small dismissible notice at the bottom explains
  that live tracking is paused, with Settings and Retry shortcuts.
- **Daily activity heatmap is readable.** Bigger cells, weekday and month labels,
  a legend with real durations instead of a vague Less→More ramp, a highlighted
  selected day, and an active-days/busiest-day summary.
- **Timeline is readable.** Taller ribbon with 2-hour gridlines and labelled hour
  ticks, hoverable/focusable session blocks with a live readout, and a per-app
  key with totals for the selected day.
- **"Tasks" is now "To-do list"** and **"Stats" is now "App Stats"**.

### Removed
- **App lock.** The PIN-to-open-Korio screen is gone. The PIN itself remains and
  is now used to protect limit snooze/ignore instead.
- **Goals tab.** Removed along with its dashboard card.

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

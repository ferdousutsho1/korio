# Korio Site Reporter (Chromium extension)

Reports your active browser tab's domain to the local Korio app so it can track
and limit time per site. Talks **only** to `http://127.0.0.1` — nothing leaves your machine.

## Install (Chrome / Edge)
1. In Korio: Settings → Browser tracking → enable, then copy the pairing token.
2. Open `chrome://extensions` (or `edge://extensions`).
3. Turn on **Developer mode**.
4. Click **Load unpacked** and select this `extension/` folder.
5. Open the extension's **Options**, paste the token (and port if you changed it), Save.

Korio's Settings should now show **Connected**.

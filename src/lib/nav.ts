import { writable } from "svelte/store";

/** A one-shot request to navigate the main window to a view id (e.g. from a notification click). */
export const navIntent = writable<string | null>(null);

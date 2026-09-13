import { create } from "zustand";

export type ThemePreference = "dark" | "light" | "auto";
export type ResolvedTheme = "dark" | "light";

const STORAGE_KEY = "blenderbase.theme";
const DARK_QUERY = "(prefers-color-scheme: dark)";

interface IThemeStore {
    /** What the user chose in Settings. */
    preference: ThemePreference,
    /** What is actually applied right now (auto resolves against the OS). */
    resolved: ResolvedTheme,
    setPreference: (preference: ThemePreference) => void,
    /** Reads the stored preference, applies it, and follows OS changes while on auto. */
    init: () => void,
}

const readStored = (): ThemePreference => {
    try {
        const v = localStorage.getItem(STORAGE_KEY);
        return v === "light" || v === "dark" || v === "auto" ? v : "dark";
    } catch {
        return "dark";
    }
}

const systemTheme = (): ResolvedTheme =>
    typeof window !== "undefined" && window.matchMedia && window.matchMedia(DARK_QUERY).matches ? "dark" : "light";

const resolve = (preference: ThemePreference): ResolvedTheme =>
    preference === "auto" ? systemTheme() : preference;

/** Stamps the resolved theme on the document so the stylesheet tokens switch. */
const apply = (resolved: ResolvedTheme) => {
    document.documentElement.dataset.theme = resolved;
    document.documentElement.style.colorScheme = resolved;
}

let mediaListenerAttached = false;

export const useThemeStore = create<IThemeStore>((set, get) => ({
    preference: "dark",
    resolved: "dark",
    setPreference: (preference) => {
        try {
            localStorage.setItem(STORAGE_KEY, preference);
        } catch (e) {
            console.error(e);
        }
        const resolved = resolve(preference);
        apply(resolved);
        set({ preference, resolved });
    },
    init: () => {
        const preference = readStored();
        const resolved = resolve(preference);
        apply(resolved);
        set({ preference, resolved });
        if (!mediaListenerAttached && window.matchMedia) {
            mediaListenerAttached = true;
            window.matchMedia(DARK_QUERY).addEventListener("change", () => {
                if (get().preference === "auto") {
                    const next = systemTheme();
                    apply(next);
                    set({ resolved: next });
                }
            });
        }
    },
}));

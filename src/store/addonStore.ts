import { create } from "zustand";
import { IAddon } from "../models";
import { AddonService } from "../services/addonService";
import { postStatus, postStatusError } from "./statusStore";

interface IAddonStore {
    /** Addons of the Blender version they were last loaded for. */
    addons: IAddon[],
    loadedForBlenderVersionId: string | null,
    /** The version most recently asked for; responses for any other version are discarded. */
    requestedBlenderVersionId: string | null,
    /** True while Blender is being run headlessly to read or change addons. */
    isBusy: boolean,
    lastError: string | null,
    /**
     * Blender versions launched since their addons were last read. Addons installed or removed
     * inside Blender's Preferences only show up after a re-read, so these are read again
     * instead of served from the cache: on the next window focus for the selected version,
     * or when the version is selected later.
     */
    launchedSinceReadIds: string[],
    noteBlenderLaunched: (blenderVersionId: string) => void,
    loadAddons: (blenderVersionId: string) => Promise<void>,
    refreshAddons: (blenderVersionId: string) => Promise<void>,
    toggleAddon: (id: string, isEnabled: boolean) => Promise<void>,
    installAddon: (blenderVersionId: string, filePath: string) => Promise<void>,
    symlinkAddon: (blenderVersionId: string, directoryPath: string) => Promise<void>,
    deleteAddon: (blenderVersionId: string, id: string) => Promise<void>,
    revealAddon: (id: string) => Promise<void>,
    clear: () => void,
}

const addonService = new AddonService();

/** Loads in flight, by Blender version id, so a repeated request shares the first one. */
const inflightLoads = new Map<string, Promise<void>>();

const errorText = (e: unknown): string => (typeof e === "string" ? e : (e as Error)?.message ?? String(e));

export const useAddonStore = create<IAddonStore>((set, get) => ({
    addons: [],
    loadedForBlenderVersionId: null,
    requestedBlenderVersionId: null,
    isBusy: false,
    lastError: null,
    launchedSinceReadIds: [],

    noteBlenderLaunched: (blenderVersionId) => set((state) => ({
        launchedSinceReadIds: state.launchedSinceReadIds.includes(blenderVersionId)
            ? state.launchedSinceReadIds
            : [...state.launchedSinceReadIds, blenderVersionId],
    })),

    /** Shows the cached list immediately, and scans with Blender when nothing is cached yet
     *  or the version has been launched since the last read. */
    async loadAddons(blenderVersionId) {
        // A newer request supersedes any earlier one: its response is discarded, and a busy
        // flag it left behind no longer applies.
        set({ requestedBlenderVersionId: blenderVersionId, isBusy: false });
        const existing = inflightLoads.get(blenderVersionId);
        if (existing) {
            return existing; // Same version asked for twice (e.g. StrictMode): share the request.
        }
        const isCurrent = () => get().requestedBlenderVersionId === blenderVersionId;
        const run = (async () => {
            try {
                const cached = await addonService.fetchAddons(blenderVersionId);
                if (!isCurrent()) {
                    return;
                }
                set({ addons: cached, loadedForBlenderVersionId: blenderVersionId, lastError: null });
                if (cached.length === 0 || get().launchedSinceReadIds.includes(blenderVersionId)) {
                    await get().refreshAddons(blenderVersionId);
                }
            } catch (e) {
                console.error(e);
                if (isCurrent()) {
                    set({ addons: [], loadedForBlenderVersionId: blenderVersionId, lastError: errorText(e) });
                }
            }
        })();
        inflightLoads.set(blenderVersionId, run);
        try {
            await run;
        } finally {
            inflightLoads.delete(blenderVersionId);
        }
    },

    async refreshAddons(blenderVersionId) {
        set((state) => ({
            requestedBlenderVersionId: blenderVersionId,
            isBusy: true,
            lastError: null,
            launchedSinceReadIds: state.launchedSinceReadIds.filter((id) => id !== blenderVersionId),
        }));
        const isCurrent = () => get().requestedBlenderVersionId === blenderVersionId;
        postStatus("Reading addons from Blender…", true);
        try {
            const addons = await addonService.refreshAddons(blenderVersionId);
            if (!isCurrent()) {
                return;
            }
            set({ addons, loadedForBlenderVersionId: blenderVersionId });
            postStatus(`${addons.length} addons read from Blender`);
        } catch (e) {
            console.error(e);
            if (isCurrent()) {
                // Counts as loaded (with nothing) so the panel leaves "Loading…".
                set({ loadedForBlenderVersionId: blenderVersionId, lastError: errorText(e) });
            }
            postStatusError(`Reading addons failed: ${errorText(e)}`);
        } finally {
            if (isCurrent()) {
                set({ isBusy: false });
            }
        }
    },

    async toggleAddon(id, isEnabled) {
        // Optimistic: flip the switch right away, revert if Blender refuses.
        const previous = get().addons;
        set({ addons: previous.map((a) => (a.id === id ? { ...a, is_enabled: isEnabled } : a)), isBusy: true, lastError: null });
        const label = previous.find((a) => a.id === id)?.name ?? "addon";
        postStatus(`${isEnabled ? "Enabling" : "Disabling"} ${label}…`, true);
        try {
            const updated = await addonService.toggleAddon(id, isEnabled);
            set({ addons: get().addons.map((a) => (a.id === id ? updated : a)) });
            postStatus(`${label} ${isEnabled ? "enabled" : "disabled"}`);
        } catch (e) {
            console.error(e);
            set({ addons: previous, lastError: errorText(e) });
            postStatusError(`${isEnabled ? "Enabling" : "Disabling"} ${label} failed: ${errorText(e)}`);
        } finally {
            set({ isBusy: false });
        }
    },

    async installAddon(blenderVersionId, filePath) {
        set({ isBusy: true, lastError: null });
        const fileName = filePath.split(/[\\/]/).pop() ?? filePath;
        postStatus(`Installing ${fileName}…`, true);
        try {
            const addons = await addonService.installAddon(blenderVersionId, filePath);
            set({ addons, loadedForBlenderVersionId: blenderVersionId });
            postStatus(`Installed ${fileName}`);
        } catch (e) {
            console.error(e);
            set({ lastError: errorText(e) });
            postStatusError(`Installing ${fileName} failed: ${errorText(e)}`);
        } finally {
            set({ isBusy: false });
        }
    },

    async symlinkAddon(blenderVersionId, directoryPath) {
        set({ isBusy: true, lastError: null });
        const dirName = directoryPath.split(/[\\/]/).filter((p) => p.length > 0).pop() ?? directoryPath;
        postStatus(`Symlinking ${dirName}…`, true);
        try {
            const addons = await addonService.symlinkAddon(blenderVersionId, directoryPath);
            set({ addons, loadedForBlenderVersionId: blenderVersionId });
            postStatus(`Symlinked ${dirName}`);
        } catch (e) {
            console.error(e);
            set({ lastError: errorText(e) });
            postStatusError(`Symlinking ${dirName} failed: ${errorText(e)}`);
            // The link may exist even when enabling failed; show the current state.
            try {
                set({ addons: await addonService.fetchAddons(blenderVersionId) });
            } catch (inner) {
                console.error(inner);
            }
        } finally {
            set({ isBusy: false });
        }
    },

    async deleteAddon(blenderVersionId, id) {
        set({ isBusy: true, lastError: null });
        const label = get().addons.find((a) => a.id === id)?.name ?? "addon";
        postStatus(`Removing ${label}…`, true);
        try {
            const addons = await addonService.deleteAddon(id);
            set({ addons, loadedForBlenderVersionId: blenderVersionId });
            postStatus(`Removed ${label}`);
        } catch (e) {
            console.error(e);
            set({ lastError: errorText(e) });
            postStatusError(`Removing ${label} failed: ${errorText(e)}`);
        } finally {
            set({ isBusy: false });
        }
    },

    async revealAddon(id) {
        try {
            await addonService.revealAddon(id);
        } catch (e) {
            console.error(e);
            set({ lastError: errorText(e) });
        }
    },

    clear: () => set({ addons: [], loadedForBlenderVersionId: null, requestedBlenderVersionId: null, isBusy: false, lastError: null }),
}));

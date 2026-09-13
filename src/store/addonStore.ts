import { create } from "zustand";
import { IAddon } from "../models";
import { AddonService } from "../services/addonService";
import { postStatus, postStatusError } from "./statusStore";

interface IAddonStore {
    /** Addons of the Blender version they were last loaded for. */
    addons: IAddon[],
    loadedForBlenderVersionId: string | null,
    /** True while Blender is being run headlessly to read or change addons. */
    isBusy: boolean,
    lastError: string | null,
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

const errorText = (e: unknown): string => (typeof e === "string" ? e : (e as Error)?.message ?? String(e));

export const useAddonStore = create<IAddonStore>((set, get) => ({
    addons: [],
    loadedForBlenderVersionId: null,
    isBusy: false,
    lastError: null,

    /** Shows the cached list immediately, and scans with Blender when nothing is cached yet. */
    async loadAddons(blenderVersionId) {
        try {
            const cached = await addonService.fetchAddons(blenderVersionId);
            set({ addons: cached, loadedForBlenderVersionId: blenderVersionId, lastError: null });
            if (cached.length === 0) {
                await get().refreshAddons(blenderVersionId);
            }
        } catch (e) {
            console.error(e);
            set({ addons: [], loadedForBlenderVersionId: blenderVersionId, lastError: errorText(e) });
        }
    },

    async refreshAddons(blenderVersionId) {
        set({ isBusy: true, lastError: null });
        postStatus("Reading addons from Blender…", true);
        try {
            const addons = await addonService.refreshAddons(blenderVersionId);
            set({ addons, loadedForBlenderVersionId: blenderVersionId });
            postStatus(`${addons.length} addons read from Blender`);
        } catch (e) {
            console.error(e);
            set({ lastError: errorText(e) });
            postStatusError(`Reading addons failed: ${errorText(e)}`);
        } finally {
            set({ isBusy: false });
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

    clear: () => set({ addons: [], loadedForBlenderVersionId: null, lastError: null }),
}));

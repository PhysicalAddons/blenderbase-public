import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { ISetupSyncStatus } from "../models";
import { SetupService } from "../services/setupService";
import { postStatus, postStatusError } from "./statusStore";
import { useSetupRestoreStore } from "./setupRestoreStore";

/**
 * The sync folder: a folder, typically inside a cloud drive, that holds one setup file every
 * computer saves to and applies from. This store knows what the folder holds and whether it
 * is newer than what this computer last synced.
 */
interface ISetupSyncStore {
    status: ISetupSyncStatus | null,
    isBusy: boolean,
    /** When the "newer setup" hint was last posted, so refocusing does not repeat it. */
    hintedHash: string,
    load: () => Promise<void>,
    setFolder: (folderPath: string | null) => Promise<void>,
    /** Captures the setup and writes it to the folder's file. */
    save: (includeAddonFiles: boolean) => Promise<void>,
    /** Opens the folder's file in the restore view. */
    openForApply: () => Promise<void>,
    /** Records that this computer now matches the folder's file (after applying it). */
    markSynced: (contentHash: string) => Promise<void>,
    /** Re-reads the folder and posts a status hint when its file is newer than this computer. */
    checkForNews: () => Promise<void>,
}

const setupService = new SetupService();

const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e)).replace(/^cmd_\w+: /, "");

/** "Saved 21/09/2026 on Studio-PC", or what stands in for it. */
export const describeSyncFile = (status: ISetupSyncStatus): string => {
    const file = status.file;
    if (!file) {
        return status.file_error ? `The file in the folder cannot be read: ${status.file_error}` : "No setup saved in the folder yet";
    }
    const when = file.meta.created ? new Date(file.meta.created).toLocaleString() : "";
    const where = file.meta.device ? ` on ${file.meta.device}` : "";
    const what = `${file.blender_versions} ${file.blender_versions === 1 ? "version" : "versions"}, ${file.series} ${file.series === 1 ? "series" : "series"}`;
    return `Saved ${when}${where} · ${what}${status.is_newer ? " · newer than this computer" : " · this computer is up to date"}`;
};

export const useSetupSyncStore = create<ISetupSyncStore>((set, get) => ({
    status: null,
    isBusy: false,
    hintedHash: "",
    async load() {
        try {
            set({ status: await setupService.getSetupSync() });
        } catch (e) {
            console.error(e);
        }
    },
    async setFolder(folderPath) {
        try {
            set({ status: await setupService.setSetupSyncFolder(folderPath), hintedHash: "" });
            postStatus(folderPath ? `Sync folder set to ${folderPath}` : "Sync folder cleared");
        } catch (e) {
            console.error(e);
            postStatusError(`Changing the sync folder failed: ${errorText(e)}`);
        }
    },
    async save(includeAddonFiles) {
        set({ isBusy: true });
        postStatus("Reading the setup from every Blender series…", true);
        const stopListening = await listen<string>("setup-progress", (event) => postStatus(event.payload, true));
        try {
            const info = await setupService.saveSetupToSyncFolder(includeAddonFiles);
            info.warnings.forEach((w) => console.warn(w));
            set({ hintedHash: info.content_hash });
            await get().load();
            postStatus(`Setup saved to the sync folder · ${info.manifest.blender.length} Blender versions, ${Object.keys(info.manifest.series).length} series`);
        } catch (e) {
            console.error(e);
            postStatusError(`Saving to the sync folder failed: ${errorText(e)}`);
        } finally {
            stopListening();
            set({ isBusy: false });
        }
    },
    async openForApply() {
        const file = get().status?.file;
        if (!file) {
            return;
        }
        try {
            await useSetupRestoreStore.getState().open(file.file_path);
        } catch (e) {
            console.error(e);
            postStatusError(`Opening the setup from the sync folder failed: ${errorText(e)}`);
        }
    },
    async markSynced(contentHash) {
        try {
            set({ status: await setupService.markSetupSynced(contentHash), hintedHash: contentHash });
        } catch (e) {
            console.error(e);
        }
    },
    async checkForNews() {
        await get().load();
        const { status, hintedHash } = get();
        const file = status?.file;
        if (!status || !file || !status.is_newer || file.content_hash === hintedHash) {
            return;
        }
        set({ hintedHash: file.content_hash });
        const where = file.meta.device ? ` on ${file.meta.device}` : "";
        postStatus(`Newer setup in your sync folder, saved${where} · Settings › Setup to apply it`);
    },
}));

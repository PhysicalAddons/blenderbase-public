import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import { ISetupBundleInfo, ISetupExportOptions, ISetupSyncStatus, ITransferSent } from "../models";
import { SETUP_FILE_FILTER } from "../constants";
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
    /** Captures the chosen parts of the setup and writes them to the folder's file. */
    save: (options: ISetupExportOptions) => Promise<void>,
    /** Opens the folder's file in the restore view. */
    openForApply: () => Promise<void>,
    /** Records that this computer now matches the folder's file (after applying it). */
    markSynced: (contentHash: string) => Promise<void>,
    /** Re-reads the folder and posts a status hint when its file is newer than this computer. */
    checkForNews: () => Promise<void>,
    /** The last transfer this computer sent, for the code to show. */
    sent: ITransferSent | null,
    isSending: boolean,
    isReceiving: boolean,
    /** Saves the chosen parts of the setup and hands them to the relay; the code lands in `sent`. */
    send: (options: ISetupExportOptions) => Promise<void>,
    /** Fetches the transfer for a code and opens it in the restore view. */
    receive: (code: string) => Promise<void>,
    /** Asks where, then writes the chosen parts of the setup to a .bbsetup file. */
    saveToFile: (options: ISetupExportOptions) => Promise<void>,
    isSavingFile: boolean,
}

const setupService = new SetupService();

const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e)).replace(/^cmd_\w+: /, "");

const formatSetupSize = (bytes: number): string => {
    const kb = bytes / 1024;
    return kb >= 1024 ? `${(kb / 1024).toFixed(1)} MB` : `${Math.max(1, Math.round(kb))} KB`;
};

const plural = (count: number, one: string, many: string): string => `${count} ${count === 1 ? one : many}`;

/** One status line for a setup file: what it holds and how much of it restores on its own. */
export const describeSetup = (info: ISetupBundleInfo): string => {
    const sections = Object.values(info.manifest.series);
    const addons = sections.flatMap((s) => s.addons).filter((a) => a.source !== "core");
    const manual = addons.filter((a) => a.source === "manual" || (a.source === "file" && !a.file)).length;
    const parts = [
        plural(info.manifest.blender.length, "Blender version", "Blender versions"),
        plural(sections.length, "configuration", "configurations"),
        plural(addons.length, "addon", "addons") + (manual > 0 ? ` (${manual} to install by hand)` : ""),
        formatSetupSize(info.file_size),
    ];
    return parts.join(" · ");
};

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
    sent: null,
    isSending: false,
    isReceiving: false,
    isSavingFile: false,
    async saveToFile(options) {
        let filePath: string | null = null;
        try {
            filePath = await save({ title: "Save setup", defaultPath: "My Blender setup.bbsetup", filters: SETUP_FILE_FILTER });
        } catch (e) {
            console.error(e);
            postStatusError(`Choosing where to save the setup failed: ${errorText(e)}`);
        }
        if (!filePath) {
            return;
        }
        set({ isSavingFile: true });
        postStatus("Reading the setup from every Blender series…", true);
        // The backend names each step (series being read, addon being packed) as it goes.
        const stopListening = await listen<string>("setup-progress", (event) => postStatus(event.payload, true));
        try {
            const info = await setupService.exportSetupBundle(filePath, options);
            info.warnings.forEach((w) => console.warn(w));
            postStatus(`Setup saved: ${describeSetup(info)}`);
        } catch (e) {
            console.error(e);
            postStatusError(`Saving the setup failed: ${errorText(e)}`);
        } finally {
            stopListening();
            set({ isSavingFile: false });
        }
    },
    async send(options) {
        set({ isSending: true, sent: null });
        postStatus("Reading the setup from every Blender series…", true);
        const stopListening = await listen<string>("setup-progress", (event) => postStatus(event.payload, true));
        try {
            const sent = await setupService.sendSetupTransfer(options);
            set({ sent });
            postStatus(`Transfer ready · type ${sent.code} on the other computer within 7 days`);
        } catch (e) {
            console.error(e);
            postStatusError(`Sending the setup failed: ${errorText(e)}`);
        } finally {
            stopListening();
            set({ isSending: false });
        }
    },
    async receive(code) {
        set({ isReceiving: true });
        postStatus("Fetching the transfer…", true);
        const stopListening = await listen<string>("setup-progress", (event) => postStatus(event.payload, true));
        try {
            const info = await setupService.receiveSetupTransfer(code);
            postStatus(`Transfer received · ${info.manifest.blender.length} Blender versions, ${Object.keys(info.manifest.series).length} series`);
            await useSetupRestoreStore.getState().open(info.file_path);
        } catch (e) {
            console.error(e);
            postStatusError(`Receiving the transfer failed: ${errorText(e)}`);
        } finally {
            stopListening();
            set({ isReceiving: false });
        }
    },
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
    async save(options) {
        set({ isBusy: true });
        postStatus("Reading the setup from every Blender series…", true);
        const stopListening = await listen<string>("setup-progress", (event) => postStatus(event.payload, true));
        try {
            const info = await setupService.saveSetupToSyncFolder(options);
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
        postStatus(`Newer setup in your sync folder, saved${where} · open Sync to apply it`);
    },
}));

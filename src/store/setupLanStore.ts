import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { ILanStatus, ISetupExportOptions } from "../models";
import { SetupService } from "../services/setupService";
import { postStatus, postStatusError } from "./statusStore";
import { useSetupRestoreStore } from "./setupRestoreStore";

/**
 * This computer on the local network: the setup it shares and the other computers running
 * Blenderbase that it sees. The backend is silent on the network until the Local network tab
 * is open or a share is on; this store asks it for news while the tab shows.
 */
interface ISetupLanStore {
    status: ILanStatus | null,
    /** Reading the setup out and starting the share. */
    isStartingShare: boolean,
    isReceiving: boolean,
    /** Peers whose share was already pointed out in the status line. */
    hintedPeers: string[],
    refresh: () => Promise<void>,
    /** On while the Local network tab is open; off closes the sockets unless a share is on. */
    browse: (active: boolean) => Promise<void>,
    share: (options: ISetupExportOptions) => Promise<void>,
    stopShare: () => Promise<void>,
    /** Fetches a peer's setup with its PIN and opens it in the restore view. */
    receive: (peerId: string, pin: string) => Promise<void>,
}

const setupService = new SetupService();

const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e)).replace(/^cmd_\w+: /, "");

/** "483 921": the PIN as it is read out. */
export const formatPin = (pin: string): string => pin.replace(/^(\d{3})(\d{3})$/, "$1 $2");

export const formatLanSize = (bytes: number): string => {
    const kb = bytes / 1024;
    return kb >= 1024 ? `${(kb / 1024).toFixed(1)} MB` : `${Math.max(1, Math.round(kb))} KB`;
};

/** "Windows", "macOS", "Linux" from the platform string a peer announces. */
export const platformLabel = (platform: string): string => {
    if (platform.startsWith("windows")) {
        return "Windows";
    }
    if (platform.startsWith("macos")) {
        return "macOS";
    }
    if (platform.startsWith("linux")) {
        return "Linux";
    }
    return platform || "Unknown system";
};

export const useSetupLanStore = create<ISetupLanStore>((set, get) => ({
    status: null,
    isStartingShare: false,
    isReceiving: false,
    hintedPeers: [],
    async refresh() {
        try {
            const status = await setupService.lanStatus();
            const { hintedPeers } = get();
            const fresh = status.peers.filter((p) => p.share && !hintedPeers.includes(p.id));
            set({ status, hintedPeers: [...hintedPeers, ...fresh.map((p) => p.id)] });
            if (fresh.length === 1) {
                postStatus(`${fresh[0].device} is sharing a Blender setup on this network`);
            } else if (fresh.length > 1) {
                postStatus(`${fresh.length} computers on this network are sharing a Blender setup`);
            }
        } catch (e) {
            console.error(e);
        }
    },
    async browse(active) {
        try {
            set({ status: await setupService.lanBrowse(active) });
        } catch (e) {
            console.error(e);
            postStatusError(`Looking for computers on the network failed: ${errorText(e)}`);
        }
    },
    async share(options) {
        set({ isStartingShare: true });
        postStatus("Reading the setup from every Blender series…", true);
        const stopListening = await listen<string>("setup-progress", (event) => postStatus(event.payload, true));
        try {
            const status = await setupService.lanShareStart(options);
            set({ status });
            const share = status.share;
            postStatus(share ? `Sharing on the local network · PIN ${formatPin(share.pin)} · ${share.versions} Blender versions, ${share.series} series` : "Sharing on the local network");
        } catch (e) {
            console.error(e);
            postStatusError(`Sharing the setup failed: ${errorText(e)}`);
        } finally {
            stopListening();
            set({ isStartingShare: false });
        }
    },
    async stopShare() {
        try {
            set({ status: await setupService.lanShareStop() });
            postStatus("No longer sharing on the local network");
        } catch (e) {
            console.error(e);
            postStatusError(`Stopping the share failed: ${errorText(e)}`);
        }
    },
    async receive(peerId, pin) {
        set({ isReceiving: true });
        postStatus("Connecting…", true);
        const stopListening = await listen<string>("setup-progress", (event) => postStatus(event.payload, true));
        try {
            const info = await setupService.lanReceive(peerId, pin);
            const from = get().status?.peers.find((p) => p.id === peerId)?.device ?? "the other computer";
            postStatus(`Setup received from ${from} · ${info.manifest.blender.length} Blender versions, ${Object.keys(info.manifest.series).length} series`);
            await useSetupRestoreStore.getState().open(info.file_path);
        } catch (e) {
            console.error(e);
            postStatusError(`Receiving the setup failed: ${errorText(e)}`);
        } finally {
            stopListening();
            set({ isReceiving: false });
        }
    },
}));

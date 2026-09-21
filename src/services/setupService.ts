import { invoke } from "@tauri-apps/api/core";
import { ISeriesApplyChoice, ISeriesApplyReport, ISetupBundleInfo, ISetupSyncStatus, ITransferSent } from "../models";

export class SetupService {
    /** Reads the setup out of every installed Blender series (headless runs) and writes it to one file. */
    public async exportSetupBundle(filePath: string, includeAddonFiles: boolean): Promise<ISetupBundleInfo> {
        return await invoke("cmd_export_setup_bundle", { filePath, options: { include_addon_files: includeAddonFiles } });
    }

    /** Reads and verifies a setup file without changing anything. */
    public async inspectSetupBundle(filePath: string): Promise<ISetupBundleInfo> {
        return await invoke("cmd_inspect_setup_bundle", { filePath });
    }

    /** Applies the chosen series of a setup file; each series is backed up first. Refused while Blender runs. */
    public async applySetupBundle(filePath: string, choices: ISeriesApplyChoice[]): Promise<ISeriesApplyReport[]> {
        return await invoke("cmd_apply_setup_bundle", { filePath, options: { choices } });
    }

    /** The .bbsetup file the app was started with, if any (opened with the app). */
    public async startupSetupFile(): Promise<string | null> {
        return await invoke("cmd_startup_setup_file");
    }

    /** The sync folder and what it holds. */
    public async getSetupSync(): Promise<ISetupSyncStatus> {
        return await invoke("cmd_get_setup_sync");
    }

    /** Sets or clears (null) the sync folder. Changing it forgets what was synced. */
    public async setSetupSyncFolder(folderPath: string | null): Promise<ISetupSyncStatus> {
        return await invoke("cmd_set_setup_sync_folder", { folderPath });
    }

    /** Saves the setup to the sync folder's file and records it as synced. */
    public async saveSetupToSyncFolder(includeAddonFiles: boolean): Promise<ISetupBundleInfo> {
        return await invoke("cmd_save_setup_to_sync_folder", { options: { include_addon_files: includeAddonFiles } });
    }

    /** Records that this computer matches the given setup (after applying the folder's file). */
    public async markSetupSynced(contentHash: string): Promise<ISetupSyncStatus> {
        return await invoke("cmd_mark_setup_synced", { contentHash });
    }

    /** Saves the setup, encrypts it under a fresh code and hands it to the relay. */
    public async sendSetupTransfer(includeAddonFiles: boolean): Promise<ITransferSent> {
        return await invoke("cmd_send_setup_transfer", { options: { include_addon_files: includeAddonFiles } });
    }

    /** Fetches and decrypts the transfer for a code into the app's transfers folder. */
    public async receiveSetupTransfer(code: string): Promise<ISetupBundleInfo> {
        return await invoke("cmd_receive_setup_transfer", { code });
    }

    /** Puts the newest backup of a series back; resolves with the number of restored files. */
    public async undoSetupApply(series: string): Promise<number> {
        return await invoke("cmd_undo_setup_apply", { series });
    }
}

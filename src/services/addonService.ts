import { invoke } from "@tauri-apps/api/core";
import { IAddon } from "../models";

export class AddonService {
    /** Cached addons for a Blender version (fast, no Blender run). */
    public async fetchAddons(blenderVersionId: string): Promise<IAddon[]> {
        return await invoke("cmd_fetch_addons", { blenderVersionId });
    }

    /** Re-reads the addon list from Blender itself (runs the build headlessly). */
    public async refreshAddons(blenderVersionId: string): Promise<IAddon[]> {
        return await invoke("cmd_refresh_addons", { blenderVersionId });
    }

    public async toggleAddon(id: string, isEnabled: boolean): Promise<IAddon> {
        return await invoke("cmd_toggle_addon", { id, isEnabled });
    }

    public async installAddon(blenderVersionId: string, filePath: string): Promise<IAddon[]> {
        return await invoke("cmd_install_addon", { blenderVersionId, filePath });
    }

    public async symlinkAddon(blenderVersionId: string, directoryPath: string): Promise<IAddon[]> {
        return await invoke("cmd_symlink_addon", { blenderVersionId, directoryPath });
    }

    public async deleteAddon(id: string): Promise<IAddon[]> {
        return await invoke("cmd_delete_addon", { id });
    }

    public async revealAddon(id: string): Promise<void> {
        await invoke("cmd_reveal_addon_in_file_explorer", { id });
    }
}

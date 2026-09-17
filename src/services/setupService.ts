import { invoke } from "@tauri-apps/api/core";
import { ISetupBundleInfo } from "../models";

export class SetupService {
    /** Reads the setup out of every installed Blender series (headless runs) and writes it to one file. */
    public async exportSetupBundle(filePath: string, includeAddonFiles: boolean): Promise<ISetupBundleInfo> {
        return await invoke("cmd_export_setup_bundle", { filePath, options: { include_addon_files: includeAddonFiles } });
    }

    /** Reads and verifies a setup file without changing anything. */
    public async inspectSetupBundle(filePath: string): Promise<ISetupBundleInfo> {
        return await invoke("cmd_inspect_setup_bundle", { filePath });
    }
}

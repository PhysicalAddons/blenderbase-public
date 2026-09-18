import { invoke } from "@tauri-apps/api/core";
import { ISeriesApplyChoice, ISeriesApplyReport, ISetupBundleInfo } from "../models";

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

    /** Puts the newest backup of a series back; resolves with the number of restored files. */
    public async undoSetupApply(series: string): Promise<number> {
        return await invoke("cmd_undo_setup_apply", { series });
    }
}

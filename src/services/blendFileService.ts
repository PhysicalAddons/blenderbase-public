import { invoke } from "@tauri-apps/api/core";
import { IBlenderSeries, IBlendFile } from "../models";

/**
 * Every method rejects when the backend command fails; callers decide how to report it.
 */
export class BlendFileService {
    /** Imports the recent-files lists of every Blender series found on disk. */
    public async refreshRecentFiles(): Promise<void> {
        await invoke<void>("cmd_refresh_blend_files");
    }

    public async fetchBlenderSeries(id: string | null, limit: number | null, blenderConfigDirectory: string | null, isMapped: boolean, order: string): Promise<IBlenderSeries[]> {
        return await invoke<IBlenderSeries[]>("cmd_fetch_blender_series", {
            id,
            limit,
            blenderConfigDirectory,
            isMapped,
            order,
        });
    }

    /** Read-only query of the blend files already imported. */
    public async fetchBlendFiles(id: string | null, limit: number | null, filePath: string | null, blenderSeriesId: string | null, order: string | null): Promise<IBlendFile[]> {
        return await invoke<IBlendFile[]>("cmd_fetch_blend_files", {
            id,
            limit,
            filePath,
            blenderSeriesId,
            order,
        });
    }

    public async updateBlenderSeries(blenderSeries: IBlenderSeries): Promise<void> {
        await invoke<void>("cmd_update_blender_series", { blenderSeries });
    }

    /** Shows the .blend file in Explorer or Finder with the file selected. */
    public async revealBlendFile(blendFileId: string): Promise<void> {
        await invoke<void>("cmd_reveal_in_file_explorer", { blendFileId });
    }

    public async openBlendFile(blendFileId: string, blenderVersionId: string): Promise<void> {
        await invoke<void>("cmd_open_blend_file", { blendFileId, blenderVersionId });
    }
}

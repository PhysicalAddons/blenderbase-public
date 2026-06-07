import { invoke } from "@tauri-apps/api/core";
import { IBlenderSeries, IBlenderVersion, IBlendFile } from "../models";

export class BlendFileService {
    public async refreshRecentFiles(): Promise<void> {
        try {
            await invoke("cmd_refresh_recent_files")
        } catch (e) {
            console.error(e);
        }
    }
    public async fetchBlenderSeries(id: string | null, limit: number | null, blenderConfigDirectory: string | null, isMapped: boolean, order: string): Promise<IBlenderSeries[]> {
        try {
            return await invoke("cmd_fetch_blender_series", {id: id, limit: limit, blenderConfigDirectory: blenderConfigDirectory, isMapped: isMapped, order: order
            });
        } catch (e) {
            console.error(e);
            return [];
        }
    }
    public async fetchBlendFiles(id: string | null, limit: number | null, filePath: string | null, blenderSeriesId: string | null, order: string | null): Promise<IBlendFile[]> {
        try {
            return await invoke("cmd_fetch_blend_files", {
                id: id,
                limit: limit,
                filePath: filePath,
                blenderSeriesId: blenderSeriesId,
                order: order
            });
        } catch (e) {
            console.error(e);
            return [];
        }
    }
    public async updateBlenderSeries(blenderSeries: IBlenderSeries): Promise<void> {
        try {
            await invoke('cmd_update_blender_series', {
                blenderSeries: blenderSeries
            });    
        } catch (e) {
            console.error(e);
        }
    }
    public async openBlendFile(blendFile: IBlendFile, blenderVersion: IBlenderVersion): Promise<void> {
        try {
            await invoke('cmd_open_blend_file', {
                blendFile: blendFile,
                blenderVersion: blenderVersion
            });
        } catch (e) {
            console.error(e);
            
        }
    }
}
import { invoke } from "@tauri-apps/api/core";
import { IBlenderVersion, IBlenderVersionBuildType, IBlenderVersionInstallBuildTypeFilter, IDownloadableBlenderVersion } from "../models";

export class BlenderService {
    public async fetchBlenderVersions(
        id: string | null, 
        limit: number | null, 
        isDefault: boolean | null, 
        executableFilePath: string | null, 
        series: string | null, 
        order: string | null,
        downloadStatusTypes: string[],
    ): Promise<IBlenderVersion[]> {
        try {
            let a = await invoke("cmd_fetch_blender_versions", {
                id: id,
                limit: limit,
                isDefault: isDefault,
                executableFilePath: executableFilePath,
                series: series,
                order: order,
                downloadStatusTypes: downloadStatusTypes
            });
            return a as IBlenderVersion[];
        } catch (e) {
            console.error(e);
            return [];
        }
    };
    public async launchInstalledBlender(id: string): Promise<void> {
        try {
            await invoke("cmd_launch_blender_version", {
                id: id,
            });
        } catch (e) {
            console.error(e);
        }
    };
    public async deleteInstalledBlender(id: string): Promise<void> {
        try {
            await invoke("cmd_delete_blender_version", {
                id: id
            });
        } catch (e) {
            console.error(e);
        }
    };
    public async updateBlenderVersionDownloadStatusType(blenderVersion: IBlenderVersion, downloadStatusType: string) {
        try {
            await invoke("cmd_update_blender_version_download_status_type", {
                blenderVersion: blenderVersion,
                downloadStatusType: downloadStatusType
            });
        } catch (e) {
            console.error(e);
        }
    };
    public async updateInstallBlenderBuildType(selectedItem: IBlenderVersionInstallBuildTypeFilter): Promise<void> {
        try {
        	await invoke("update_install_blender_build_type", {
        		code: selectedItem.text
        	});
        } catch (e) {
        	console.error(e);
        }
    };
    public async fetchBlenderVersionBuildTypes(id: number | null, limit: number | null, code: string | null): Promise<IBlenderVersionBuildType[]> {
        try {
            return await invoke("cmd_fetch_blender_version_build_types", {
                id: id,
                limit: limit,
                code: code
            });
        } catch (e) {
            console.error(e);
            return [];
        }
    };
    public async updateDownloadBlenderBuildType(code: string | null): Promise<void> {
        try {
            await invoke("cmd_update_download_blender_build_type", {
				code: code
			});
        } catch (e) {
            console.error(e);
        }
    };
    public async getDownloadableBlenderVersionData(build: string, order: string): Promise<IDownloadableBlenderVersion[]> {
        try {
            return await invoke("cmd_get_downloadable_blender_version_data", {
                build: build,
                order: order
            });
        } catch (e) {
            console.error(e);
            return [];
        }
    };
    public async installBlenderVersion(id: string, archiveFilePath: string): Promise<void> {
        try {
            await invoke("cmd_install_blender_version", {
                id: id,
                archiveFilePath: archiveFilePath,
            });
        } catch (e) {
            console.error(e);
        }
    };  
    public async writeBlenderVersionDownloadData(downloadableBlenderVersion: IDownloadableBlenderVersion, directoryPath: string): Promise<void> {
        try {
            await invoke("cmd_write_blender_version_download_data", {
                downloadableBlenderVersion: downloadableBlenderVersion,
                directoryPath: directoryPath,
            });
        } catch (e) {
            console.error(e);
        }
    };
    public async updateBlenderVersion(blenderVersion: IBlenderVersion): Promise<void> {
        try {
            await invoke("cmd_update_blender_version", {
                blenderVersion: blenderVersion,
            });
        } catch (e) {
            console.error(e);
        }
    };
    public async setBlenderVersionAsDefault(id: String): Promise<void> {
        try {
            await invoke("cmd_set_blender_version_as_default", {
                id: id,
            });
        } catch (e) {
            console.error(e);
        }
    };
}

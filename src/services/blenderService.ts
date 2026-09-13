import { invoke } from "@tauri-apps/api/core";
import { IBlenderVersion, IBlenderVersionBuildType, IBlenderVersionInstallBuildTypeFilter, IDownloadableBlenderVersion } from "../models";

/**
 * Every method rejects when the backend command fails; callers decide how to report it.
 */
export class BlenderService {
    /** Rescans the installation locations on disk and picks a default version. */
    public async refreshBlenderVersions(): Promise<void> {
        await invoke<void>("cmd_refresh_blender_versions");
    }

    /** Read-only query of the versions already registered in the database. */
    public async fetchBlenderVersions(
        id: string | null,
        limit: number | null,
        isDefault: boolean | null,
        executableFilePath: string | null,
        series: string | null,
        order: string | null,
        downloadStatusTypes: string[],
    ): Promise<IBlenderVersion[]> {
        return await invoke<IBlenderVersion[]>("cmd_fetch_blender_versions", {
            id,
            limit,
            isDefault,
            executableFilePath,
            series,
            order,
            downloadStatusTypes,
        });
    }

    public async launchInstalledBlender(id: string): Promise<void> {
        await invoke<void>("cmd_launch_blender_version", { id });
    }

    /** Asks each build for its date, commit hash, branch and cycle; returns the updated rows. */
    public async refreshBlenderVersionDetails(ids: string[]): Promise<IBlenderVersion[]> {
        return await invoke<IBlenderVersion[]>("cmd_refresh_blender_version_details", { ids });
    }

    public async deleteInstalledBlender(id: string): Promise<void> {
        await invoke<void>("cmd_delete_blender_version", { id });
    }

    public async updateBlenderVersionDownloadStatusType(blenderVersion: IBlenderVersion, downloadStatusType: string): Promise<void> {
        await invoke<void>("cmd_update_blender_version_download_status_type", {
            blenderVersion,
            downloadStatusType,
        });
    }

    public async updateInstallBlenderBuildType(selectedItem: IBlenderVersionInstallBuildTypeFilter): Promise<void> {
        await invoke<void>("update_install_blender_build_type", { code: selectedItem.text });
    }

    public async fetchBlenderVersionBuildTypes(id: number | null, limit: number | null, code: string | null): Promise<IBlenderVersionBuildType[]> {
        return await invoke<IBlenderVersionBuildType[]>("cmd_fetch_blender_version_build_types", { id, limit, code });
    }

    public async updateDownloadBlenderBuildType(code: string | null): Promise<void> {
        await invoke<void>("cmd_update_download_blender_build_type", { code });
    }

    public async getDownloadableBlenderVersionData(build: string, order: string): Promise<IDownloadableBlenderVersion[]> {
        return await invoke<IDownloadableBlenderVersion[]>("cmd_get_downloadable_blender_version_data", { build, order });
    }

    public async installBlenderVersion(id: string, archiveFilePath: string): Promise<void> {
        await invoke<void>("cmd_install_blender_version", { id, archiveFilePath });
    }

    public async writeBlenderVersionDownloadData(downloadableBlenderVersion: IDownloadableBlenderVersion, directoryPath: string): Promise<void> {
        await invoke<void>("cmd_write_blender_version_download_data", {
            downloadableBlenderVersion,
            directoryPath,
        });
    }

    public async updateBlenderVersion(blenderVersion: IBlenderVersion): Promise<void> {
        await invoke<void>("cmd_update_blender_version", { blenderVersion });
    }

    public async setBlenderVersionAsDefault(id: string): Promise<void> {
        await invoke<void>("cmd_set_blender_version_as_default", { id });
    }
}

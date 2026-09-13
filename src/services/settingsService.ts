import { invoke } from "@tauri-apps/api/core";
import { IAppSetting, IAppSettingType, IBlenderInstallationLocation, IInputValueType } from "../models";

/**
 * Every method rejects when the backend command fails; callers decide how to report it.
 */
export class SettingsService {
    public async appSettingsInit(): Promise<void> {
        await invoke<void>("cmd_app_settings_init");
    }

    public async fetchBlenderInstallationPaths(
        id: string | null,
        limit: number | null,
        directoryPath: string | null,
        isDefault: boolean | null
    ): Promise<IBlenderInstallationLocation[]> {
        return await invoke<IBlenderInstallationLocation[]>("cmd_fetch_blender_installation_locations", {
            id,
            limit,
            directoryPath,
            isDefault,
        });
    }

    /** Confirms the install location (creating it when needed) and makes it the default. */
    public async confirmBlenderInstallationLocation(id: string, directoryPath: string): Promise<IBlenderInstallationLocation> {
        return await invoke<IBlenderInstallationLocation>("cmd_confirm_blender_installation_location", { id, directoryPath });
    }

    public async insertBlenderInstallationLocation(): Promise<void> {
        await invoke<void>("cmd_insert_blender_installation_location");
    }

    public async setBlenderInstallationLocationAsDefault(id: string, isDefault: boolean): Promise<void> {
        await invoke<void>("cmd_set_blender_installation_location_as_default", { id, isDefault });
    }

    public async deleteBlenderInstallationLocation(id: string): Promise<void> {
        await invoke<void>("cmd_delete_blender_installation_location", { id });
    }

    public async fetchAppSetting(id: number | null, limit: number | null, code: string | null, isReadOnAppLaunch: boolean | null, appSettingTypeId: number | null): Promise<IAppSetting[]> {
        return await invoke<IAppSetting[]>("cmd_fetch_app_setting", {
            id,
            limit,
            code,
            isReadOnAppLaunch,
            appSettingTypeId,
        });
    }

    public async fetchAppSettingType(id: number | null, limit: number | null, code: string | null): Promise<IAppSettingType[]> {
        return await invoke<IAppSettingType[]>("cmd_fetch_app_setting_type", { id, limit, code });
    }

    public async fetchInputValueType(id: number | null, limit: number | null, code: string | null): Promise<IInputValueType[]> {
        return await invoke<IInputValueType[]>("cmd_fetch_input_value_type", { id, limit, code });
    }

    /** Applies a changed setting. Rejects with the backend's message when it is refused. */
    public async handleSetting(appSetting: IAppSetting): Promise<void> {
        await invoke<void>("cmd_handle_setting", { appSetting });
    }
}

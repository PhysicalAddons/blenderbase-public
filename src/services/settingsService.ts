import { invoke } from "@tauri-apps/api/core";
import { IAppSetting, IAppSettingType, IBlenderInstallationLocation, IInputValueType } from "../models";

export class SettingsService {
    public async appSettingsInit(): Promise<void> {
        try {
            await invoke("cmd_app_settings_init");
        } catch (e) {
            console.error(e);
        }
    }
    public async fetchBlenderInstallationPaths(
        id: string | null,
        limit: number | null,
        directoryPath: string | null,
        isDefault: boolean | null
    ): Promise<IBlenderInstallationLocation[]> {
        try {
            return await invoke("cmd_fetch_blender_installation_locations", {
                id: id,
                limit: limit,
                directoryPath: directoryPath,
                isDefault: isDefault
            });
        } catch (e) {
            console.error(e);
            return [];
        }
    };
    public async insertBlenderInstallationLocation(): Promise<void> {
        try {
            await invoke("cmd_insert_blender_installation_location");
        } catch (e) {
            console.error(e);
        }
    };
    public async setBlenderInstallationLocationAsDefault(id: string, is_default: boolean): Promise<void> {
        try {
            await invoke("cmd_set_blender_installation_location_as_default", {
                id: id,
                isDefault: is_default
            });
        } catch (e) {
            console.error(e);
        }
    };
    public async deleteBlenderInstallationLocation(id: string): Promise<void> {
        try {
            await invoke("cmd_delete_blender_installation_location", { id });
        } catch (e) {
            console.error(e);
        }
    };
    public async fetchAppSetting(id: number | null, limit: number | null, code: string | null, isReadOnAppLaunch: boolean | null, appSettingTypeId: number | null): Promise<IAppSetting[]> {
        try {
            return await invoke("cmd_fetch_app_setting", {
                id: id,
                limit: limit,
                code: code,
                is_read_on_app_launch: isReadOnAppLaunch,
                appSettingTypeId: appSettingTypeId
            });
        } catch (e) {
            console.error(e);
            return [];
        }
    };
    public async fetchAppSettingType(id: number | null, limit: number | null, code: string | null): Promise<IAppSettingType[]> {
        try {
            return await invoke("cmd_fetch_app_setting_type", {
                id: id,
                limit: limit,
                code: code
            })
        } catch (e) {
            console.error(e);
            return [];
        }
    };
    public async fetchInputValueType(id: number | null, limit: number | null, code: string | null): Promise<IInputValueType[]> {
        try {
            return await invoke("cmd_fetch_input_value_type", {
                id: id,
                limit: limit,
                code: code
            })
        } catch (e) {
            console.error(e);
            return [];
        }
    };
    public async handleSetting(appSetting: IAppSetting): Promise<string | undefined> {
        try {
            await invoke("cmd_handle_setting", {
                appSetting: appSetting
            });
        } catch (e: any) {
            console.error(e);
            return e?.message || String(e);
        }
    };
}
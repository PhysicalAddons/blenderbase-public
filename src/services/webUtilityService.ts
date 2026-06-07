import { invoke } from "@tauri-apps/api/core";

export class WebUtilityService {
    public async checkInternetConnection(isOverride: boolean | null): Promise<boolean | undefined | null> {
        try {
            return await invoke("cmd_check_internet_connection", {
                isOverride: isOverride,
            });
        } catch (e) {
            console.error(e);
            return null;
        }
    }
}


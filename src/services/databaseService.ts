import { invoke } from "@tauri-apps/api/core";

export class DatabaseService {
    public async appDbInit(): Promise<void> {
        try {
            await invoke("cmd_app_db_init");
        } catch (e) {
            console.error(e);
        }
    }
}

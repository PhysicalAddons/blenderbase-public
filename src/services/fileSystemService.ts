import { invoke } from "@tauri-apps/api/core";

export class FileSystemService {
    public async instancePopupWindow(
        label: string,
        title: string,
        urlPath: string,
        width: number,
        height: number,
        resizeable: boolean,
        alwaysOnTop: boolean,
        focused: boolean,
        skipTaskbar: boolean
    ): Promise<void> {
        try {
            await invoke("cmd_instance_popup_window", {
                label: label,
                title: title,
                urlPath: urlPath,
				width: width,
				height: height,
				resizeable: resizeable,
				alwaysOnTop: alwaysOnTop,
				focused: focused,
				skipTaskbar: skipTaskbar
            });
        } catch (e) {
            console.error(e);
        }
    }
}

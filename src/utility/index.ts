import { download } from "@tauri-apps/plugin-upload";

export const generateTitle = (data: any) => {
    try {
        let title = "";
        Object.entries(data).forEach(([k, v]) => {
            if (v !== null && v !== "") {
                let formattedKey = k.replace(/_/g, ' ');
                let capitalizedKey = formattedKey.charAt(0).toUpperCase() + formattedKey.slice(1);
                title += `${capitalizedKey}: ${v} \n`;
            }
        })    
    } catch (e) {
        console.error(e);
        return "";
    }
}

export async function downloadFile(url: string, filePath: string, buttonId: string) : Promise<boolean> {
    let isSuccess = true;
    const button = document.getElementById(buttonId) as HTMLButtonElement;
    if (!button) {
        isSuccess = false;
        return isSuccess;
    }
    let accumulated = 0;
    const originalText = button.textContent;

    button.disabled = true;
    button.textContent = "Starting...";

    try {
        await download(url, filePath, ({ progress, total }) => {
            accumulated += progress;
            const percent = Math.floor((accumulated / total) * 100);
            const button = document.getElementById(buttonId) as HTMLButtonElement;
            if (button) {
                button.textContent = `${percent}%`;
                button.disabled = true;
            }
        });
        const finishedButton = document.getElementById(buttonId) as HTMLButtonElement;
        if (finishedButton) {
            finishedButton.textContent = originalText;
            finishedButton.disabled = false;
        }
    } catch (e) {
        isSuccess = false;
        if (button) {
            button.textContent = "Error";
        }
        console.error(e);
    } finally {
        if (button) {
            button.disabled = false;
        } 
        return isSuccess;
    }
}

export const parseVersion = (v: string) => v.split('.').map(p => p.padStart(10, '0')).join('.');
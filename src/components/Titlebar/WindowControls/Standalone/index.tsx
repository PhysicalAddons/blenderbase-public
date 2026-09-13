import { Close } from '@carbon/react/icons'
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

const StandaloneWindowControls = () => {
    return (
        <div className="windows_app_window_control">
            <button
                type="button"
                className="window_control_options close-win"
                aria-label="Close"
                title="Close"
                onClick={() => appWindow.close().catch((e) => console.error(e))}
            >
                <Close />
            </button>
        </div>
    )
}

export default StandaloneWindowControls

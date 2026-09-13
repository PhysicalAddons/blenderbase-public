import { Close, SubtractLarge, Scale } from '@carbon/react/icons';
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

const MainWindowControls = () => {
    return (
        <div className="windows_app_window_control">
            <button
                type="button"
                className="window_control_options"
                aria-label="Minimize"
                title="Minimize"
                onClick={() => appWindow.minimize().catch((e) => console.error(e))}
            >
                <SubtractLarge />
            </button>
            <button
                type="button"
                className="window_control_options"
                aria-label="Maximize"
                title="Maximize"
                onClick={() => appWindow.toggleMaximize().catch((e) => console.error(e))}
            >
                <Scale />
            </button>
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

export default MainWindowControls

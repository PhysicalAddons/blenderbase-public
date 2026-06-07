import { Close, SubtractLarge, Scale } from '@carbon/react/icons';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEffect } from 'react';

const appWindow = getCurrentWindow();

const MainWindowControls = () => {
    useEffect(() => {
        document.getElementById('minimize-win')?.addEventListener('click', () => appWindow.minimize());
        document.getElementById('maximize-win')?.addEventListener('click', () => appWindow.toggleMaximize());
        document.getElementById('close-win')?.addEventListener('click', () => appWindow.close());
        return () => {
            document.getElementById('minimize-win')?.removeEventListener('click', () => appWindow.minimize());
            document.getElementById('maximize-win')?.removeEventListener('click', () => appWindow.toggleMaximize());
            document.getElementById('close-win')?.removeEventListener('click', () => appWindow.close());
        }
    }, []);
    return (
        <>
            <div
                className="windows_app_window_control"
            >
                <div
                    id="minimize-win"
                    className="window_control_options"
                >
                    <SubtractLarge />
                </div>
                <div
                    id="maximize-win"
                    className="window_control_options"
                >
                    <Scale />
                </div>
                <div
                    id="close-win"
                    className="window_control_options close-win"
                >
                    <Close />
                </div>
            </div>
        </>
    )
}

export default MainWindowControls
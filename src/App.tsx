import { HashRouter } from "react-router-dom";
import AppRouter from "./router";
import "./styles.scss";
import { useEffect } from "react";
import { useDisplayInformationStore } from "./store/displayInformationStore";
import { getVersion } from '@tauri-apps/api/app';
import { useNetworkInformationStore } from "./store/networkInformationStore";
import { WebUtilityService } from "./services/webUtilityService";
import { SettingsService } from "./services/settingsService";
import { DatabaseService } from "./services/databaseService";
import { useThemeStore } from "./store/themeStore";
import { useActivityStore } from "./store/activityStore";

const webUtilityService = new WebUtilityService();
const settingsService = new SettingsService();
const databaseService = new DatabaseService();

const AppContent = () => {
    const setAppVersion = useDisplayInformationStore((s) => s.setAppVersion)
    const setHasInternetConnection = useNetworkInformationStore((s) => s.setHasInternetConnection)
    const initTheme = useThemeStore((t) => t.init)
    // const location = useLocation();
    // Show title bar if we're not in a popup route.
    // const isStandalone = location.pathname.startsWith('/standalone');
    useEffect(() => {
        initTheme();
        async function init() {
            try {
                await fetchVersion();
                await checkInternetConnectionOverride();
                await databaseService.appDbInit();
                await settingsService.appSettingsInit();
            } catch (e) {
                console.error(e);
            }
            // Stats: sessions Blender logged while the app was closed are read now. Its own
            // try: a stats hiccup must not look like a startup failure.
            try {
                await useActivityStore.getState().importAndRefresh(true);
            } catch (e) {
                console.error(e);
            }
        }
        init();
    }, [])
    useEffect(() => {
        const checkInternetConnectionHandler = async () => {
            const a = await webUtilityService.checkInternetConnection(false);
            if (a !== null && a !== undefined) {
                setHasInternetConnection(a);
            }
        };
        // Coming back from Blender is when its log has grown: read it then.
        const importActivityHandler = () => {
            useActivityStore.getState().importAndRefresh().catch((e) => console.error(e));
        };
        window.addEventListener("focus", checkInternetConnectionHandler);
        window.addEventListener("focus", importActivityHandler);
        return () => {
            window.removeEventListener("focus", checkInternetConnectionHandler);
            window.removeEventListener("focus", importActivityHandler);
        };
    }, []);
    // The webview's own drag-and-drop handling is off (dragDropEnabled in tauri.conf.json):
    // dragging addons between the columns needs that on Windows. Without it a file dropped
    // on the window would be opened in place of the app, so file drops are swallowed here.
    useEffect(() => {
        const swallowFileDrop = (e: DragEvent) => {
            if (e.dataTransfer?.types.includes("Files")) {
                e.preventDefault();
            }
        };
        document.addEventListener("dragover", swallowFileDrop);
        document.addEventListener("drop", swallowFileDrop);
        return () => {
            document.removeEventListener("dragover", swallowFileDrop);
            document.removeEventListener("drop", swallowFileDrop);
        };
    }, []);
    const fetchVersion = async () => {
        try {
            setAppVersion(await getVersion());
        } catch (e) {
            console.error(e);
        }
    };
    const checkInternetConnectionOverride = async () => {
        try {
            setHasInternetConnection(await webUtilityService.checkInternetConnection(true) as boolean);
        } catch (e) {
            console.error(e);
        }
    }
    return (
        <>
            {/* {!isStandalone && <Titlebar />} */}
            {/* {isStandalone && <StandaloneTitlebar />} */}
            <AppRouter />
        </>
    );
};
const App = () => (
    <HashRouter>
        <AppContent />
    </HashRouter>
);

export default App;
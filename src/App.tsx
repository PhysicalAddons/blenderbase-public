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

const webUtilityService = new WebUtilityService();
const settingsService = new SettingsService();
const databaseService = new DatabaseService();

const AppContent = () => {
    const { setAppVersion } = useDisplayInformationStore()
    const { setHasInternetConnection } = useNetworkInformationStore()
    // const location = useLocation();
    // Show title bar if we're not in a popup route.
    // const isStandalone = location.pathname.startsWith('/standalone');
    useEffect(() => {
        async function init() {
            try {
                await fetchVersion();
                await checkInternetConnectionOverride();
                await databaseService.appDbInit();
                await settingsService.appSettingsInit();
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
        window.addEventListener("focus", checkInternetConnectionHandler);
        return () => {
            window.removeEventListener("focus", checkInternetConnectionHandler);
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
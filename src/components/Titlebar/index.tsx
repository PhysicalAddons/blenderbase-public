import { useEffect, useState } from "react";
import Links from "./Links";
import UtilityOptions from "./UtilityOptions";
import MainWindowControls from "./WindowControls/Main";
import { OS_OBJECT } from "../../constants";
import StatusInformation from "./InternetConnection";

const Titlebar = () => {
    const [OS, setOS] = useState<string>("");
    useEffect(() => {
        const userAgent = window.navigator.userAgent;
        if (userAgent.indexOf("Windows") !== -1) {
            setOS(OS_OBJECT.win);
        } else if (userAgent.indexOf("Mac") !== -1) {
            setOS(OS_OBJECT.mac);
        }
    }, []);

    return (
        <nav className={`navigation_bar ${OS}`}>
            <div
                className="navigation_bar_drag_region_title_bar"
                data-tauri-drag-region
            >
                {/* This is to be left empty. */}
            </div>
            <Links />
            <div className="navigation_bar_delimiter">
                {/* This is to be left empty. */}
            </div>
            <StatusInformation />
            <UtilityOptions />
            <MainWindowControls />
        </nav>
    );
};

export default Titlebar;

import { useEffect, useState } from "react";
import { OS_OBJECT } from "../../constants";
import StandaloneWindowControls from "../Titlebar/WindowControls/Standalone";

const StandaloneTitlebar = () => {
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
            <div className="navigation_bar_delimiter">
                {/* This is to be left empty. */}
            </div>
            <StandaloneWindowControls />
        </nav>
    );
};

export default StandaloneTitlebar;

import { useUiControlsStore } from "../../store/uiControlsStore";
import ButtonSideBarToggle from "./Actions/Button";
import Sections from "./Sections";
import { useBlendFileStore } from "../../store/blendFileStore";
import { useEffect } from "react";

const RecentFiles = () => {
    const { isSidebarExpanded, setIsSidebarExpanded } = useUiControlsStore()
    const { setBlenderSeries } = useBlendFileStore()

    useEffect(() => {
        async function init() {
            setBlenderSeries();
        }
        init();
    }, [])

    const handleRecentFilesSidebarToggle = async (v: boolean) => {
        try {
            setIsSidebarExpanded(v)
			await setBlenderSeries();
        } catch (e) {
            console.error(e);
        }
    }

    return (
        <div className={`recent_files_panel ${isSidebarExpanded ? '' : 'hidden'}`}>
            <div className="recent_files"> 
                {/* theme="g100"> */}
                <h4>Recent Files</h4>
                <ButtonSideBarToggle 
                    isSidebarExpanded={isSidebarExpanded} 
                    setIsSidebarExpanded={handleRecentFilesSidebarToggle} 
                />
                <Sections />
            </div>
        </div>
    )
}

export default RecentFiles
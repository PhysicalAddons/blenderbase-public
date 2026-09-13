import { useUiControlsStore } from "../../store/uiControlsStore";
import ButtonSideBarToggle from "./Actions/Button";
import Sections from "./Sections";
import { useBlendFileStore } from "../../store/blendFileStore";
import { useEffect, useRef } from "react";
import { usePagedScroll } from "../../utility/usePagedScroll";

const RecentFiles = () => {
    const { isSidebarExpanded, setIsSidebarExpanded } = useUiControlsStore()
    const { setBlenderSeries } = useBlendFileStore()
    const listRef = useRef<HTMLDivElement>(null)
    usePagedScroll(listRef)

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
                {/* Same top-right corner as the open button in the middle column. */}
                <ButtonSideBarToggle
                    isSidebarExpanded={isSidebarExpanded}
                    setIsSidebarExpanded={handleRecentFilesSidebarToggle}
                />
                <div className="column_header">
                    <div className="column_header__titles">
                        <span className="column_header__title">Recent Files</span>
                        {/* Empty on purpose: keeps the header the same height as the other columns. */}
                        <span className="column_header__subtitle">{" "}</span>
                    </div>
                </div>
                <div className="recent_files__sections" ref={listRef}>
                    <Sections />
                </div>
            </div>
        </div>
    )
}

export default RecentFiles

import { useUiControlsStore } from "../../store/uiControlsStore";
import ButtonSideBarToggle from "./Actions/Button";
import Sections from "./Sections";
import { useBlendFileStore } from "../../store/blendFileStore";
import { useEffect, useRef } from "react";
import { useShallow } from "zustand/react/shallow";
import { usePagedScroll } from "../../utility/usePagedScroll";

const RecentFiles = () => {
    const { isSidebarExpanded, setIsSidebarExpanded } = useUiControlsStore(
        useShallow((s) => ({ isSidebarExpanded: s.isSidebarExpanded, setIsSidebarExpanded: s.setIsSidebarExpanded }))
    )
    const setBlenderSeries = useBlendFileStore((s) => s.setBlenderSeries)
    const listRef = useRef<HTMLDivElement>(null)
    usePagedScroll(listRef)

    // First mount: import the recent files from disk, then load. Later mounts (and StrictMode's
    // second run) only load. The store owns the state, so nothing is set after unmount.
    useEffect(() => {
        const { hasRefreshedBlendFiles, refreshBlendFiles, setBlenderSeries } = useBlendFileStore.getState();
        const load = hasRefreshedBlendFiles ? setBlenderSeries : refreshBlendFiles;
        load().catch((e) => console.error(e));
    }, [])

    const handleRecentFilesSidebarToggle = async (v: boolean) => {
        setIsSidebarExpanded(v)
        await setBlenderSeries();
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
                        {/* A non-breaking space, not a plain one: a whitespace-only span collapses to
                            zero height and the whole column would sit 18px higher than its neighbours. */}
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

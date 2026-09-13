import { useUiControlsStore } from "../../store/uiControlsStore";
import RecentFilesToggle from "./Actions/Button";
import Sections from "./Sections";
import { useBlendFileStore } from "../../store/blendFileStore";
import { useEffect, useRef } from "react";
import { usePagedScroll } from "../../utility/usePagedScroll";

const RecentFiles = () => {
    const isSidebarExpanded = useUiControlsStore((s) => s.isSidebarExpanded)
    const listRef = useRef<HTMLDivElement>(null)
    // Series headers are one row, file rows half a row; both edges are snap points.
    usePagedScroll(listRef, { rowSelector: '.cds--contained-list__header, .cds--contained-list-item' })

    // First mount: import the recent files from disk, then load. Later mounts (and StrictMode's
    // second run) only load. The store owns the state, so nothing is set after unmount.
    useEffect(() => {
        const { hasRefreshedBlendFiles, refreshBlendFiles, setBlenderSeries } = useBlendFileStore.getState();
        const load = hasRefreshedBlendFiles ? setBlenderSeries : refreshBlendFiles;
        load().catch((e) => console.error(e));
    }, [])

    return (
        <div className={`recent_files_panel ${isSidebarExpanded ? '' : 'hidden'}`}>
            <div className="recent_files">
                <div className="column_header">
                    <div className="column_header__titles">
                        <span className="column_header__title">Recent Files</span>
                        {/* A non-breaking space, not a plain one: a whitespace-only span collapses to
                            zero height and the whole column would sit 18px higher than its neighbours. */}
                        <span className="column_header__subtitle">{" "}</span>
                    </div>
                </div>
                {/* Toolbar band on the same row as the other columns; the hide block sits at the right edge. */}
                <div className="column_actions recent_files__toolbar">
                    <RecentFilesToggle placement='recent' />
                </div>
                {/* Same list-header line as the other columns, so the series and file rows start on their grid. */}
                <div className="list_header recent_files__list_header">
                    <span>Files by Blender version</span>
                </div>
                <div className="recent_files__sections" ref={listRef}>
                    <Sections />
                </div>
            </div>
        </div>
    )
}

export default RecentFiles

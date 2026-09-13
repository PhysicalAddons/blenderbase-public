import { SidePanelClose, SidePanelOpen } from '@carbon/react/icons'
import { useShallow } from 'zustand/react/shallow'
import { useUiControlsStore } from '../../../../store/uiControlsStore'
import { useBlendFileStore } from '../../../../store/blendFileStore'

type Props = {
    /**
     * Where the block lives. In the middle column it only shows while Recent Files is hidden
     * (the open panel has its own block at the window's right edge); in the Recent Files band
     * it always shows, since the band disappears with the panel.
     */
    placement: 'middle' | 'recent',
}

/**
 * Shows or hides the Recent Files column. A 48px block at the right end of a
 * toolbar band: the middle column's band while the column is hidden, the
 * Recent Files band while it is shown, so it always sits at the window's right edge.
 */
const RecentFilesToggle = ({ placement }: Props) => {
    const { isSidebarExpanded, setIsSidebarExpanded } = useUiControlsStore(
        useShallow((s) => ({ isSidebarExpanded: s.isSidebarExpanded, setIsSidebarExpanded: s.setIsSidebarExpanded }))
    )
    const setBlenderSeries = useBlendFileStore((s) => s.setBlenderSeries)

    const toggle = async () => {
        setIsSidebarExpanded(!isSidebarExpanded)
        await setBlenderSeries()
    }

    if (placement === 'middle' && isSidebarExpanded) {
        return null;
    }
    const label = isSidebarExpanded ? "Hide recent files" : "Show recent files";
    const Icon = isSidebarExpanded ? SidePanelOpen : SidePanelClose;
    // In the middle column the block names what it opens, like the filter block next to it;
    // in the Recent Files band the column title already says it, so the icon is enough.
    const labelled = placement === 'middle';
    return (
        <button
            type="button"
            className={`recent_files_toggle ${labelled ? "recent_files_toggle--labelled" : ""}`}
            title={label}
            aria-label={label}
            aria-pressed={isSidebarExpanded}
            onClick={() => void toggle()}
        >
            {labelled && <span className='recent_files_toggle__text'>Recent Files</span>}
            <Icon size={16} aria-hidden="true" />
        </button>
    )
}

export default RecentFilesToggle

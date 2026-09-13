import { SidePanelClose } from '@carbon/icons-react';
import { Button } from '@carbon/react';
import RecentFiles from '../../components/RecentFiles';
import LauncherBar from '../../components/LauncherBar/index';
import BlenderColumn from '../../components/BlenderColumn';
import AddonPanel from '../../components/AddonPanel';
import InstallBlenderPanel from '../../components/InstallBlenderPanel';
import EmptyState from '../../components/EmptyState';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { useEffect } from 'react';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { useBlendFileStore } from '../../store/blendFileStore';

const Home = () => {
    const { isSidebarExpanded, isInstallBlenderOpen, setIsSidebarExpanded, setIsInstallBlenderOpen } = useUiControlsStore()
    const { setBlenderSeries } = useBlendFileStore()
    const { installedBuilds, hasLoadedInstalledBuilds } = useBlenderManagerStore()
    const isEmpty = hasLoadedInstalledBuilds && installedBuilds.length === 0;

    // Escape leaves the Install Blender view, like closing a dialog.
    useEffect(() => {
        if (!isInstallBlenderOpen) {
            return;
        }
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsInstallBlenderOpen(false);
            }
        };
        document.addEventListener('keydown', onKeyDown);
        return () => document.removeEventListener('keydown', onKeyDown);
    }, [isInstallBlenderOpen]);

    const openRecentFilesPanel = async () => {
        try {
            setIsSidebarExpanded(true)
            await setBlenderSeries();
        } catch (e) {
            console.error(e);
        }
    }

    return (
        <>
            <div className={`home ${isInstallBlenderOpen ? 'home--installing' : ''}`}>
                <BlenderColumn />
                <div className='home__main'>
                    {!isSidebarExpanded && (
                        <div className='sidebar_toggle__open'>
                            <Button
                                renderIcon={SidePanelClose}
                                kind="ghost"
                                iconDescription="Open recent files"
                                title="Open recent files"
                                hasIconOnly
                                onClick={openRecentFilesPanel}
                            />
                        </div>
                    )}
                    {isInstallBlenderOpen ? <InstallBlenderPanel /> : isEmpty ? <EmptyState /> : <AddonPanel />}
                </div>
                <RecentFiles />
            </div>
            <LauncherBar />
        </>
    )
}

export default Home

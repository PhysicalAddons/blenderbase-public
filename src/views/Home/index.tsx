import { SidePanelClose } from '@carbon/react/icons';
import { Button } from '@carbon/react';
import { useShallow } from 'zustand/react/shallow';
import RecentFiles from '../../components/RecentFiles';
import LauncherBar from '../../components/LauncherBar/index';
import BlenderColumn from '../../components/BlenderColumn';
import AddonPanel from '../../components/AddonPanel';
import InstallBlenderPanel from '../../components/InstallBlenderPanel';
import SettingsPanel from '../../components/SettingsPanel';
import EmptyState from '../../components/EmptyState';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { useEffect } from 'react';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { useBlendFileStore } from '../../store/blendFileStore';
import { postStatusError } from '../../store/statusStore';

const Home = () => {
    const { isSidebarExpanded, isInstallBlenderOpen, isSettingsOpen, setIsSidebarExpanded, setIsInstallBlenderOpen, setIsSettingsOpen } = useUiControlsStore(
        useShallow((s) => ({
            isSidebarExpanded: s.isSidebarExpanded,
            isInstallBlenderOpen: s.isInstallBlenderOpen,
            isSettingsOpen: s.isSettingsOpen,
            setIsSidebarExpanded: s.setIsSidebarExpanded,
            setIsInstallBlenderOpen: s.setIsInstallBlenderOpen,
            setIsSettingsOpen: s.setIsSettingsOpen,
        }))
    )
    const setBlenderSeries = useBlendFileStore((s) => s.setBlenderSeries)
    const { installedBuilds, hasLoadedInstalledBuilds } = useBlenderManagerStore(
        useShallow((s) => ({ installedBuilds: s.installedBuilds, hasLoadedInstalledBuilds: s.hasLoadedInstalledBuilds }))
    )
    const isEmpty = hasLoadedInstalledBuilds && installedBuilds.length === 0;

    // First visit: scan the installation locations on disk, then load the list. Later visits
    // (and StrictMode's second run) only reload; the refresh button rescans on demand.
    useEffect(() => {
        const { hasRefreshedInstalledBuilds, refreshInstalledBuilds, setInstalledBuilds } = useBlenderManagerStore.getState();
        const load = hasRefreshedInstalledBuilds ? setInstalledBuilds : refreshInstalledBuilds;
        load().catch((e) => {
            console.error(e);
            postStatusError(`Loading installed Blender versions failed: ${e}`);
        });
    }, []);

    // Escape leaves the Install Blender or Settings view, like closing a dialog.
    useEffect(() => {
        if (!isInstallBlenderOpen && !isSettingsOpen) {
            return;
        }
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsInstallBlenderOpen(false);
                setIsSettingsOpen(false);
            }
        };
        document.addEventListener('keydown', onKeyDown);
        return () => document.removeEventListener('keydown', onKeyDown);
    }, [isInstallBlenderOpen, isSettingsOpen]);

    const openRecentFilesPanel = async () => {
        setIsSidebarExpanded(true)
        await setBlenderSeries();
    }

    return (
        <>
            <div className={`home ${isInstallBlenderOpen ? 'home--installing' : ''} ${isSettingsOpen ? 'home--settings' : ''}`}>
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
                    {isSettingsOpen ? <SettingsPanel /> : isInstallBlenderOpen ? <InstallBlenderPanel /> : isEmpty ? <EmptyState /> : <AddonPanel />}
                </div>
                <RecentFiles />
            </div>
            <LauncherBar />
        </>
    )
}

export default Home

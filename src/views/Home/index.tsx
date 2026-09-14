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
import { postStatusError } from '../../store/statusStore';
import { useRescanOnFocus } from '../../utility/useRescanOnFocus';

const Home = () => {
    const { isInstallBlenderOpen, isSettingsOpen, setIsInstallBlenderOpen, setIsSettingsOpen } = useUiControlsStore(
        useShallow((s) => ({
            isInstallBlenderOpen: s.isInstallBlenderOpen,
            isSettingsOpen: s.isSettingsOpen,
            setIsInstallBlenderOpen: s.setIsInstallBlenderOpen,
            setIsSettingsOpen: s.setIsSettingsOpen,
        }))
    )
    const { installedBuilds, hasLoadedInstalledBuilds } = useBlenderManagerStore(
        useShallow((s) => ({ installedBuilds: s.installedBuilds, hasLoadedInstalledBuilds: s.hasLoadedInstalledBuilds }))
    )
    const isEmpty = hasLoadedInstalledBuilds && installedBuilds.length === 0;
    useRescanOnFocus();

    // First visit: scan the installation locations on disk, then load the list. Later visits
    // (and StrictMode's second run) only reload; refocusing the window rescans later on.
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

    return (
        <>
            <div className={`home ${isInstallBlenderOpen ? 'home--installing' : ''} ${isSettingsOpen ? 'home--settings' : ''}`}>
                <BlenderColumn />
                <div className='home__main'>
                    {isSettingsOpen ? <SettingsPanel /> : isInstallBlenderOpen ? <InstallBlenderPanel /> : isEmpty ? <EmptyState /> : <AddonPanel />}
                </div>
                {/* Settings and Install Blender take the middle column on their own; the Recent
                    Files column and its toggle come back with the Addons view. */}
                {!isSettingsOpen && !isInstallBlenderOpen && <RecentFiles />}
            </div>
            <LauncherBar />
        </>
    )
}

export default Home

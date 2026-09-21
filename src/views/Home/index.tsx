import { useShallow } from 'zustand/react/shallow';
import RecentFiles from '../../components/RecentFiles';
import LauncherBar from '../../components/LauncherBar/index';
import BlenderColumn from '../../components/BlenderColumn';
import AddonPanel from '../../components/AddonPanel';
import InstallBlenderPanel from '../../components/InstallBlenderPanel';
import SettingsPanel from '../../components/SettingsPanel';
import RestoreSetupPanel from '../../components/RestoreSetupPanel';
import SyncPanel from '../../components/SyncPanel';
import ShareSetupPanel from '../../components/ShareSetupPanel';
import EmptyState from '../../components/EmptyState';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { useEffect } from 'react';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { postStatusError } from '../../store/statusStore';
import { useSetupRestoreStore } from '../../store/setupRestoreStore';
import { useSetupSyncStore } from '../../store/setupSyncStore';
import { SetupService } from '../../services/setupService';
import { useRescanOnFocus } from '../../utility/useRescanOnFocus';

const setupService = new SetupService();
// The startup file is looked at once per process, whatever remounts the view.
let hasCheckedStartupFile = false;

const Home = () => {
    const { isInstallBlenderOpen, isSettingsOpen, isRestoreSetupOpen, isSyncOpen, isShareSetupOpen, setIsInstallBlenderOpen, setIsSettingsOpen, setIsRestoreSetupOpen, setIsSyncOpen, setIsShareSetupOpen } = useUiControlsStore(
        useShallow((s) => ({
            isInstallBlenderOpen: s.isInstallBlenderOpen,
            isSettingsOpen: s.isSettingsOpen,
            isRestoreSetupOpen: s.isRestoreSetupOpen,
            isSyncOpen: s.isSyncOpen,
            isShareSetupOpen: s.isShareSetupOpen,
            setIsInstallBlenderOpen: s.setIsInstallBlenderOpen,
            setIsSettingsOpen: s.setIsSettingsOpen,
            setIsRestoreSetupOpen: s.setIsRestoreSetupOpen,
            setIsSyncOpen: s.setIsSyncOpen,
            setIsShareSetupOpen: s.setIsShareSetupOpen,
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
        load()
            .then(() => useSetupSyncStore.getState().checkForNews())
            .catch((e) => {
                console.error(e);
                postStatusError(`Loading installed Blender versions failed: ${e}`);
            });
        // A .bbsetup opened with the app goes straight to the restore view.
        if (!hasCheckedStartupFile) {
            hasCheckedStartupFile = true;
            setupService.startupSetupFile()
                .then((path) => (path ? useSetupRestoreStore.getState().open(path) : undefined))
                .catch((e) => {
                    console.error(e);
                    postStatusError(`Opening the setup file failed: ${e}`);
                });
        }
    }, []);

    // Escape leaves the Install Blender, Settings, Sync, Restore or What to share view, like
    // closing a dialog; from What to share it goes back to Sync, where it came from.
    useEffect(() => {
        if (!isInstallBlenderOpen && !isSettingsOpen && !isRestoreSetupOpen && !isSyncOpen && !isShareSetupOpen) {
            return;
        }
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                if (isShareSetupOpen) {
                    setIsShareSetupOpen(false);
                    setIsSyncOpen(true);
                    return;
                }
                setIsInstallBlenderOpen(false);
                setIsSettingsOpen(false);
                setIsRestoreSetupOpen(false);
                setIsSyncOpen(false);
            }
        };
        document.addEventListener('keydown', onKeyDown);
        return () => document.removeEventListener('keydown', onKeyDown);
    }, [isInstallBlenderOpen, isSettingsOpen, isRestoreSetupOpen, isSyncOpen, isShareSetupOpen]);

    const isPanelOpen = isSettingsOpen || isRestoreSetupOpen || isSyncOpen || isShareSetupOpen;

    return (
        <>
            <div className={`home ${isInstallBlenderOpen ? 'home--installing' : ''} ${isPanelOpen ? 'home--settings' : ''}`}>
                <BlenderColumn />
                <div className='home__main'>
                    {isShareSetupOpen ? <ShareSetupPanel /> : isRestoreSetupOpen ? <RestoreSetupPanel /> : isSyncOpen ? <SyncPanel /> : isSettingsOpen ? <SettingsPanel /> : isInstallBlenderOpen ? <InstallBlenderPanel /> : isEmpty ? <EmptyState /> : <AddonPanel />}
                </div>
                {/* Settings, Sync, What to share, Restore and Install Blender take the middle column on
                    their own; the Recent Files column and its toggle come back with the Addons view. */}
                {!isPanelOpen && !isInstallBlenderOpen && <RecentFiles />}
            </div>
            <LauncherBar />
        </>
    )
}

export default Home

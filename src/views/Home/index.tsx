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
import StatsPanel from '../../components/StatsPanel';
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
    const { isInstallBlenderOpen, isSettingsOpen, isRestoreSetupOpen, isSyncOpen, isShareSetupOpen, isStatsOpen, setIsInstallBlenderOpen, setIsSettingsOpen, setIsRestoreSetupOpen, setIsSyncOpen, setIsShareSetupOpen, setIsStatsOpen } = useUiControlsStore(
        useShallow((s) => ({
            isInstallBlenderOpen: s.isInstallBlenderOpen,
            isSettingsOpen: s.isSettingsOpen,
            isRestoreSetupOpen: s.isRestoreSetupOpen,
            isSyncOpen: s.isSyncOpen,
            isShareSetupOpen: s.isShareSetupOpen,
            isStatsOpen: s.isStatsOpen,
            setIsInstallBlenderOpen: s.setIsInstallBlenderOpen,
            setIsSettingsOpen: s.setIsSettingsOpen,
            setIsRestoreSetupOpen: s.setIsRestoreSetupOpen,
            setIsSyncOpen: s.setIsSyncOpen,
            setIsShareSetupOpen: s.setIsShareSetupOpen,
            setIsStatsOpen: s.setIsStatsOpen,
        }))
    )
    const { installedBuilds, hasLoadedInstalledBuilds } = useBlenderManagerStore(
        useShallow((s) => ({ installedBuilds: s.installedBuilds, hasLoadedInstalledBuilds: s.hasLoadedInstalledBuilds }))
    )
    const isEmpty = hasLoadedInstalledBuilds && installedBuilds.length === 0;
    useRescanOnFocus();

    // First visit: pick up Blender installed outside Blenderbase (only while no location is
    // registered, so once per fresh install), scan the installation locations on disk, then load
    // the list. Later visits (and StrictMode's second run) only reload; refocusing the window
    // rescans later on.
    useEffect(() => {
        const { hasRefreshedInstalledBuilds, firstLoadInstalledBuilds, setInstalledBuilds } = useBlenderManagerStore.getState();
        const load = hasRefreshedInstalledBuilds ? setInstalledBuilds : firstLoadInstalledBuilds;
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

    // Escape leaves the Install Blender, Settings, Sync, Stats, Restore or What to share view,
    // like closing a dialog; from What to share it goes back to Sync, where it came from.
    useEffect(() => {
        if (!isInstallBlenderOpen && !isSettingsOpen && !isRestoreSetupOpen && !isSyncOpen && !isShareSetupOpen && !isStatsOpen) {
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
                setIsStatsOpen(false);
            }
        };
        document.addEventListener('keydown', onKeyDown);
        return () => document.removeEventListener('keydown', onKeyDown);
    }, [isInstallBlenderOpen, isSettingsOpen, isRestoreSetupOpen, isSyncOpen, isShareSetupOpen, isStatsOpen]);

    const isPanelOpen = isSettingsOpen || isRestoreSetupOpen || isSyncOpen || isShareSetupOpen || isStatsOpen;

    return (
        <>
            <div className={`home ${isInstallBlenderOpen ? 'home--installing' : ''} ${isPanelOpen ? 'home--settings' : ''}`}>
                <BlenderColumn />
                <div className='home__main'>
                    {isShareSetupOpen ? <ShareSetupPanel /> : isRestoreSetupOpen ? <RestoreSetupPanel /> : isSyncOpen ? <SyncPanel /> : isStatsOpen ? <StatsPanel /> : isSettingsOpen ? <SettingsPanel /> : isInstallBlenderOpen ? <InstallBlenderPanel /> : isEmpty ? <EmptyState /> : <AddonPanel />}
                </div>
                {/* Settings, Sync, Stats, What to share, Restore and Install Blender take the middle column
                    on their own; the Recent Files column and its toggle come back with the Addons view. */}
                {!isPanelOpen && !isInstallBlenderOpen && <RecentFiles />}
            </div>
            <LauncherBar />
        </>
    )
}

export default Home

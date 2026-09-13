import { create } from "zustand"

interface IUiControlsStore {
    /** Whether the Recent Files column is shown. */
    isSidebarExpanded: boolean,
    /** Whether the middle column shows the Install Blender (download) view instead of Addons. */
    isInstallBlenderOpen: boolean,
    /** Whether the middle column shows the Settings panel. Never open together with Install Blender. */
    isSettingsOpen: boolean,
    /** The Blender version highlighted in the left column. Falls back to the default version when null. */
    selectedBlenderVersionId: string | null,
    /** Versions that finished installing during this session and have not been selected or launched yet. */
    newlyInstalledBlenderIds: string[],
    /** Launch Blender with its console/terminal visible (Python output). Remembered across sessions. */
    launchWithConsole: boolean,
    setLaunchWithConsole: (v: boolean) => void,
    setIsSidebarExpanded: (v: boolean) => void,
    setIsInstallBlenderOpen: (v: boolean) => void,
    setIsSettingsOpen: (v: boolean) => void,
    setSelectedBlenderVersionId: (id: string | null) => void,
    addNewlyInstalledBlenderId: (id: string) => void,
    clearNewlyInstalledBlenderId: (id: string) => void,
}

const LAUNCH_WITH_CONSOLE_KEY = "blenderbase.launchWithConsole";

const readLaunchWithConsole = (): boolean => {
    try {
        return localStorage.getItem(LAUNCH_WITH_CONSOLE_KEY) === "true";
    } catch {
        return false;
    }
};

export const useUiControlsStore = create<IUiControlsStore>((set) => ({
    isSidebarExpanded: false,
    isInstallBlenderOpen: false,
    isSettingsOpen: false,
    selectedBlenderVersionId: null,
    newlyInstalledBlenderIds: [],
    launchWithConsole: readLaunchWithConsole(),
    setLaunchWithConsole: (v) => {
        try {
            localStorage.setItem(LAUNCH_WITH_CONSOLE_KEY, v ? "true" : "false");
        } catch {
            // Storage may be unavailable; the choice still applies for this session.
        }
        set({ launchWithConsole: v });
    },
    setIsSidebarExpanded: (v) => set({ isSidebarExpanded: v }),
    // The middle column shows one of the two; opening either closes the other.
    setIsInstallBlenderOpen: (v) => set((state) => ({ isInstallBlenderOpen: v, isSettingsOpen: v ? false : state.isSettingsOpen })),
    setIsSettingsOpen: (v) => set((state) => ({ isSettingsOpen: v, isInstallBlenderOpen: v ? false : state.isInstallBlenderOpen })),
    setSelectedBlenderVersionId: (id) => set((state) => ({
        selectedBlenderVersionId: id,
        newlyInstalledBlenderIds: id === null
            ? state.newlyInstalledBlenderIds
            : state.newlyInstalledBlenderIds.filter((x) => x !== id),
    })),
    addNewlyInstalledBlenderId: (id) => set((state) => ({
        newlyInstalledBlenderIds: state.newlyInstalledBlenderIds.includes(id)
            ? state.newlyInstalledBlenderIds
            : [...state.newlyInstalledBlenderIds, id],
    })),
    clearNewlyInstalledBlenderId: (id) => set((state) => ({
        newlyInstalledBlenderIds: state.newlyInstalledBlenderIds.filter((x) => x !== id),
    })),
}))

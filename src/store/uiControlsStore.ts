import { create } from "zustand"

interface IUiControlsStore {
    /** Whether the Recent Files column is shown. */
    isSidebarExpanded: boolean,
    /** Whether the middle column shows the Install Blender (download) view instead of Addons. */
    isInstallBlenderOpen: boolean,
    /** The Blender version highlighted in the left column. Falls back to the default version when null. */
    selectedBlenderVersionId: string | null,
    /** Versions that finished installing during this session and have not been selected or launched yet. */
    newlyInstalledBlenderIds: string[],
    setIsSidebarExpanded: (v: boolean) => void,
    setIsInstallBlenderOpen: (v: boolean) => void,
    setSelectedBlenderVersionId: (id: string | null) => void,
    addNewlyInstalledBlenderId: (id: string) => void,
    clearNewlyInstalledBlenderId: (id: string) => void,
}

export const useUiControlsStore = create<IUiControlsStore>((set) => ({
    isSidebarExpanded: false,
    isInstallBlenderOpen: false,
    selectedBlenderVersionId: null,
    newlyInstalledBlenderIds: [],
    setIsSidebarExpanded: (v) => set({ isSidebarExpanded: v }),
    setIsInstallBlenderOpen: (v) => set({ isInstallBlenderOpen: v }),
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

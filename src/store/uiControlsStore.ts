import { create } from "zustand"

interface IUiControlsStore {
    isSidebarExpanded: boolean,
    selectedLibraryTabIndex: number,
    selectedLibrarySubTabIndex: number,
    setIsSidebarExpanded: (v: boolean) => void,
    setSelectedLibraryTabIndex: (v: number) => void,
    setSelectedLibrarySubTabIndex: (v: number) => void
}

export const useUiControlsStore = create<IUiControlsStore>((set, _get) => ({
    isSidebarExpanded: false,
    setIsSidebarExpanded: (v) => set({ isSidebarExpanded: v }),
    selectedLibraryTabIndex: 0,
    setSelectedLibraryTabIndex: (v) => set({ selectedLibraryTabIndex: v }),
    selectedLibrarySubTabIndex: 0,
    setSelectedLibrarySubTabIndex: (v) => set({ selectedLibrarySubTabIndex: v }),
}))
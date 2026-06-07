import { create } from "zustand"

interface IDisplayInformationStore {
    appVersion: string,
    setAppVersion: (v: string) => void
}

export const useDisplayInformationStore = create<IDisplayInformationStore>((set, _get) => ({
    appVersion: "",
    setAppVersion: (v) => set({ appVersion: v })
}))
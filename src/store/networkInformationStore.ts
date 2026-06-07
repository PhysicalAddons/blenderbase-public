import { create } from "zustand"

interface INetworkInformationStore {
    hasInternetConnection: boolean,
    setHasInternetConnection: (v: boolean | null) => void
}

export const useNetworkInformationStore = create<INetworkInformationStore>((set, _get) => ({
    hasInternetConnection: false,
    setHasInternetConnection: (v) => {
        set({ hasInternetConnection: v != null ? v : false })
    }
}))
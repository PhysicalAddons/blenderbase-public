import { create } from 'zustand'

interface ILoadingManagerStore {
    // Blender
    isRegisteringInstalledBlenderData: boolean,
    isDownloadingBlender: boolean,
    isDeletingBlender: boolean,
    isRegisteringDownloadableBlenderData: boolean,
    // Addons data
    isInstallingExternalAddon: boolean,
    isSymlinkingExternalAddon: boolean,
    isUninstallingExternalAddon: boolean,
    // Settings data
    isCheckingForBlenderbaseVersionUpdate: boolean,
    isChangedBlenderInstallationLocation: boolean,  

    getIsRegisteringInstalledBlenderData: () => boolean
    getIsDownloadingBlender: () => boolean

    getIsDeletingBlender: () => boolean
    getIsRegisteringDownloadableBlenderData: () => boolean

    getIsInstallingExternalAddon: () => boolean
    getIsSymlinkingExternalAddon: () => boolean
    getIsUninstallingExternalAddon: () => boolean

    getIsCheckingForBlenderbaseVersionUpdate: () => boolean
    getIsChangedBlenderInstallationLocation: () => boolean

    setIsRegisteringInstalledBlenderData: (newValue: boolean) => void
    setIsDownloadingBlender: (newValue: boolean) => void

    setIsDeletingBlender: (newValue: boolean) => void
    setIsRegisteringDownloadableBlenderData: (newValue: boolean) => void

    setIsInstallingExternalAddon: (newValue: boolean) => void
    setIsSymlinkingExternalAddon: (newValue: boolean) => void
    setIsUninstallingExternalAddon: (newValue: boolean) => void

    setIsCheckingForBlenderbaseVersionUpdate: (newValue: boolean) => void
    setIsChangedBlenderInstallationLocation: (newValue: boolean) => void

}

export const useLoadingManagerStore = create<ILoadingManagerStore>((set, get) => ({
    // Blender data
    isRegisteringInstalledBlenderData: false,
    isDownloadingBlender: false,
    // isExtractingBlenderArchive: false,
    isDeletingBlender: false,
    isRegisteringDownloadableBlenderData: false,

    // Addons data
    isInstallingExternalAddon: false,
    isSymlinkingExternalAddon: false,
    isUninstallingExternalAddon: false,

    // Settings data
    isCheckingForBlenderbaseVersionUpdate: false,
    isChangedBlenderInstallationLocation: false,    

    // Getters
    getIsRegisteringInstalledBlenderData: () => get().isRegisteringInstalledBlenderData,
    getIsDownloadingBlender: () => get().isDownloadingBlender,
    // getIsExtractingBlenderArchive: () => get().isExtractingBlenderArchive,
    getIsDeletingBlender: () => get().isDeletingBlender,
    getIsRegisteringDownloadableBlenderData: () => get().isRegisteringDownloadableBlenderData,
    
    getIsInstallingExternalAddon: () => get().isInstallingExternalAddon,
    getIsSymlinkingExternalAddon: () => get().isSymlinkingExternalAddon,
    getIsUninstallingExternalAddon: () => get().isUninstallingExternalAddon,

    getIsCheckingForBlenderbaseVersionUpdate: () => get().isCheckingForBlenderbaseVersionUpdate,
    getIsChangedBlenderInstallationLocation: () => get().isChangedBlenderInstallationLocation,

    // Setters
    setIsRegisteringInstalledBlenderData: (newValue) => set((state) => ({...state, isRegisteringInstalledBlenderData: newValue})),
    setIsDownloadingBlender: (newValue) => set((state) => ({...state, isDownloadingBlender: newValue})),
    // getIsExtractingBlenderArchive: (newValue) => set((state) => ({...state, isExtractingBlenderArchive: newValue})),
    setIsDeletingBlender: (newValue) => set((state) => ({...state, isDeletingBlender: newValue})),
    setIsRegisteringDownloadableBlenderData: (newValue) => set((state) => ({...state, isRegisteringDownloadableBlenderData: newValue})),
    
    setIsInstallingExternalAddon: (newValue) => set((state) => ({...state, isInstallingExternalAddon: newValue})),
    setIsSymlinkingExternalAddon: (newValue) => set((state) => ({...state, isSymlinkingExternalAddon: newValue})),
    setIsUninstallingExternalAddon: (newValue) => set((state) => ({...state, isUninstallingExternalAddon: newValue})),

    setIsCheckingForBlenderbaseVersionUpdate: (newValue) => set((state) => ({...state, isCheckingForBlenderbaseVersionUpdate: newValue})),
    setIsChangedBlenderInstallationLocation: (newValue) => set((state) => ({...state, isChangedBlenderInstallationLocation: newValue})),
}));
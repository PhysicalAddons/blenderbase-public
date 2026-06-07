import { create } from "zustand";
import { IBlenderVersion, IBlenderVersionDownloadBuildTypeFilter, IBlenderVersionInstallBuildTypeFilter, IDownloadableBlenderVersionDTO } from "../models";
import { BlenderBuildKind } from "../enums";
import { fromStrBlenderBuildTypeKind } from "../enums/helpers";
import { COMPLETED_LOWERCASE, DESC_LOWERCASE } from "../constants";
import { BlenderService } from "../services/blenderService";

interface IBlenderManagerStore {
    installedBuilds: IBlenderVersion[],
    downloadableBuilds: IDownloadableBlenderVersionDTO,
    activeDownloadBuildType: IBlenderVersionDownloadBuildTypeFilter | null,
    activeInstallBuildType: IBlenderVersionInstallBuildTypeFilter | null,
    setInstalledBuilds: () => Promise<void>,
    setDownloadableBuilds: (build: string) => Promise<void>,
    setActiveDownloadBuildType: (type: IBlenderVersionDownloadBuildTypeFilter | null) => void
    setActiveInstallBuildType: (type: IBlenderVersionInstallBuildTypeFilter | null) => void
}

const blenderService = new BlenderService();

export const useBlenderManagerStore = create<IBlenderManagerStore>((set, _get) => ({
    installedBuilds: [],
    downloadableBuilds: {
        releaseBuilds: [],
        dailyBuilds: [],
        patchBuilds: []
    } as IDownloadableBlenderVersionDTO,
    activeDownloadBuildType: null,
    activeInstallBuildType: null,
    async setInstalledBuilds() {
        try {
            set({ installedBuilds: await blenderService.fetchBlenderVersions(null, null, null, null, null, DESC_LOWERCASE, [COMPLETED_LOWERCASE]) })
        } catch (e) {
            console.error(e)
        }
    },
    async setDownloadableBuilds(build: string) {
        try {
            const data = await blenderService.getDownloadableBlenderVersionData(build, DESC_LOWERCASE);
            set((state) => {
                switch (fromStrBlenderBuildTypeKind(build)) {
                    case BlenderBuildKind.Release:
                        return { downloadableBuilds: { ...state.downloadableBuilds, releaseBuilds: data } }
                    case BlenderBuildKind.Daily:
                        return { downloadableBuilds: { ...state.downloadableBuilds, dailyBuilds: data } }
                    case BlenderBuildKind.Patch:
                        return { downloadableBuilds: { ...state.downloadableBuilds, patchBuilds: data } }
                    default:
                        return state
                }
            })
        } catch (e) {
            console.error(e);
        }
    },
    setActiveDownloadBuildType: (type) => set({ activeDownloadBuildType: type }),
    setActiveInstallBuildType: (type) => set({ activeInstallBuildType: type })
}));
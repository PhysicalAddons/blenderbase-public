import { create } from "zustand";
import { IBlenderVersion, IBlenderVersionDownloadBuildTypeFilter, IBlenderVersionInstallBuildTypeFilter, IDownloadableBlenderVersionDTO } from "../models";
import { BlenderBuildKind } from "../enums";
import { fromStrBlenderBuildTypeKind } from "../enums/helpers";
import { COMPLETED_LOWERCASE, DESC_LOWERCASE } from "../constants";
import { BlenderService } from "../services/blenderService";
import { postStatus } from "./statusStore";

interface IBlenderManagerStore {
    installedBuilds: IBlenderVersion[],
    /** False until the installed list has been fetched once. */
    hasLoadedInstalledBuilds: boolean,
    /** Ids already asked for their build details this session (probed once, success or not). */
    probedDetailIds: string[],
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
    hasLoadedInstalledBuilds: false,
    probedDetailIds: [],
    downloadableBuilds: {
        releaseBuilds: [],
        dailyBuilds: [],
        patchBuilds: []
    } as IDownloadableBlenderVersionDTO,
    activeDownloadBuildType: null,
    activeInstallBuildType: null,
    async setInstalledBuilds() {
        try {
            const builds = await blenderService.fetchBlenderVersions(null, null, null, null, null, DESC_LOWERCASE, [COMPLETED_LOWERCASE]);
            set({ installedBuilds: builds, hasLoadedInstalledBuilds: true });
            // Versions installed without download data have no date or hash yet; ask each build
            // once (in the background) and refresh the list when the answers are in.
            const probed = _get().probedDetailIds;
            const missing = builds.filter((b) => (!b.hash || b.file_mtime === 0) && !probed.includes(b.id)).map((b) => b.id);
            if (missing.length > 0) {
                set({ probedDetailIds: [...probed, ...missing] });
                postStatus(`Reading build details of ${missing.length} Blender ${missing.length === 1 ? "version" : "versions"}…`, true);
                blenderService.refreshBlenderVersionDetails(missing)
                    .then(async () => {
                        set({ installedBuilds: await blenderService.fetchBlenderVersions(null, null, null, null, null, DESC_LOWERCASE, [COMPLETED_LOWERCASE]) });
                        postStatus("Build details updated");
                    })
                    .catch((e) => console.error(e));
            }
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
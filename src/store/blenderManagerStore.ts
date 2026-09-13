import { create } from "zustand";
import { IBlenderVersion, IBlenderVersionDownloadBuildTypeFilter, IBlenderVersionInstallBuildTypeFilter, IDownloadableBlenderVersionDTO } from "../models";
import { BlenderBuildKind } from "../enums";
import { fromStrBlenderBuildTypeKind } from "../enums/helpers";
import { COMPLETED_LOWERCASE, DESC_LOWERCASE } from "../constants";
import { BlenderService } from "../services/blenderService";
import { postStatus, postStatusError } from "./statusStore";

interface IBlenderManagerStore {
    installedBuilds: IBlenderVersion[],
    /** False until the installed list has been fetched once. */
    hasLoadedInstalledBuilds: boolean,
    /** True once the disk scan has run this session (guards the Home mount effect). */
    hasRefreshedInstalledBuilds: boolean,
    /** Ids already asked for their build details this session (probed once, success or not). */
    probedDetailIds: string[],
    downloadableBuilds: IDownloadableBlenderVersionDTO,
    activeDownloadBuildType: IBlenderVersionDownloadBuildTypeFilter | null,
    activeInstallBuildType: IBlenderVersionInstallBuildTypeFilter | null,
    /** Read-only: loads the versions already registered. Rejects on failure. */
    setInstalledBuilds: () => Promise<void>,
    /** Rescans the installation locations on disk, then loads the list. Rejects on failure. */
    refreshInstalledBuilds: () => Promise<void>,
    /** Rejects on failure. */
    setDownloadableBuilds: (build: string) => Promise<void>,
    setActiveDownloadBuildType: (type: IBlenderVersionDownloadBuildTypeFilter | null) => void
    setActiveInstallBuildType: (type: IBlenderVersionInstallBuildTypeFilter | null) => void
}

const blenderService = new BlenderService();

const fetchInstalled = () =>
    blenderService.fetchBlenderVersions(null, null, null, null, null, DESC_LOWERCASE, [COMPLETED_LOWERCASE]);

export const useBlenderManagerStore = create<IBlenderManagerStore>((set, get) => ({
    installedBuilds: [],
    hasLoadedInstalledBuilds: false,
    hasRefreshedInstalledBuilds: false,
    probedDetailIds: [],
    downloadableBuilds: {
        releaseBuilds: [],
        dailyBuilds: [],
        patchBuilds: []
    } as IDownloadableBlenderVersionDTO,
    activeDownloadBuildType: null,
    activeInstallBuildType: null,
    async setInstalledBuilds() {
        const builds = await fetchInstalled();
        set({ installedBuilds: builds, hasLoadedInstalledBuilds: true });
        // Versions installed without download data have no date or hash yet; ask each build
        // once (in the background) and refresh the list when the answers are in.
        const probed = get().probedDetailIds;
        const missing = builds.filter((b) => (!b.hash || b.file_mtime === 0) && !probed.includes(b.id)).map((b) => b.id);
        if (missing.length > 0) {
            set({ probedDetailIds: [...probed, ...missing] });
            postStatus(`Reading build details of ${missing.length} Blender ${missing.length === 1 ? "version" : "versions"}…`, true);
            blenderService.refreshBlenderVersionDetails(missing)
                .then(async () => {
                    set({ installedBuilds: await fetchInstalled() });
                    postStatus("Build details updated");
                })
                .catch((e) => {
                    console.error(e);
                    postStatusError(`Reading build details failed: ${e}`);
                });
        }
    },
    async refreshInstalledBuilds() {
        // Set before awaiting so a second mount (StrictMode) does not start a second scan.
        set({ hasRefreshedInstalledBuilds: true });
        await blenderService.refreshBlenderVersions();
        await get().setInstalledBuilds();
    },
    async setDownloadableBuilds(build: string) {
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
    },
    setActiveDownloadBuildType: (type) => set({ activeDownloadBuildType: type }),
    setActiveInstallBuildType: (type) => set({ activeInstallBuildType: type })
}));

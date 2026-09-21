import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { join } from "@tauri-apps/api/path";
import {
    IBlenderInstallationLocation, IBlenderVersion, IDownloadableBlenderVersion, ISeriesApplyChoice, ISeriesApplyReport,
    ISetupBlenderVersion, ISetupBundleInfo, ISetupSeries,
} from "../models";
import { COMPLETED_LOWERCASE, DAILY_LOWERCASE, DESC_LOWERCASE, DOWNLOADING_LOWERCASE, FAILED_LOWERCASE, PATCH_LOWERCASE, RELEASE_LOWERCASE } from "../constants";
import { BlenderService } from "../services/blenderService";
import { SettingsService } from "../services/settingsService";
import { SetupService } from "../services/setupService";
import { WebUtilityService } from "../services/webUtilityService";
import { downloadFile } from "../utility";
import { IBuildLists, IResolvedVersion, needsBuilderListings, resolveDownloadableBuild } from "../utility/setupVersions";
import { useBlenderManagerStore } from "./blenderManagerStore";
import { useNetworkInformationStore } from "./networkInformationStore";
import { postStatus, postStatusError } from "./statusStore";
import { useUiControlsStore } from "./uiControlsStore";

export type MissingVersionState = "looking" | "ready" | "unavailable" | "downloading" | "installing" | "installed" | "failed";

/** A Blender version the setup names that is not installed here. */
export interface IMissingVersion {
    /** Version and branch together: a daily and a patch build can share a version number. */
    id: string,
    wanted: ISetupBlenderVersion,
    resolved: IResolvedVersion | null,
    state: MissingVersionState,
    /** The row's line: the resolver's note, or what went wrong. */
    message: string,
}

/** Asks where Blender versions go before the first download, as the Install Blender view does. */
export interface ILocationPrompt {
    location: IBlenderInstallationLocation | null,
    path: string,
    /** The missing entry to install once the folder is confirmed. */
    id: string,
    buttonId: string,
}

/**
 * A setup file opened for restoring: what it holds, what the user ticked per series, and what
 * happened when it was applied.
 */
interface ISetupRestoreStore {
    info: ISetupBundleInfo | null,
    choices: ISeriesApplyChoice[],
    reports: ISeriesApplyReport[],
    isBusy: boolean,
    missing: IMissingVersion[],
    locationPrompt: ILocationPrompt | null,
    /** Looks each missing version up in the download listings; needs the internet. */
    resolveMissingVersions: () => Promise<void>,
    /** Looks up again the versions that have no answer yet (after the connection came back). */
    retryLookup: () => Promise<void>,
    /** Downloads and installs one missing version; `buttonId` names the element that shows progress. */
    installMissingVersion: (id: string, buttonId: string) => Promise<void>,
    installAllMissing: () => Promise<void>,
    setLocationPromptPath: (path: string) => void,
    /** Registers or confirms the prompted folder, then installs the version the prompt was for. */
    confirmLocationPrompt: () => Promise<void>,
    cancelLocationPrompt: () => void,
    /** Reads the file, then shows the restore view. Rejects when the file cannot be read. */
    open: (filePath: string) => Promise<void>,
    close: () => void,
    setChoice: (series: string, patch: Partial<Omit<ISeriesApplyChoice, "series">>) => void,
    /** Applies the ticked series; the backend refuses while a Blender is open. */
    apply: () => Promise<void>,
    undo: (series: string) => Promise<void>,
}

const setupService = new SetupService();
const blenderService = new BlenderService();
const settingsService = new SettingsService();
const webUtilityService = new WebUtilityService();

export const missingVersionId = (v: ISetupBlenderVersion): string => `${v.version}|${v.branch ?? ""}`;

/** The element id of a missing row's button, where the download progress is written. */
export const missingButtonId = (id: string): string => `restore-install-${id.replace(/[^A-Za-z0-9]+/g, "-")}`;

const versionLabel = (v: ISetupBlenderVersion): string => `Blender ${v.version}${v.channel && v.channel !== "stable" ? ` ${v.channel.toUpperCase() === "LTS" ? "LTS" : v.channel}` : ""}`;

/** The versions of the setup that are not installed here yet. */
const missingOf = (info: ISetupBundleInfo, installed: IBlenderVersion[]): IMissingVersion[] =>
    info.manifest.blender
        .filter((v) => !installed.some((b) => b.version === v.version))
        .map((wanted) => ({ id: missingVersionId(wanted), wanted, resolved: null, state: "looking" as MissingVersionState, message: "Looking up the download…" }));

// Backend errors arrive as "cmd_name: reason"; the status line only needs the reason.
const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e)).replace(/^cmd_\w+: /, "");

/** Addons a series can restore on its own: from a repository, or from a file the setup holds. */
export const restorableAddons = (section: ISetupSeries) =>
    section.addons.filter((a) => a.source === "repo" || (a.source === "file" && a.file !== undefined));

export const useSetupRestoreStore = create<ISetupRestoreStore>((set, get) => ({
    info: null,
    choices: [],
    reports: [],
    isBusy: false,
    missing: [],
    locationPrompt: null,
    async open(filePath) {
        const info = await setupService.inspectSetupBundle(filePath);
        // Everything a series holds is ticked to start with; the row shows what is missing.
        const choices = Object.entries(info.manifest.series).map(([series, section]) => ({
            series,
            preferences: Boolean(section.preferences),
            theme: Boolean(section.theme),
            keymap: Boolean(section.keymap),
            addons: restorableAddons(section).length > 0,
        }));
        set({ info, choices, reports: [], missing: missingOf(info, useBlenderManagerStore.getState().installedBuilds), locationPrompt: null });
        useUiControlsStore.getState().setIsRestoreSetupOpen(true);
        void get().resolveMissingVersions();
    },
    close() {
        useUiControlsStore.getState().setIsRestoreSetupOpen(false);
        set({ info: null, choices: [], reports: [], missing: [], locationPrompt: null });
    },
    async resolveMissingVersions() {
        const { missing } = get();
        if (missing.length === 0) {
            return;
        }
        // The app's own check may not have run yet when a file opens at startup. This is an
        // explicit lookup, so the check runs now instead of waiting for its cooldown.
        if (!useNetworkInformationStore.getState().hasInternetConnection) {
            const online = await webUtilityService.checkInternetConnection(true);
            if (online === true) {
                useNetworkInformationStore.getState().setHasInternetConnection(true);
            } else {
                set((state) => ({ missing: state.missing.map((m) => (m.state === "looking" ? { ...m, state: "unavailable", message: "No internet connection" } : m)) }));
                return;
            }
        }
        try {
            // The release listing answers most versions; the builder listings only when a
            // pre-release or patch build is wanted.
            const releases = await blenderService.getDownloadableBlenderVersionData(RELEASE_LOWERCASE, DESC_LOWERCASE);
            const wantsBuilder = missing.some((m) => needsBuilderListings(m.wanted) || !releases.some((b) => b.version === m.wanted.version));
            const [dailies, patches] = wantsBuilder
                ? await Promise.all([
                    blenderService.getDownloadableBlenderVersionData(DAILY_LOWERCASE, DESC_LOWERCASE),
                    blenderService.getDownloadableBlenderVersionData(PATCH_LOWERCASE, DESC_LOWERCASE),
                ])
                : [[], []];
            const lists: IBuildLists = { releases, dailies, patches };
            set((state) => ({
                missing: state.missing.map((m) => {
                    if (m.state !== "looking") {
                        return m;
                    }
                    const resolved = resolveDownloadableBuild(m.wanted, lists);
                    return { ...m, resolved, state: resolved.build ? "ready" : "unavailable", message: resolved.note };
                }),
            }));
        } catch (e) {
            console.error(e);
            set((state) => ({ missing: state.missing.map((m) => (m.state === "looking" ? { ...m, state: "unavailable", message: `Could not read the download listings: ${errorText(e)}` } : m)) }));
        }
    },
    async retryLookup() {
        set((state) => ({ missing: state.missing.map((m) => (m.resolved === null ? { ...m, state: "looking", message: "Looking up the download…" } : m)) }));
        await get().resolveMissingVersions();
    },
    async installMissingVersion(id, buttonId) {
        const entry = get().missing.find((m) => m.id === id);
        const build = entry?.resolved?.build;
        if (!entry || !build || entry.state !== "ready") {
            return;
        }
        let location: IBlenderInstallationLocation | undefined;
        try {
            const locations = await settingsService.fetchBlenderInstallationPaths(null, null, null, true);
            location = locations[0];
            if (!location) {
                const path = await settingsService.defaultInstallationDirectory();
                set({ locationPrompt: { location: null, path, id, buttonId } });
                return;
            }
            if (!location.is_confirmed) {
                set({ locationPrompt: { location, path: location.directory_path, id, buttonId } });
                return;
            }
        } catch (e) {
            console.error(e);
            postStatusError(`Could not start the download: ${errorText(e)}`);
            return;
        }
        await installResolved(entry, build, location, buttonId, set);
    },
    async installAllMissing() {
        // One after the other: downloads share the connection and installs unpack big archives.
        for (const m of get().missing) {
            if (m.state === "ready") {
                await get().installMissingVersion(m.id, missingButtonId(m.id));
                if (get().locationPrompt) {
                    return;
                }
            }
        }
    },
    setLocationPromptPath(path) {
        set((state) => (state.locationPrompt ? { locationPrompt: { ...state.locationPrompt, path } } : {}));
    },
    async confirmLocationPrompt() {
        const prompt = get().locationPrompt;
        if (!prompt) {
            return;
        }
        try {
            const confirmed = prompt.location
                ? await settingsService.confirmBlenderInstallationLocation(prompt.location.id, prompt.path)
                : await settingsService.registerBlenderInstallationLocation(prompt.path);
            set({ locationPrompt: null });
            const entry = get().missing.find((m) => m.id === prompt.id);
            const build = entry?.resolved?.build;
            if (entry && build) {
                await installResolved(entry, build, confirmed, prompt.buttonId, set);
            }
        } catch (e) {
            console.error(e);
            postStatusError(`Could not use that folder: ${errorText(e)}`);
        }
    },
    cancelLocationPrompt() {
        set({ locationPrompt: null });
    },
    setChoice(series, patch) {
        set((state) => ({ choices: state.choices.map((c) => (c.series === series ? { ...c, ...patch } : c)) }));
    },
    async apply() {
        const { info, choices } = get();
        if (!info) {
            return;
        }
        const chosen = choices.filter((c) => c.preferences || c.theme || c.keymap || c.addons);
        if (chosen.length === 0) {
            postStatusError("Nothing is selected to apply");
            return;
        }
        set({ isBusy: true, reports: [] });
        postStatus("Applying the setup…", true);
        const stopListening = await listen<string>("setup-progress", (event) => postStatus(event.payload, true));
        try {
            const reports = await setupService.applySetupBundle(info.file_path, chosen);
            set({ reports });
            const applied = reports.filter((r) => !r.skipped_reason).length;
            const skipped = reports.length - applied;
            postStatus(`Setup applied to ${applied} Blender ${applied === 1 ? "series" : "series"}${skipped > 0 ? `, ${skipped} skipped` : ""}`);
        } catch (e) {
            console.error(e);
            postStatusError(`Applying the setup failed: ${errorText(e)}`);
        } finally {
            stopListening();
            set({ isBusy: false });
        }
    },
    async undo(series) {
        set({ isBusy: true });
        postStatus(`Restoring the previous configuration of Blender ${series}…`, true);
        try {
            const restored = await setupService.undoSetupApply(series);
            set((state) => ({ reports: state.reports.filter((r) => r.series !== series) }));
            postStatus(`Blender ${series}: previous configuration restored (${restored} ${restored === 1 ? "file" : "files"})`);
        } catch (e) {
            console.error(e);
            postStatusError(`Undo failed: ${errorText(e)}`);
        } finally {
            set({ isBusy: false });
        }
    },
}));

type SetState = (partial: Partial<ISetupRestoreStore> | ((state: ISetupRestoreStore) => Partial<ISetupRestoreStore>)) => void;

/** The same steps the Install Blender view takes, in order: register, download, install, record. */
async function installResolved(entry: IMissingVersion, build: IDownloadableBlenderVersion, location: IBlenderInstallationLocation, buttonId: string, set: SetState): Promise<void> {
    const label = versionLabel(entry.wanted);
    const setState = (state: MissingVersionState, message?: string) =>
        set((s) => ({ missing: s.missing.map((m) => (m.id === entry.id ? { ...m, state, message: message ?? m.message } : m)) }));
    setState("downloading");
    try {
        const blenderVersion: IBlenderVersion = await invoke("cmd_init_blender_version", { downloadableBlenderVersion: build, blenderInstallationLocation: location });
        const archiveFilePath = await join(location.directory_path, build.file_name);
        postStatus(`Downloading ${label}…`, true);
        await blenderService.updateBlenderVersionDownloadStatusType(blenderVersion, DOWNLOADING_LOWERCASE);
        const downloaded = await downloadFile(build.url, archiveFilePath, buttonId, (percent) => postStatus(`Downloading ${label} · ${percent}%`, true));
        if (!downloaded) {
            await blenderService.updateBlenderVersionDownloadStatusType(blenderVersion, FAILED_LOWERCASE);
            setState("failed", "Download failed");
            postStatusError(`Download of ${label} failed`);
            return;
        }
        await blenderService.updateBlenderVersionDownloadStatusType(blenderVersion, COMPLETED_LOWERCASE);
        setState("installing");
        postStatus(`Installing ${label}…`, true);
        const installedDirectory = await blenderService.installBlenderVersion(blenderVersion.id, archiveFilePath);
        await blenderService.writeBlenderVersionDownloadData(build, installedDirectory);
        useUiControlsStore.getState().addNewlyInstalledBlenderId(blenderVersion.id);
        await useBlenderManagerStore.getState().setInstalledBuilds();
        setState("installed", `Installed ${build.version}`);
        postStatus(`Installed ${label}`);
    } catch (e) {
        console.error(e);
        setState("failed", `Install failed: ${errorText(e)}`);
        postStatusError(`Installing ${label} failed: ${errorText(e)}`);
    }
}

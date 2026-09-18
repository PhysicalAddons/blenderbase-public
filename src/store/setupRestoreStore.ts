import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import { ISeriesApplyChoice, ISeriesApplyReport, ISetupBundleInfo, ISetupSeries } from "../models";
import { SetupService } from "../services/setupService";
import { postStatus, postStatusError } from "./statusStore";
import { useUiControlsStore } from "./uiControlsStore";

/**
 * A setup file opened for restoring: what it holds, what the user ticked per series, and what
 * happened when it was applied.
 */
interface ISetupRestoreStore {
    info: ISetupBundleInfo | null,
    choices: ISeriesApplyChoice[],
    reports: ISeriesApplyReport[],
    isBusy: boolean,
    /** Reads the file, then shows the restore view. Rejects when the file cannot be read. */
    open: (filePath: string) => Promise<void>,
    close: () => void,
    setChoice: (series: string, patch: Partial<Omit<ISeriesApplyChoice, "series">>) => void,
    /** Applies the ticked series; the backend refuses while a Blender is open. */
    apply: () => Promise<void>,
    undo: (series: string) => Promise<void>,
}

const setupService = new SetupService();

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
        set({ info, choices, reports: [] });
        useUiControlsStore.getState().setIsRestoreSetupOpen(true);
    },
    close() {
        useUiControlsStore.getState().setIsRestoreSetupOpen(false);
        set({ info: null, choices: [], reports: [] });
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

import { create } from "zustand";
import { IBlenderSeries, IBlendFile } from "../models"
import { BlendFileService } from "../services/blendFileService";
import { DESC_LOWERCASE } from "../constants";
import { postStatusError } from "./statusStore";

interface IBlendFileStore {
    blenderSeries: IBlenderSeries[],
    /** Recent files grouped by the Blender series they belong to, keyed by series id. */
    blendFilesBySeries: Record<string, IBlendFile[]>,
    /** True once the recent-files import has run this session (guards the panel's mount effect). */
    hasRefreshedBlendFiles: boolean,
    /** Read-only: loads the series list. Failures are reported to the status line. */
    setBlenderSeries: () => Promise<void>,
    /** Imports the recent files from disk, then loads the series list and their files. */
    refreshBlendFiles: () => Promise<void>,
    /** Read-only: loads the files of one series. Failures are reported to the status line. */
    setBlendFiles: (blenderSeriesId: string) => Promise<void>,
    /** Persists the collapsed state of a series and mirrors it in the store. */
    setSeriesCollapsed: (blenderSeriesId: string, isCollapsed: boolean) => Promise<void>,
}

const blendFileService = new BlendFileService();

export const useBlendFileStore = create<IBlendFileStore>((set, get) => ({
    blenderSeries: [],
    blendFilesBySeries: {},
    hasRefreshedBlendFiles: false,
    async setBlenderSeries() {
        try {
            const series = await blendFileService.fetchBlenderSeries(null, null, null, true, DESC_LOWERCASE);
            // One entry per series, even if the backend reports a series once per mapped file.
            const unique = series.filter((s, i) => series.findIndex((x) => x.id === s.id) === i);
            set({ blenderSeries: unique });
        } catch (e) {
            console.error(e);
            postStatusError(`Loading recent files failed: ${e}`);
        }
    },
    async refreshBlendFiles() {
        set({ hasRefreshedBlendFiles: true });
        try {
            await blendFileService.refreshRecentFiles();
        } catch (e) {
            console.error(e);
            postStatusError(`Importing recent files failed: ${e}`);
        }
        await get().setBlenderSeries();
        // Files already shown for a series are stale after an import.
        await Promise.all(Object.keys(get().blendFilesBySeries).map((id) => get().setBlendFiles(id)));
    },
    async setBlendFiles(blenderSeriesId: string) {
        try {
            const files = await blendFileService.fetchBlendFiles(null, null, null, blenderSeriesId, DESC_LOWERCASE);
            set((state) => ({ blendFilesBySeries: { ...state.blendFilesBySeries, [blenderSeriesId]: files } }));
        } catch (e) {
            console.error(e);
            postStatusError(`Loading recent files failed: ${e}`);
        }
    },
    async setSeriesCollapsed(blenderSeriesId, isCollapsed) {
        const current = get().blenderSeries.find((s) => s.id === blenderSeriesId);
        if (!current) {
            return;
        }
        const updated = { ...current, is_collapsed: isCollapsed };
        try {
            await blendFileService.updateBlenderSeries(updated);
            set((state) => ({ blenderSeries: state.blenderSeries.map((s) => (s.id === blenderSeriesId ? updated : s)) }));
        } catch (e) {
            console.error(e);
            postStatusError(`Saving the recent files layout failed: ${e}`);
        }
    },
}));

import { create } from "zustand";
import { IBlenderSeries, IBlendFile } from "../models"
import { BlendFileService } from "../services/blendFileService";
import { DESC_LOWERCASE } from "../constants";

interface IBlendFileStore {
    blenderSeries: IBlenderSeries[],
    /** Recent files grouped by the Blender series they belong to, keyed by series id. */
    blendFilesBySeries: Record<string, IBlendFile[]>,
    setBlenderSeries: () => Promise<void>,
    setBlendFiles: (blenderSeriesId: string) => Promise<void>,
}

const blendFileService = new BlendFileService();

export const useBlendFileStore = create<IBlendFileStore>((set) => ({
    blenderSeries: [],
    blendFilesBySeries: {},
    async setBlenderSeries() {
        try {
            const series = await blendFileService.fetchBlenderSeries(null, null, null, true, DESC_LOWERCASE);
            // One entry per series, even if the backend reports a series once per mapped file.
            const unique = series.filter((s, i) => series.findIndex((x) => x.id === s.id) === i);
            set({ blenderSeries: unique });
        } catch (e) {
            console.error(e);
        }
    },
    async setBlendFiles(blenderSeriesId: string) {
        try {
            const files = await blendFileService.fetchBlendFiles(null, null, null, blenderSeriesId, DESC_LOWERCASE);
            set((state) => ({ blendFilesBySeries: { ...state.blendFilesBySeries, [blenderSeriesId]: files } }));
        } catch (e) {
            console.error(e);
        }
    }
}));

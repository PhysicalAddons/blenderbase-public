import { create } from "zustand";
import { IBlenderSeries, IBlendFile } from "../models"
import { BlendFileService } from "../services/blendFileService";
import { DESC_LOWERCASE } from "../constants";

interface IBlendFileStore {
    blenderSeries: IBlenderSeries[],
    blendFiles: IBlendFile[],
    setBlenderSeries: () => Promise<void>,
    setBlendFiles: (blenderSeriesId: string) => Promise<void>,
}

const blendFileService = new BlendFileService();

export const useBlendFileStore = create<IBlendFileStore>((set, _get) => ({
    blenderSeries: [],
    blendFiles: [],
    async setBlenderSeries() {
        try {
            set({ blenderSeries: await blendFileService.fetchBlenderSeries(null, null, null, true, DESC_LOWERCASE) });
        } catch (e) {
            console.error(e);
        }
    },
    async setBlendFiles(blenderSeriesId: string) {
        try {
            set({ blendFiles: await blendFileService.fetchBlendFiles(null, null, null, blenderSeriesId, DESC_LOWERCASE) });
        } catch (e) {
            console.error(e);
        }
    }

}));
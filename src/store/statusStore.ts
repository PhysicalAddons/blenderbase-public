import { create } from "zustand";

/**
 * One line of status for the footer: what is running right now, or the last thing that finished.
 */
interface IStatusStore {
    message: string,
    isBusy: boolean,
    isError: boolean,
    updatedAt: number,
    setStatus: (message: string, isBusy?: boolean, isError?: boolean) => void,
}

export const useStatusStore = create<IStatusStore>((set) => ({
    message: "",
    isBusy: false,
    isError: false,
    updatedAt: 0,
    setStatus: (message, isBusy = false, isError = false) => set({ message, isBusy, isError, updatedAt: Date.now() }),
}));

/** Posts a status line from anywhere (stores, services, handlers). */
export const postStatus = (message: string, isBusy: boolean = false): void => {
    useStatusStore.getState().setStatus(message, isBusy, false);
}

/** Posts a failure line; the footer shows it until the next status. */
export const postStatusError = (message: string): void => {
    useStatusStore.getState().setStatus(message, false, true);
}

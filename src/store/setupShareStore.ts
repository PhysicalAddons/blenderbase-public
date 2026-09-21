import { create } from "zustand";
import { IBlenderVersion, ISeriesShareChoice, ISetupExportOptions, ISetupShareSelection } from "../models";

/**
 * What goes into a shared setup, whichever way it travels: which installed Blender versions,
 * and per series which parts and which addons. Everything goes until the user unticks it, so
 * the selection records what is left out. Remembered across sessions on this computer.
 */
interface ISetupShareStore {
    selection: ISetupShareSelection,
    setIncludeAddonFiles: (on: boolean) => void,
    setVersionIncluded: (versionId: string, on: boolean) => void,
    setSeriesPart: (series: string, part: SharePart, on: boolean) => void,
    setAddonIncluded: (series: string, addonKey: string, on: boolean) => void,
    /** Back to everything. */
    reset: () => void,
}

export type SharePart = "preferences" | "theme" | "keymap" | "addons";

const STORAGE_KEY = "blenderbase.setupShare";

const EVERYTHING: ISetupShareSelection = { include_addon_files: false, excluded_version_ids: [], series: {} };

export const wholeSeries = (): ISeriesShareChoice => ({ preferences: true, theme: true, keymap: true, addons: true, excluded_addons: [] });

const isWhole = (choice: ISeriesShareChoice): boolean =>
    choice.preferences && choice.theme && choice.keymap && choice.addons && choice.excluded_addons.length === 0;

const readSelection = (): ISetupShareSelection => {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw) {
            return EVERYTHING;
        }
        const parsed = JSON.parse(raw) as Partial<ISetupShareSelection>;
        const series: Record<string, ISeriesShareChoice> = {};
        Object.entries(parsed.series ?? {}).forEach(([key, value]) => {
            series[key] = { ...wholeSeries(), ...value, excluded_addons: Array.isArray(value?.excluded_addons) ? value.excluded_addons : [] };
        });
        return {
            include_addon_files: Boolean(parsed.include_addon_files),
            excluded_version_ids: Array.isArray(parsed.excluded_version_ids) ? parsed.excluded_version_ids.map(String) : [],
            series,
        };
    } catch {
        return EVERYTHING;
    }
};

const writeSelection = (selection: ISetupShareSelection): void => {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(selection));
    } catch {
        // A private window or blocked storage: the choice lasts for the session.
    }
};

/** The choice for a series, whole when none was recorded. */
export const seriesChoice = (selection: ISetupShareSelection, series: string): ISeriesShareChoice => selection.series[series] ?? wholeSeries();

/** Nothing left out: the whole setup goes. */
export const isEverything = (selection: ISetupShareSelection): boolean =>
    selection.excluded_version_ids.length === 0 && Object.values(selection.series).every(isWhole);

/** What the backend receives with every save, send or share. */
export const exportOptions = (selection: ISetupShareSelection): ISetupExportOptions => ({
    include_addon_files: selection.include_addon_files,
    excluded_version_ids: selection.excluded_version_ids,
    series: selection.series,
});

/** One line for the Sync view: "Everything: 17 Blender versions, 9 series", or what is left. */
export const describeShare = (selection: ISetupShareSelection, installed: IBlenderVersion[]): string => {
    const seriesOf = (versions: IBlenderVersion[]): Set<string> => new Set(versions.map((v) => v.series ?? "").filter((s) => s.length > 0));
    const included = installed.filter((v) => !selection.excluded_version_ids.includes(v.id));
    const allSeries = seriesOf(installed);
    const includedSeries = seriesOf(included);
    const trimmed = [...includedSeries].filter((s) => !isWhole(seriesChoice(selection, s))).length;
    const files = selection.include_addon_files ? " · addon files included" : "";
    const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;
    if (included.length === installed.length && trimmed === 0) {
        return `Everything: ${plural(installed.length, "Blender version", "Blender versions")}, ${plural(allSeries.size, "series", "series")}${files}`;
    }
    const parts = [
        `${included.length} of ${plural(installed.length, "Blender version", "Blender versions")}`,
        `${includedSeries.size} of ${plural(allSeries.size, "series", "series")}`,
    ];
    if (trimmed > 0) {
        parts.push(`parts of ${plural(trimmed, "series", "series")} left out`);
    }
    return parts.join(" · ") + files;
};

export const useSetupShareStore = create<ISetupShareStore>((set, get) => {
    const update = (change: (selection: ISetupShareSelection) => ISetupShareSelection) => {
        const next = change(get().selection);
        // Series with nothing left out are dropped, so "everything" stays easy to tell.
        const series: Record<string, ISeriesShareChoice> = {};
        Object.entries(next.series).forEach(([key, choice]) => {
            if (!isWhole(choice)) {
                series[key] = choice;
            }
        });
        const pruned = { ...next, series };
        writeSelection(pruned);
        set({ selection: pruned });
    };
    return {
        selection: readSelection(),
        setIncludeAddonFiles: (on) => update((s) => ({ ...s, include_addon_files: on })),
        setVersionIncluded: (versionId, on) => update((s) => ({
            ...s,
            excluded_version_ids: on ? s.excluded_version_ids.filter((id) => id !== versionId) : [...new Set([...s.excluded_version_ids, versionId])],
        })),
        setSeriesPart: (series, part, on) => update((s) => ({
            ...s,
            series: { ...s.series, [series]: { ...seriesChoice(s, series), [part]: on } },
        })),
        setAddonIncluded: (series, addonKey, on) => update((s) => {
            const choice = seriesChoice(s, series);
            const excluded = on ? choice.excluded_addons.filter((k) => k !== addonKey) : [...new Set([...choice.excluded_addons, addonKey])];
            return { ...s, series: { ...s.series, [series]: { ...choice, excluded_addons: excluded } } };
        }),
        reset: () => update((s) => ({ ...EVERYTHING, include_addon_files: s.include_addon_files })),
    };
});

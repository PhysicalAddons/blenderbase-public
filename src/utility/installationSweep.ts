import { IBlenderInstallationSweep, ISweptBlenderLocation } from "../models";

/** "a", "a and b", "a, b and c" */
const joinList = (items: string[]): string =>
    items.length <= 1 ? items.join("") : `${items.slice(0, -1).join(", ")} and ${items[items.length - 1]}`;

const versions = (count: number): string => `${count} Blender ${count === 1 ? "version" : "versions"}`;

/** "3 Blender versions in Program Files and 1 in Steam" */
const describeFound = (locations: ISweptBlenderLocation[]): string =>
    joinList(locations.map((l, i) => `${i === 0 ? versions(l.version_count) : l.version_count} in ${l.label}`));

/**
 * The status line for a sweep of the usual Blender folders, or null when the sweep did not run.
 * On first launch a miss also points at Settings, since the empty state is what the user sees next.
 */
export const sweepStatusMessage = (sweep: IBlenderInstallationSweep, isFirstLaunch: boolean): string | null => {
    if (sweep.skipped) {
        return null;
    }
    const added = sweep.locations.filter((l) => l.is_new);
    if (added.length > 0) {
        return `Found ${describeFound(added)}`;
    }
    if (sweep.locations.length > 0) {
        return `Blender in ${joinList(sweep.locations.map((l) => l.label))} is already registered`;
    }
    return isFirstLaunch
        ? "No Blender installs found in the usual folders. Installs elsewhere can be added under Settings › Locations."
        : "No Blender installs found in the usual folders";
};

import { download } from "@tauri-apps/plugin-upload";
import { IAddon, IBlenderVersion, IDownloadableBlenderVersion } from "../models";

export async function downloadFile(url: string, filePath: string, buttonId: string, onProgress?: (percent: number) => void) : Promise<boolean> {
    let isSuccess = true;
    const button = document.getElementById(buttonId) as HTMLButtonElement;
    if (!button) {
        isSuccess = false;
        return isSuccess;
    }
    let accumulated = 0;
    let lastPercent = -1;
    const originalText = button.textContent;

    button.disabled = true;
    button.textContent = "Starting...";

    try {
        await download(url, filePath, ({ progress, total }) => {
            accumulated += progress;
            const percent = Math.floor((accumulated / total) * 100);
            if (percent === lastPercent) {
                return; // Only repaint when the whole-number percentage moves.
            }
            lastPercent = percent;
            const button = document.getElementById(buttonId) as HTMLButtonElement;
            if (button) {
                button.textContent = `${percent}%`;
                button.disabled = true;
            }
            if (onProgress) {
                onProgress(percent);
            }
        });
        const finishedButton = document.getElementById(buttonId) as HTMLButtonElement;
        if (finishedButton) {
            finishedButton.textContent = originalText;
            finishedButton.disabled = false;
        }
    } catch (e) {
        isSuccess = false;
        if (button) {
            button.textContent = "Error";
        }
        console.error(e);
    } finally {
        if (button) {
            button.disabled = false;
        } 
        return isSuccess;
    }
}

export const parseVersion = (v: string) => v.split('.').map(p => p.padStart(10, '0')).join('.');
/**
 * The Blender version the UI treats as selected: the explicitly selected one when it is
 * still installed, otherwise the default version, otherwise the first installed one.
 */
export const resolveSelectedBlenderVersion = (
    installedBuilds: IBlenderVersion[],
    selectedBlenderVersionId: string | null
): IBlenderVersion | undefined => {
    return installedBuilds.find((x) => x.id === selectedBlenderVersionId)
        ?? installedBuilds.find((x) => x.is_default === true)
        ?? installedBuilds[0];
}

/**
 * Whether a downloadable build is already installed locally.
 */
export const isDownloadableBuildInstalled = (
    installedBuilds: IBlenderVersion[],
    build: IDownloadableBlenderVersion
): boolean => {
    return installedBuilds.some((x) =>
        x.hash && build.hash
            ? x.hash === build.hash && x.version === build.version
            : x.version === build.version && x.risk_id === build.risk_id
    );
}

/**
 * Formats a unix timestamp (seconds) as a short date, or an empty string when it is unset.
 */
export const formatBuildDate = (fileMtime: number): string => {
    if (!fileMtime) {
        return "";
    }
    return new Date(fileMtime * 1000).toISOString().slice(0, 10);
}

/**
 * Channel word for a build's meta line: the patch id for patch builds, "daily" for builds
 * off the main branch, nothing for release-branch builds.
 */
export const buildChannel = (x: IBlenderVersion): string => {
    if (x.patch && x.patch.length > 0) {
        return x.patch;
    }
    const branch = (x.branch ?? "").toLowerCase();
    if (branch === "main" || branch.startsWith("main")) {
        return "daily";
    }
    return "";
}

export const shortHash = (hash: string | null | undefined): string => (hash ?? "").slice(0, 7);

/**
 * Display label for an installed version, e.g. "5.2.1 Stable" or "4.5.4 LTS".
 */
export const blenderVersionLabel = (x: IBlenderVersion | undefined): string => {
    if (!x) {
        return "";
    }
    const variant = describeBuildVariant(x.release_cycle, x.risk_id, x.series);
    return [x.version ?? "", variant?.label ?? ""].filter((v) => v.length > 0).join(" ");
}

/** Display name of an addon row. */
export const addonLabel = (a: IAddon): string => a.name || a.functional_name || "this addon";

/**
 * The `major.minor` series of an installed build. Blender keeps one user folder per series
 * (addons, extensions, preferences), so every build of a series sees the same addons.
 */
export const blenderSeriesOf = (x: IBlenderVersion): string => {
    const series = (x.series ?? "").trim();
    if (series.length > 0) {
        return series;
    }
    return (x.version ?? "").trim().split(".").slice(0, 2).join(".");
}

/** True when both builds read the same series folder, so an addon of one is already there for the other. */
export const shareAddonFolder = (a: IBlenderVersion, b: IBlenderVersion): boolean => {
    const series = blenderSeriesOf(a);
    return series.length > 0 && series === blenderSeriesOf(b);
}

/** An installed series an addon can be applied to. */
export interface IApplyTarget {
    /** The series the addon lands in, e.g. "4.4". */
    series: string,
    /** The build that places and enables the addon: the series' default version, else its newest. */
    version: IBlenderVersion,
    /** Every installed build of the series, newest first; they all see the addon. */
    versions: IBlenderVersion[],
    /** "Blender 4.4.3 Stable" for a lone build, "Blender 4.4 (4.4.0, 4.4.1, 4.4.3)" for several. */
    label: string,
}

const applyTargetLabel = (series: string, versions: IBlenderVersion[]): string => {
    if (versions.length === 1) {
        return `Blender ${blenderVersionLabel(versions[0])}`;
    }
    // The list says which builds share the folder; a number twice (two 4.5.1 builds) adds nothing.
    const numbers = [...new Set(versions.map((v) => (v.version ?? "").trim()).filter((v) => v.length > 0))];
    return numbers.length === 0 ? `Blender ${series}` : `Blender ${series} (${numbers.join(", ")})`;
}

/** The installed builds grouped by series, in the list's order (newest first). */
export const applyTargetsOf = (installedBuilds: IBlenderVersion[]): IApplyTarget[] => {
    const groups = new Map<string, IBlenderVersion[]>();
    for (const x of installedBuilds) {
        const series = blenderSeriesOf(x);
        if (series.length === 0) {
            continue;
        }
        groups.set(series, [...(groups.get(series) ?? []), x]);
    }
    return [...groups.entries()].map(([series, versions]) => ({
        series,
        version: versions.find((v) => v.is_default) ?? versions[0],
        versions,
        label: applyTargetLabel(series, versions),
    }));
}

/** The series an addon of `source` can be applied to: every installed series but its own. */
export const describeApplyTargets = (installedBuilds: IBlenderVersion[], source: IBlenderVersion | undefined): IApplyTarget[] => {
    const own = source ? blenderSeriesOf(source) : "";
    return applyTargetsOf(installedBuilds).filter((t) => t.series !== own);
}

/** The series group `target` belongs to (a group of one when it is not in the list). */
export const applyTargetOf = (installedBuilds: IBlenderVersion[], target: IBlenderVersion): IApplyTarget => {
    const series = blenderSeriesOf(target);
    return applyTargetsOf(installedBuilds).find((t) => t.series === series)
        ?? { series, version: target, versions: [target], label: applyTargetLabel(series, [target]) };
}

export type BuildVariantKind = "lts" | "stable" | "candidate" | "beta" | "alpha" | "neutral";

export interface IBuildVariant {
    label: string,
    kind: BuildVariantKind,
}

const PLATFORM_WORDS = ["windows", "win", "darwin", "macos", "mac", "linux", "x64", "x86", "arm64"];

/**
 * Describes a build's variant for display. Prefers the release cycle (lts / stable / candidate /
 * beta / alpha) and falls back to the variant parsed from the install directory name. Platform
 * words that sometimes land in that fallback (e.g. "windows") produce no tag.
 */
/** Series Blender maintains as long-term support releases. Mirrors LTS_VERSION_ARR in the backend. */
const LTS_SERIES = ["2.83", "2.93", "3.3", "3.6", "4.2", "4.5"];

export const describeBuildVariant = (releaseCycle: string | null | undefined, riskId: string | null | undefined, series?: string | null): IBuildVariant | null => {
    const raw = (releaseCycle ?? "").trim() || (riskId ?? "").trim();
    if (raw.length === 0) {
        return null;
    }
    const v = raw.toLowerCase();
    if (PLATFORM_WORDS.includes(v)) {
        // Release archives carry only the platform in their name, so this is a release build.
        if (series && series.length > 0) {
            return LTS_SERIES.includes(series) ? { label: "LTS", kind: "lts" } : { label: "Stable", kind: "stable" };
        }
        return null;
    }
    switch (v) {
        case "lts":
            return { label: "LTS", kind: "lts" };
        case "stable":
        case "release":
            return { label: "Stable", kind: "stable" };
        case "candidate":
        case "rc":
            return { label: "Candidate", kind: "candidate" };
        case "beta":
            return { label: "Beta", kind: "beta" };
        case "alpha":
            return { label: "Alpha", kind: "alpha" };
        default:
            return { label: raw.charAt(0).toUpperCase() + raw.slice(1), kind: "neutral" };
    }
}

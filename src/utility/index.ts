import { download } from "@tauri-apps/plugin-upload";
import { IBlenderVersion, IDownloadableBlenderVersion } from "../models";

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

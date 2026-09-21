import { IDownloadableBlenderVersion, ISetupBlenderVersion } from "../models";

/** The download listings a setup's versions are looked up in. */
export interface IBuildLists {
    releases: IDownloadableBlenderVersion[],
    dailies: IDownloadableBlenderVersion[],
    patches: IDownloadableBlenderVersion[],
}

export interface IResolvedVersion {
    /** The build to download, or null when nothing fitting is offered any more. */
    build: IDownloadableBlenderVersion | null,
    /** True when it is the very version that was saved; false for a newer build of the same branch. */
    exact: boolean,
    /** One line for the row: where it comes from, or why there is nothing. */
    note: string,
}

const PRE_RELEASE_CHANNELS = ["alpha", "beta", "candidate"];

/** Blender names one branch several ways: `blender-v5.2-release`, `v52`, `main-PR163154`. */
export const branchKey = (branch: string | null | undefined): string =>
    (branch ?? "")
        .toLowerCase()
        .replace(/^blender-/, "")
        .replace(/-release$/, "")
        .replace(/^main-/, "")
        .replace(/\./g, "");

const isPatchBranch = (branch: string | null | undefined): boolean => /^(main-)?pr\d+$/i.test(branch ?? "");

/** Whether the daily and patch listings are needed for this version, or the release listing is enough. */
export const needsBuilderListings = (wanted: ISetupBlenderVersion): boolean =>
    PRE_RELEASE_CHANNELS.includes(wanted.channel.toLowerCase()) || isPatchBranch(wanted.branch);

const newest = (builds: IDownloadableBlenderVersion[]): IDownloadableBlenderVersion | null =>
    builds.length === 0 ? null : builds.reduce((best, b) => (b.file_mtime > best.file_mtime ? b : best));

const sizeMb = (build: IDownloadableBlenderVersion): string => `${Math.round(build.file_size / 1024 / 1024)} MB`;

/**
 * Finds the build to install for a saved version. Releases are matched exactly. Daily and
 * patch builds expire upstream, so those fall back to the newest build of the same branch,
 * then of the same series, and the note says so.
 */
export const resolveDownloadableBuild = (wanted: ISetupBlenderVersion, lists: IBuildLists): IResolvedVersion => {
    const release = lists.releases.find((b) => b.version === wanted.version);
    if (release) {
        return { build: release, exact: true, note: `Release download · ${sizeMb(release)}` };
    }
    const wantedBranch = branchKey(wanted.branch);
    if (isPatchBranch(wanted.branch)) {
        const patch = newest(lists.patches.filter((b) => branchKey(b.patch ?? b.branch) === wantedBranch));
        if (patch) {
            const exact = patch.version === wanted.version;
            return { build: patch, exact, note: `Patch build ${patch.patch ?? patch.branch} · ${sizeMb(patch)}${exact ? "" : ` · newer than the saved ${wanted.version}`}` };
        }
        return { build: null, exact: false, note: `Patch build ${wanted.branch} is no longer offered` };
    }
    const sameVersion = newest(lists.dailies.filter((b) => b.version === wanted.version));
    if (sameVersion) {
        return { build: sameVersion, exact: true, note: `Daily build · ${sizeMb(sameVersion)}` };
    }
    const sameBranch = wantedBranch.length > 0 ? newest(lists.dailies.filter((b) => branchKey(b.branch) === wantedBranch)) : null;
    if (sameBranch) {
        return { build: sameBranch, exact: false, note: `Newest daily build of ${wanted.branch} (${sameBranch.version}) · the saved ${wanted.version} has expired · ${sizeMb(sameBranch)}` };
    }
    const sameSeries = newest(lists.dailies.filter((b) => b.version.startsWith(`${wanted.series}.`)));
    if (sameSeries) {
        return { build: sameSeries, exact: false, note: `Newest ${wanted.series} daily build (${sameSeries.version}) · the saved ${wanted.version} has expired · ${sizeMb(sameSeries)}` };
    }
    // The release listing is scraped with a limit on how far back it goes, so an old version can be
    // absent here and still exist on the server.
    return { build: null, exact: false, note: `Blender ${wanted.version} is not in the download listings; install it by hand from blender.org` };
};

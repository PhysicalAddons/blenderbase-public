import { invoke } from "@tauri-apps/api/core";
import { IAchievement, IActivityImportReport, IActivitySummary, IBlenderVersionTime } from "../models";

/**
 * Stats: time and events in Blender, counted by the startup script Blenderbase places in each
 * Blender series and imported from its logs, and the achievements read off those numbers.
 * Every method rejects when the backend command fails; callers decide how to report it.
 */
export class ActivityService {
    /** Reads whatever is new in the log folder; cheap when nothing is. */
    public async importActivity(): Promise<IActivityImportReport> {
        return await invoke<IActivityImportReport>("cmd_import_activity");
    }

    /**
     * Time per installed version, most used first, over the sessions started at or after
     * `since` (UTC ISO 8601; null for all time). A null version id is time of versions no
     * longer installed.
     */
    public async fetchBlenderVersionTime(since: string | null): Promise<IBlenderVersionTime[]> {
        return await invoke<IBlenderVersionTime[]>("cmd_fetch_blender_version_time", { since });
    }

    /** The figures over the range, plus today and this week from the given local boundaries. */
    public async fetchActivitySummary(since: string | null, todaySince: string, weekSince: string): Promise<IActivitySummary> {
        return await invoke<IActivitySummary>("cmd_fetch_activity_summary", { since, todaySince, weekSince });
    }

    /** The whole catalogue with progress and unlock dates, in catalogue order. */
    public async fetchAchievements(): Promise<IAchievement[]> {
        return await invoke<IAchievement[]>("cmd_fetch_achievements");
    }

    /** The Stats view was opened: the badge goes. */
    public async markAchievementsSeen(): Promise<void> {
        await invoke<void>("cmd_mark_achievements_seen");
    }
}

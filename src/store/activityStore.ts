import { create } from "zustand";
import { IAchievement, IActivityImportReport, IActivitySummary, IBlenderVersionTime } from "../models";
import { ActivityService } from "../services/activityService";
import { postStatus } from "./statusStore";

const activityService = new ActivityService();

/** Two imports closer together than this are one: focus and start fire within the same second. */
const IMPORT_COOLDOWN_MS = 10_000;

/** The import that is running right now, so a second caller joins it instead of starting another. */
let inFlight: Promise<void> | null = null;

/** The range the Stats view looks at. */
export type ActivityRange = 'today' | 'week' | 'month' | 'all';

/** A moment in the shape the session rows carry (UTC, RFC 3339, whole seconds). */
const toSessionTime = (d: Date): string => d.toISOString().replace(/\.\d{3}Z$/, "+00:00");

/** Local midnight today. */
export const startOfToday = (now: Date = new Date()): string =>
    toSessionTime(new Date(now.getFullYear(), now.getMonth(), now.getDate()));

/** Local Monday, midnight, of this week. */
export const startOfWeek = (now: Date = new Date()): string => {
    const d = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    d.setDate(d.getDate() - ((d.getDay() + 6) % 7));
    return toSessionTime(d);
};

/** `since` for a range; null for all time. */
export const sinceOf = (range: ActivityRange, now: Date = new Date()): string | null => {
    switch (range) {
        case 'today':
            return startOfToday(now);
        case 'week':
            return startOfWeek(now);
        case 'month':
            return toSessionTime(new Date(now.getTime() - 30 * 86_400_000));
        default:
            return null;
    }
};

interface ActivityState {
    /** All-time time per version, keyed by the version id, for the Blender rows. Versions no longer installed are not in here. */
    timeByVersion: Record<string, IBlenderVersionTime>,
    /** Every session's open time, matched or not. */
    totalOpenSeconds: number,
    isCounting: boolean,
    lastReport: IActivityImportReport | null,
    lastImportAt: number,
    /** Unlocks nobody has looked at yet: the title-bar badge. */
    unseenUnlocks: number,
    /** The Stats view: its range and what it shows. */
    range: ActivityRange,
    timeRows: IBlenderVersionTime[],
    summary: IActivitySummary | null,
    achievements: IAchievement[],
    isStatsLoading: boolean,
    /** Reads the all-time hours for the Blender rows. */
    refresh: () => Promise<void>,
    /** Imports new logs, then refreshes; a run within the cooldown of the last one is skipped. */
    importAndRefresh: (force?: boolean) => Promise<void>,
    setRange: (range: ActivityRange) => void,
    /** Everything the Stats view shows, for the current range. */
    fetchStats: () => Promise<void>,
    markAchievementsSeen: () => Promise<void>,
}

export const useActivityStore = create<ActivityState>((set, get) => ({
    timeByVersion: {},
    totalOpenSeconds: 0,
    isCounting: false,
    lastReport: null,
    lastImportAt: 0,
    unseenUnlocks: 0,
    range: 'all',
    timeRows: [],
    summary: null,
    achievements: [],
    isStatsLoading: false,
    async refresh() {
        const rows = await activityService.fetchBlenderVersionTime(null);
        const timeByVersion: Record<string, IBlenderVersionTime> = {};
        let totalOpenSeconds = 0;
        for (const row of rows) {
            totalOpenSeconds += row.open_seconds;
            if (row.blender_version_id) {
                timeByVersion[row.blender_version_id] = row;
            }
        }
        set({ timeByVersion, totalOpenSeconds });
    },
    async importAndRefresh(force = false) {
        if (inFlight) {
            return inFlight;
        }
        const now = Date.now();
        if (!force && now - get().lastImportAt < IMPORT_COOLDOWN_MS) {
            return;
        }
        set({ lastImportAt: now });
        inFlight = (async () => {
            try {
                const report = await activityService.importActivity();
                set({ lastReport: report, isCounting: report.is_counting, unseenUnlocks: report.unseen_unlocks });
                if (report.unlocked.length > 0) {
                    postStatus(unlockMessage(report.unlocked));
                }
                if (report.sessions_updated > 0 || Object.keys(get().timeByVersion).length === 0) {
                    await get().refresh();
                }
            } finally {
                inFlight = null;
            }
        })();
        return inFlight;
    },
    setRange(range) {
        set({ range });
    },
    async fetchStats() {
        const since = sinceOf(get().range);
        set({ isStatsLoading: true });
        try {
            const [timeRows, summary, achievements] = await Promise.all([
                activityService.fetchBlenderVersionTime(since),
                activityService.fetchActivitySummary(since, startOfToday(), startOfWeek()),
                activityService.fetchAchievements(),
            ]);
            set({ timeRows, summary, achievements, unseenUnlocks: summary.unseen_unlocks });
        } finally {
            set({ isStatsLoading: false });
        }
    },
    async markAchievementsSeen() {
        if (get().unseenUnlocks === 0 && get().achievements.every((a) => a.unlocked_at === null || a.is_seen)) {
            return;
        }
        await activityService.markAchievementsSeen();
        set((state) => ({
            unseenUnlocks: 0,
            achievements: state.achievements.map((a) => (a.unlocked_at ? { ...a, is_seen: true } : a)),
        }));
    },
}));

/** The status-line notice for what an import unlocked. */
export const unlockMessage = (unlocked: { name: string, description: string }[]): string => {
    if (unlocked.length === 1) {
        return `Achievement unlocked: ${unlocked[0].name} · ${unlocked[0].description}`;
    }
    return `Achievements unlocked: ${unlocked.map((a) => a.name).join(", ")}`;
};

/**
 * Open time as the Blender rows show it: whole hours, "<1 h" for anything shorter, nothing for
 * zero. Rounded to the nearest hour, so 90 minutes reads "2 h".
 */
export const formatBlenderHours = (seconds: number): string => {
    if (!seconds || seconds <= 0) {
        return "";
    }
    const hours = Math.round(seconds / 3600);
    return hours === 0 ? "<1 h" : `${hours} h`;
};

/**
 * Open time as the Stats view shows it: "128 h" from ten hours up, "2 h 5 min" under that,
 * "35 min" under an hour, "0 min" for nothing.
 */
export const formatDuration = (seconds: number): string => {
    if (!seconds || seconds <= 0) {
        return "0 min";
    }
    let hours = Math.floor(seconds / 3600);
    let minutes = Math.round((seconds % 3600) / 60);
    if (minutes === 60) {
        hours += 1;
        minutes = 0;
    }
    if (hours >= 10) {
        return `${Math.round(seconds / 3600)} h`;
    }
    if (hours >= 1) {
        return minutes > 0 ? `${hours} h ${minutes} min` : `${hours} h`;
    }
    return `${Math.max(1, minutes)} min`;
};

/** "2026-10-06" in local time from a UTC ISO 8601 string; empty when it does not parse. */
export const localDate = (iso: string | null | undefined): string => {
    if (!iso) {
        return "";
    }
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) {
        return "";
    }
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
};

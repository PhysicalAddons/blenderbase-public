import { useEffect, useMemo } from 'react';
import { Button, Dropdown, InlineLoading } from '@carbon/react';
import { Apps, ArrowLeft, Asleep, Camera, Catalog, ColorPalette, Cube, Download, Earth, FaceSatisfied, Favorite, Fire, Flag, Hourglass, Layers, Move, Plug, Renew, Rocket, Rotate, Save, Security, Share, Shuffle, Star, Time, Tools, Trophy, Undo, Video, ZoomIn } from '@carbon/react/icons';
import { useShallow } from 'zustand/react/shallow';
import { IAchievement } from '../../models';
import { STATS_DOCUMENTATION_URL } from '../../constants';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { ActivityRange, formatDuration, localDate, useActivityStore } from '../../store/activityStore';
import { StatsSection, useUiControlsStore } from '../../store/uiControlsStore';
import { postStatusError } from '../../store/statusStore';
import { describeBuildVariant, IBuildVariant } from '../../utility';
import DocumentationLink from '../DocumentationLink';

const SECTIONS: { id: StatsSection, label: string }[] = [
	{ id: 'overview', label: 'Overview' },
	{ id: 'achievements', label: 'Achievements' },
];

type RangeOption = { id: ActivityRange, label: string };

const RANGE_OPTIONS: RangeOption[] = [
	{ id: 'all', label: 'All time' },
	{ id: 'month', label: 'Last 30 days' },
	{ id: 'week', label: 'This week' },
	{ id: 'today', label: 'Today' },
];

/** The catalogue names an icon; the look is chosen here, so no UI detail lives in the catalogue. */
const ICONS: Record<string, React.ElementType> = {
	cube: Cube,
	clock: Time,
	moon: Asleep,
	flag: Flag,
	shuffle: Shuffle,
	heart: Favorite,
	shield: Security,
	camera: Camera,
	save: Save,
	undo: Undo,
	monkey: FaceSatisfied,
	rocket: Rocket,
	fire: Fire,
	star: Star,
	hourglass: Hourglass,
	rotate: Rotate,
	earth: Earth,
	zoom: ZoomIn,
	video: Video,
	move: Move,
	plug: Plug,
	tools: Tools,
	catalog: Catalog,
	apps: Apps,
	share: Share,
	sync: Renew,
	download: Download,
	layers: Layers,
	palette: ColorPalette,
};

const plural = (count: number, one: string, many: string): string => `${count.toLocaleString()} ${count === 1 ? one : many}`;

const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e)).replace(/^cmd_\w+: /, "");

type TimeRow = {
	key: string,
	label: string,
	variant: IBuildVariant | null,
	meta: string,
	openSeconds: number,
	/** Bar length against the longest row, 0 to 100. */
	share: number,
	/** Sessions of versions no longer installed, one quiet row. */
	isGone: boolean,
};

type Figure = { label: string, value: string };

/**
 * Hours in each Blender version and the achievements read off them. Takes the middle column
 * and the tab band like Settings and Sync; the range block at the right of the band narrows
 * the Overview to the last days.
 */
const StatsPanel = () => {
	const { section, setSection, setIsStatsOpen } = useUiControlsStore(
		useShallow((s) => ({ section: s.statsSection, setSection: s.setStatsSection, setIsStatsOpen: s.setIsStatsOpen }))
	)
	const installedBuilds = useBlenderManagerStore((s) => s.installedBuilds)
	const { range, setRange, timeRows, summary, achievements, isCounting, isLoading, fetchStats, markAchievementsSeen, importAndRefresh } = useActivityStore(
		useShallow((s) => ({
			range: s.range,
			setRange: s.setRange,
			timeRows: s.timeRows,
			summary: s.summary,
			achievements: s.achievements,
			isCounting: s.isCounting,
			isLoading: s.isStatsLoading,
			fetchStats: s.fetchStats,
			markAchievementsSeen: s.markAchievementsSeen,
			importAndRefresh: s.importAndRefresh,
		}))
	)

	// Opening the view reads the logs once more, then everything it shows; looking at it
	// clears the badge. A range change refetches alone.
	useEffect(() => {
		importAndRefresh()
			.then(() => fetchStats())
			.then(() => markAchievementsSeen())
			.catch((e) => {
				console.error(e);
				postStatusError(`Loading the stats failed: ${errorText(e)}`);
			});
	}, [range]);

	const rows = useMemo<TimeRow[]>(() => {
		const longest = Math.max(1, ...timeRows.map((r) => r.open_seconds));
		return timeRows
			.filter((r) => r.sessions > 0)
			.map((r) => {
				const build = r.blender_version_id ? installedBuilds.find((b) => b.id === r.blender_version_id) : undefined;
				const lastUsed = localDate(r.last_used);
				return {
					key: r.blender_version_id ?? "gone",
					label: build ? build.version ?? "" : "Versions no longer installed",
					variant: build ? describeBuildVariant(build.release_cycle, build.risk_id, build.series) : null,
					meta: [lastUsed ? `Last used ${lastUsed}` : "", plural(r.sessions, "session", "sessions")].filter(Boolean).join(" · "),
					openSeconds: r.open_seconds,
					share: Math.round((r.open_seconds / longest) * 100),
					isGone: !build,
				};
			});
	}, [timeRows, installedBuilds]);

	const unlocked = useMemo(
		() => achievements
			.filter((a) => a.unlocked_at !== null)
			.sort((a, b) => (b.unlocked_at ?? "").localeCompare(a.unlocked_at ?? "")),
		[achievements]
	);
	const locked = useMemo(
		() => achievements
			.filter((a) => a.unlocked_at === null)
			.sort((a, b) => (b.value / b.threshold) - (a.value / a.threshold)),
		[achievements]
	);

	const subtitle = (() => {
		if (section === 'achievements') {
			return `${unlocked.length} of ${achievements.length} achievements unlocked`;
		}
		if (!summary || summary.sessions === 0) {
			return isCounting
				? "Nothing counted yet. Hours appear after the next Blender session"
				: "Turn on Count time and events in Blender under Settings › Launch to start counting";
		}
		return `${formatDuration(summary.open_seconds)} in Blender · ${plural(summary.sessions, "session", "sessions")}`;
	})();

	// Today and this week stand whatever the range says; the rest follows the range.
	const overviewFigures: Figure[] = [
		{ label: "Today", value: formatDuration(summary?.today_open_seconds ?? 0) },
		{ label: "This week", value: formatDuration(summary?.week_open_seconds ?? 0) },
		{ label: "Total", value: formatDuration(summary?.open_seconds ?? 0) },
		{ label: "Active", value: formatDuration(summary?.active_seconds ?? 0) },
		{ label: "Longest session", value: formatDuration(summary?.longest_session_seconds ?? 0) },
		{ label: "Cubes deleted", value: (summary?.counters.cube_deleted ?? 0).toLocaleString() },
	];

	const nextUp = locked[0];
	const latest = unlocked[0];
	const achievementFigures: Figure[] = [
		{ label: "Unlocked", value: `${unlocked.length} of ${achievements.length}` },
		{ label: "Latest", value: latest ? `${latest.name} · ${localDate(latest.unlocked_at)}` : "None yet" },
		{ label: "Next up", value: nextUp ? `${nextUp.name} · ${nextUp.value.toLocaleString()} of ${nextUp.threshold.toLocaleString()}` : "All done" },
	];

	const renderTimeRow = (row: TimeRow) => (
		<div key={row.key} className={`stats_row ${row.isGone ? "stats_row--gone" : ""}`}>
			<div className='stats_row__main'>
				<div className='stats_row__title'>
					<span className='stats_row__version'>{row.label}</span>
					{row.variant && (
						<span className={`blender_tag blender_tag--small blender_tag--${row.variant.kind}`}>{row.variant.label}</span>
					)}
				</div>
				<span className='stats_row__meta'>{row.meta}</span>
			</div>
			<div className='stats_bar' aria-hidden="true">
				<div className='stats_bar__fill' style={{ width: `${row.share}%` }} />
			</div>
			<span className='stats_row__value'>{formatDuration(row.openSeconds)}</span>
		</div>
	);

	const renderAchievementRow = (a: IAchievement) => {
		const Icon = ICONS[a.icon] ?? Trophy;
		const isUnlocked = a.unlocked_at !== null;
		const share = Math.min(100, Math.round((a.value / a.threshold) * 100));
		return (
			<div key={a.id} className={`stats_row stats_row--achievement ${isUnlocked ? "" : "stats_row--locked"}`}>
				<span className={`stats_tile ${isUnlocked ? "stats_tile--unlocked" : ""}`} aria-hidden="true">
					<Icon size={16} />
				</span>
				<div className='stats_row__main'>
					<span className='stats_row__name'>{a.name}</span>
					<span className='stats_row__meta'>{a.description}</span>
				</div>
				{isUnlocked ? (
					<span className='stats_row__unlocked'>Unlocked {localDate(a.unlocked_at)}</span>
				) : (
					<>
						<div className='stats_bar stats_bar--short' aria-hidden="true">
							<div className='stats_bar__fill' style={{ width: `${share}%` }} />
						</div>
						<span className='stats_row__value stats_row__value--muted'>{a.value.toLocaleString()} / {a.threshold.toLocaleString()}</span>
					</>
				)}
			</div>
		);
	};

	const figures = section === 'achievements' ? achievementFigures : overviewFigures;

	return (
		<div className='settings_panel stats_panel'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title column_header__title_row'>
						Stats
						<DocumentationLink href={STATS_DOCUMENTATION_URL} hint="How counting works · opens the documentation in your browser" />
					</span>
					<span className='column_header__subtitle'>{subtitle}</span>
				</div>
				{isLoading && <InlineLoading className="column_header__loading" iconDescription="Loading" />}
				<Button kind="ghost" size="lg" className='column_header__back' title="Back to addons" onClick={() => setIsStatsOpen(false)}>
					<ArrowLeft /> Back to Addons
				</Button>
			</div>
			<div className='column_actions settings_panel__toolbar stats_panel__toolbar'>
				<div className='build_type_switch' role="tablist" aria-label="Stats section">
					{SECTIONS.map((s) => (
						<Button
							key={s.id}
							kind="secondary"
							size="lg"
							role="tab"
							aria-selected={s.id === section}
							className={`build_type_switch__option ${s.id === section ? "build_type_switch__option--selected" : ""}`}
							onClick={() => setSection(s.id)}
						>
							{s.label}
						</Button>
					))}
				</div>
				{section === 'overview' && (
					<Dropdown<RangeOption>
						id="stats-range"
						className='stats_panel__range'
						size="sm"
						hideLabel
						titleText="Range"
						label="Range"
						items={RANGE_OPTIONS}
						itemToString={(item) => item?.label ?? ""}
						selectedItem={RANGE_OPTIONS.find((o) => o.id === range) ?? RANGE_OPTIONS[0]}
						onChange={({ selectedItem }) => {
							if (selectedItem) {
								setRange(selectedItem.id);
							}
						}}
					/>
				)}
			</div>
			<div className='stats_panel__figures'>
				{figures.map((f) => (
					<div key={f.label} className='stats_figure'>
						<span className='stats_figure__label'>{f.label}</span>
						<span className='stats_figure__value' title={f.value}>{f.value}</span>
					</div>
				))}
			</div>
			<div className='list_header settings_panel__list_header stats_panel__list_header'>
				<span>{section === 'achievements' ? "Achievement" : "Time by Blender version"}</span>
				<span className='stats_panel__list_header_right'>{section === 'achievements' ? "Progress" : "Open time"}</span>
			</div>
			<div className='settings_panel__list settings_panel__list--scroll'>
				{section === 'achievements'
					? [...unlocked, ...locked].map(renderAchievementRow)
					: rows.length === 0
						? <div className='settings_panel__empty'>{isCounting ? "No sessions in this range yet." : "Nothing counted yet."}</div>
						: rows.map(renderTimeRow)}
			</div>
		</div>
	)
}

export default StatsPanel

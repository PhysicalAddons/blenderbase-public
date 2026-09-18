import { useRef } from 'react';
import { Button, InlineLoading, Toggle } from '@carbon/react';
import { ArrowLeft, Checkmark, Undo } from '@carbon/react/icons';
import { useShallow } from 'zustand/react/shallow';
import { IBlenderVersion, ISeriesApplyReport, ISetupSeries } from '../../models';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { restorableAddons, useSetupRestoreStore } from '../../store/setupRestoreStore';
import { blenderVersionLabel, parseVersion } from '../../utility';
import { usePagedScroll } from '../../utility/usePagedScroll';

const fileName = (path: string): string => path.split(/[\\/]/).pop() ?? path;

const plural = (count: number, one: string, many: string): string => `${count} ${count === 1 ? one : many}`;

/** One line of what applying a series did, for the row's meta text. */
const describeReport = (report: ISeriesApplyReport): string => {
	if (report.skipped_reason) {
		return `Skipped: ${report.skipped_reason}`;
	}
	const parts: string[] = [];
	if (report.preferences_set > 0 || report.preferences_skipped.length > 0) {
		parts.push(plural(report.preferences_set, "preference", "preferences") + (report.preferences_skipped.length > 0 ? ` (${report.preferences_skipped.length} unknown here)` : ""));
	}
	if (report.theme_applied) {
		parts.push("theme");
	}
	if (report.keymaps_applied.length > 0) {
		parts.push(`keymap (${report.keymaps_applied.join(", ")})`);
	}
	if (report.addons_installed.length > 0) {
		parts.push(plural(report.addons_installed.length, "addon installed", "addons installed"));
	}
	if (report.addons_failed.length > 0) {
		parts.push(plural(report.addons_failed.length, "addon failed", "addons failed"));
	}
	if (report.addons_manual.length > 0) {
		parts.push(`${report.addons_manual.length} to install by hand`);
	}
	const applied = parts.length > 0 ? `Applied with ${report.applied_with}: ${parts.join(" · ")}` : `Nothing to apply with ${report.applied_with}`;
	return report.warnings.length > 0 ? `${applied} · ${plural(report.warnings.length, "warning", "warnings")}` : applied;
};

/**
 * A setup file opened for restoring: one row per Blender series it holds, with a switch for
 * each part that can be applied, and the outcome once it was.
 */
const RestoreSetupPanel = () => {
	const installedBuilds = useBlenderManagerStore((s) => s.installedBuilds)
	const { info, choices, reports, isBusy, close, setChoice, apply, undo } = useSetupRestoreStore(
		useShallow((s) => ({
			info: s.info,
			choices: s.choices,
			reports: s.reports,
			isBusy: s.isBusy,
			close: s.close,
			setChoice: s.setChoice,
			apply: s.apply,
			undo: s.undo,
		}))
	)
	const listRef = useRef<HTMLDivElement>(null)
	usePagedScroll(listRef, { rowSelector: '.restore_row' })

	if (!info) {
		return null;
	}
	const manifest = info.manifest;

	// The newest installed version of a series applies the setup, as when it was saved.
	const targetOf = (series: string): IBlenderVersion | undefined =>
		installedBuilds
			.filter((b) => b.series === series && (b.executable_file_path ?? "").length > 0)
			.sort((a, b) => parseVersion(b.version ?? "0").localeCompare(parseVersion(a.version ?? "0")))[0];

	const rows = Object.entries(manifest.series)
		.sort(([a], [b]) => parseVersion(b).localeCompare(parseVersion(a)));
	const missingVersions = manifest.blender.filter((v) => !installedBuilds.some((b) => b.version === v.version));
	const canApply = choices.some((c) => (c.preferences || c.theme || c.keymap || c.addons) && targetOf(c.series) !== undefined);

	const savedOn = manifest.meta.created ? new Date(manifest.meta.created).toLocaleDateString() : "";
	const subtitle = [fileName(info.file_path), savedOn ? `saved ${savedOn}` : "", manifest.meta.platform]
		.filter((v) => v.length > 0)
		.join(" · ");

	// Core addons only carry an enabled state, so they are not counted as addons to restore.
	const addonSummary = (section: ISetupSeries): string => {
		const listed = section.addons.filter((a) => a.source !== "core").length;
		if (listed === 0) {
			return "none saved";
		}
		const restorable = restorableAddons(section).length;
		return restorable === listed ? `${listed}` : `${restorable} of ${listed}`;
	};

	const renderRow = (series: string, section: ISetupSeries) => {
		const choice = choices.find((c) => c.series === series);
		const report = reports.find((r) => r.series === series);
		const target = targetOf(series);
		const meta = report
			? describeReport(report)
			: target
				? `Applies with Blender ${blenderVersionLabel(target)} · saved from ${section.captured_with}`
				: `Blender ${series} is not installed · saved from ${section.captured_with}`;
		const switchFor = (key: "preferences" | "theme" | "keymap" | "addons", present: boolean, label: string, note?: string) => (
			<div className='restore_row__switch'>
				<Toggle
					id={`restore-${series}-${key}`}
					size="sm"
					hideLabel
					labelA=""
					labelB=""
					labelText={`${label} of Blender ${series}`}
					toggled={Boolean(choice?.[key]) && present && target !== undefined}
					disabled={isBusy || !present || !target || report !== undefined}
					onToggle={(checked: boolean) => setChoice(series, { [key]: checked })}
				/>
				{!present && <span className='restore_row__absent'>not saved</span>}
				{present && note && <span className='restore_row__absent'>{note}</span>}
			</div>
		);
		return (
			<div key={series} className={`restore_row ${target ? "" : "restore_row--unavailable"} ${report?.skipped_reason ? "restore_row--skipped" : ""}`}>
				<div className='restore_row__main'>
					<span className='restore_row__label'>Blender {series}</span>
					<span className='restore_row__meta' title={meta}>{meta}</span>
				</div>
				{switchFor("preferences", Boolean(section.preferences), "Preferences")}
				{switchFor("theme", Boolean(section.theme), section.theme?.name ? `Theme ${section.theme.name}` : "Theme")}
				{switchFor("keymap", Boolean(section.keymap), "Keymap")}
				{switchFor("addons", restorableAddons(section).length > 0, "Addons", addonSummary(section))}
				<div className='restore_row__undo'>
					{report && !report.skipped_reason && (
						<Button
							kind="ghost"
							size="md"
							className='restore_row__undo_button'
							renderIcon={Undo}
							iconDescription="Undo: put the previous configuration back"
							title="Undo: put the previous configuration back"
							hasIconOnly
							disabled={isBusy}
							onClick={() => void undo(series)}
						/>
					)}
				</div>
			</div>
		);
	};

	return (
		<div className='restore_panel'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title'>Restore setup</span>
					<span className='column_header__subtitle' title={info.file_path}>{subtitle}</span>
				</div>
				{isBusy && <InlineLoading className="column_header__loading" iconDescription="Working" />}
			</div>
			<div className='column_actions restore_panel__toolbar'>
				<Button
					kind="secondary"
					size="lg"
					className='install_button'
					title="Back up each series, then write the selected preferences, theme and keymap"
					disabled={isBusy || !canApply}
					onClick={() => void apply()}
				>
					<Checkmark /> Apply setup
				</Button>
				<Button
					kind="secondary"
					size="lg"
					className='install_button'
					title="Leave without changing anything"
					disabled={isBusy}
					onClick={close}
				>
					<ArrowLeft /> Back
				</Button>
			</div>
			<div className='list_header restore_panel__list_header'>
				<span>Series</span>
				<span className='centered'>Preferences</span>
				<span className='centered'>Theme</span>
				<span className='centered'>Keymap</span>
				<span className='centered'>Addons</span>
				<span></span>
			</div>
			<div className='restore_panel__list' ref={listRef}>
				{rows.length === 0 ? (
					<div className='restore_panel__empty'>This setup file holds no Blender configuration.</div>
				) : rows.map(([series, section]) => renderRow(series, section))}
				{missingVersions.length > 0 && (
					<div className='restore_panel__note'>
						Not installed here: Blender {missingVersions.map((v) => v.version).join(", ")}. Install them, then open this file again to apply their series.
					</div>
				)}
			</div>
		</div>
	)
}

export default RestoreSetupPanel

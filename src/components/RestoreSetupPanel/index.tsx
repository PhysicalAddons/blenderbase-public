import { useRef } from 'react';
import { Button, InlineLoading, Modal, Toggle } from '@carbon/react';
import { ArrowLeft, Checkmark, Download, Undo } from '@carbon/react/icons';
import { open } from '@tauri-apps/plugin-dialog';
import { useShallow } from 'zustand/react/shallow';
import { IBlenderVersion, ISeriesApplyReport, ISetupBlenderVersion, ISetupSeries } from '../../models';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { IMissingVersion, missingButtonId, restorableAddons, useSetupRestoreStore } from '../../store/setupRestoreStore';
import { blenderVersionLabel, describeBuildVariant, parseVersion } from '../../utility';
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
	const { info, choices, reports, isBusy, missing, locationPrompt, close, setChoice, apply, undo } = useSetupRestoreStore(
		useShallow((s) => ({
			info: s.info,
			choices: s.choices,
			reports: s.reports,
			isBusy: s.isBusy,
			missing: s.missing,
			locationPrompt: s.locationPrompt,
			close: s.close,
			setChoice: s.setChoice,
			apply: s.apply,
			undo: s.undo,
		}))
	)
	const { installMissingVersion, installAllMissing, retryLookup, setLocationPromptPath, confirmLocationPrompt, cancelLocationPrompt } = useSetupRestoreStore(
		useShallow((s) => ({
			installMissingVersion: s.installMissingVersion,
			installAllMissing: s.installAllMissing,
			retryLookup: s.retryLookup,
			setLocationPromptPath: s.setLocationPromptPath,
			confirmLocationPrompt: s.confirmLocationPrompt,
			cancelLocationPrompt: s.cancelLocationPrompt,
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
	// Rows of versions that arrived meanwhile stay, marked installed, so the list does not jump.
	const missingRows = [...missing].sort((a, b) => parseVersion(b.wanted.version).localeCompare(parseVersion(a.wanted.version)));
	const readyToInstall = missing.filter((m) => m.state === "ready").length;
	const isInstalling = missing.some((m) => m.state === "downloading" || m.state === "installing");
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

	const pickPromptDirectory = async () => {
		if (!locationPrompt) {
			return;
		}
		try {
			const selected = await open({ multiple: false, directory: true, title: "Choose where Blender versions are installed", defaultPath: locationPrompt.path });
			if (typeof selected === "string" && selected.length > 0) {
				setLocationPromptPath(selected);
			}
		} catch (e) {
			console.error(e);
		}
	};

	const missingLabel = (wanted: ISetupBlenderVersion): string => {
		const variant = describeBuildVariant(wanted.channel, null, wanted.series);
		return [wanted.version, variant?.label ?? ""].filter((v) => v.length > 0).join(" ");
	};

	// One version the setup names that this computer lacks: what would be downloaded, and a
	// button whose text shows the download progress.
	const renderMissingRow = (m: IMissingVersion) => {
		const buttonId = missingButtonId(m.id);
		const busy = m.state === "downloading" || m.state === "installing";
		// No answer from the listings yet (offline, or the lookup failed): the button asks again.
		const canRetry = m.state === "unavailable" && m.resolved === null;
		return (
			<div key={m.id} className={`restore_missing ${m.state === "unavailable" || m.state === "failed" ? "restore_missing--unavailable" : ""}`}>
				<div className='restore_missing__main'>
					<span className='restore_missing__label'>Blender {missingLabel(m.wanted)}{m.wanted.default ? " · default" : ""}</span>
					<span className='restore_missing__meta' title={m.message}>{m.message}</span>
				</div>
				<div className='restore_missing__action'>
					{m.state === "installed" ? (
						<span className='restore_missing__installed'><Checkmark /> Installed</span>
					) : (
						<Button
							kind="secondary"
							size="lg"
							className='download_button restore_missing__button'
							title={canRetry ? "Look up the download again" : m.resolved?.build ? `Download and install Blender ${m.resolved.build.version}` : m.message}
							renderIcon={Download}
							disabled={(m.state !== "ready" && !canRetry) || isBusy}
							onClick={() => void (canRetry ? retryLookup() : installMissingVersion(m.id, buttonId))}
						>
							<span id={buttonId} className='progress'>{m.state === "installing" ? "Installing…" : m.state === "looking" ? "Looking up…" : busy ? "…" : canRetry ? "Retry" : m.state === "unavailable" ? "Unavailable" : m.state === "failed" ? "Failed" : "Install"}</span>
						</Button>
					)}
				</div>
			</div>
		);
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
				{(!present || note) && <span className='restore_row__absent'>{note ?? "not saved"}</span>}
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
			<Modal
				open={locationPrompt !== null}
				size="sm"
				modalHeading="Where should Blender versions be installed?"
				modalLabel="First download"
				primaryButtonText="Use this folder"
				secondaryButtonText="Cancel"
				onRequestClose={cancelLocationPrompt}
				onRequestSubmit={() => void confirmLocationPrompt()}
			>
				<p className='location_prompt__text'>
					Every version you download is unpacked into its own folder here. You can change it later in Settings.
				</p>
				<div className='location_prompt__row'>
					<code className='location_prompt__path' title={locationPrompt?.path ?? ""}>{locationPrompt?.path ?? ""}</code>
					<Button kind="tertiary" size="md" onClick={() => void pickPromptDirectory()}>
						Change…
					</Button>
				</div>
			</Modal>
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
				{readyToInstall > 1 && (
					<Button
						kind="secondary"
						size="lg"
						className='install_button'
						title="Download and install every missing Blender version, one after the other"
						disabled={isBusy || isInstalling}
						onClick={() => void installAllMissing()}
					>
						<Download /> Install {readyToInstall} missing versions
					</Button>
				)}
				<Button
					kind="secondary"
					size="lg"
					className='install_button'
					title="Leave without changing anything"
					disabled={isBusy || isInstalling}
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
				{missingRows.length > 0 && (
					<>
						<div className='restore_panel__section'>
							<span className='restore_panel__section_title'>Blender versions to install</span>
							<span className='restore_panel__section_text'>The setup names these versions and this computer does not have them. Once one is installed, its series above can be applied.</span>
						</div>
						{missingRows.map(renderMissingRow)}
					</>
				)}
			</div>
		</div>
	)
}

export default RestoreSetupPanel

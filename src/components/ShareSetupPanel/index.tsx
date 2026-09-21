import { Fragment, useMemo, useState } from 'react';
import { Button, InlineLoading, Toggle } from '@carbon/react';
import { ArrowLeft, Checkmark, ChevronDown, Reset } from '@carbon/react/icons';
import { useShallow } from 'zustand/react/shallow';
import { IAddon, IBlenderVersion } from '../../models';
import { AddonService } from '../../services/addonService';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { describeShare, isEverything, seriesChoice, SharePart, useSetupShareStore } from '../../store/setupShareStore';
import { postStatusError } from '../../store/statusStore';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { blenderVersionLabel } from '../../utility';

const addonService = new AddonService();

/** The key an addon is left out by: its module (a legacy addon's folder) or extension id. */
const addonKey = (addon: IAddon): string => addon.functional_name || addon.name || addon.id;

const kindLabel = (addon: IAddon): string => {
	switch (addon.variant_type) {
		case 'extension':
			return 'Extension';
		case 'core':
			return 'Built in';
		default:
			return 'Addon';
	}
};

const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e)).replace(/^cmd_\w+: /, "");

/**
 * What to share: which installed Blender versions go, and per series which parts and which
 * addons. Laid out like the restore view, so both ends of a transfer look the same. Everything
 * goes until something is unticked; the choice is remembered on this computer.
 */
const ShareSetupPanel = () => {
	const installedBuilds = useBlenderManagerStore((s) => s.installedBuilds)
	const { selection, setIncludeAddonFiles, setVersionIncluded, setSeriesPart, setAddonIncluded, reset } = useSetupShareStore(
		useShallow((s) => ({ selection: s.selection, setIncludeAddonFiles: s.setIncludeAddonFiles, setVersionIncluded: s.setVersionIncluded, setSeriesPart: s.setSeriesPart, setAddonIncluded: s.setAddonIncluded, reset: s.reset }))
	)
	const { setIsShareSetupOpen, setIsSyncOpen } = useUiControlsStore(
		useShallow((s) => ({ setIsShareSetupOpen: s.setIsShareSetupOpen, setIsSyncOpen: s.setIsSyncOpen }))
	)
	/** The series whose addons are unfolded under its row. */
	const [expanded, setExpanded] = useState<string | null>(null)
	const [addonsBySeries, setAddonsBySeries] = useState<Record<string, IAddon[]>>({})
	const [loadingSeries, setLoadingSeries] = useState<string | null>(null)

	// Installed versions grouped by series, in the order of the left column (newest first).
	const groups = useMemo(() => {
		const map = new Map<string, IBlenderVersion[]>();
		installedBuilds.forEach((v) => {
			if (!v.series) {
				return;
			}
			map.set(v.series, [...(map.get(v.series) ?? []), v]);
		});
		return [...map.entries()];
	}, [installedBuilds]);

	// Back to the Sync view, on the tab it was on.
	const close = () => {
		setIsShareSetupOpen(false);
		setIsSyncOpen(true);
	};

	const loadAddons = async (series: string, versions: IBlenderVersion[]) => {
		if (addonsBySeries[series] || versions.length === 0) {
			return;
		}
		// The newest version of the series speaks for it, as when the setup is read.
		setLoadingSeries(series);
		try {
			const list = await addonService.fetchAddons(versions[0].id);
			list.sort((a, b) => (a.name ?? addonKey(a)).localeCompare(b.name ?? addonKey(b)));
			setAddonsBySeries((m) => ({ ...m, [series]: list }));
		} catch (e) {
			console.error(e);
			postStatusError(`Reading the addons of Blender ${series} failed: ${errorText(e)}`);
		} finally {
			setLoadingSeries(null);
		}
	};

	const toggleExpanded = (series: string, versions: IBlenderVersion[]) => {
		if (expanded === series) {
			setExpanded(null);
			return;
		}
		setExpanded(series);
		void loadAddons(series, versions);
	};

	const renderAddons = (series: string) => {
		const addons = addonsBySeries[series];
		if (loadingSeries === series || !addons) {
			return <div className='share_addon--empty'><InlineLoading iconDescription="Reading" description="Reading the addons…" /></div>;
		}
		if (addons.length === 0) {
			return <div className='share_addon--empty'>Blender {series} has no addons</div>;
		}
		const excluded = seriesChoice(selection, series).excluded_addons;
		// The same switch the Addons panel uses for enabling, here for going or staying.
		return addons.map((a) => {
			const name = a.name || addonKey(a);
			return (
				<div key={a.id} className='share_addon'>
					<div className='share_addon__main'>
						<span className='share_addon__label'>{name}</span>
						<span className='share_addon__meta'>{kindLabel(a)}{a.version ? ` · ${a.version}` : ""}{a.is_enabled ? "" : " · disabled in Blender"}</span>
					</div>
					<div className='share_addon__switch'>
						<Toggle
							id={`share-addon-${series}-${a.id}`}
							size="sm"
							hideLabel
							labelA=""
							labelB=""
							labelText={`${name} of Blender ${series}`}
							toggled={!excluded.includes(addonKey(a))}
							onToggle={(checked: boolean) => setAddonIncluded(series, addonKey(a), checked)}
						/>
					</div>
				</div>
			);
		});
	};

	const renderRow = ([series, versions]: [string, IBlenderVersion[]]) => {
		const choice = seriesChoice(selection, series);
		const anyVersion = versions.some((v) => !selection.excluded_version_ids.includes(v.id));
		const addons = addonsBySeries[series];
		const excluded = choice.excluded_addons;
		const countLabel = addons
			? `${addons.filter((a) => !excluded.includes(addonKey(a))).length} of ${addons.length}`
			: excluded.length > 0 ? `${excluded.length} left out` : "all";
		const switchFor = (part: SharePart, label: string) => (
			<div className='share_row__switch'>
				<Toggle
					id={`share-${series}-${part}`}
					size="sm"
					hideLabel
					labelA=""
					labelB=""
					labelText={`${label} of Blender ${series}`}
					toggled={choice[part]}
					disabled={!anyVersion}
					onToggle={(checked: boolean) => setSeriesPart(series, part, checked)}
				/>
				{part === 'addons' && (
					<button
						type="button"
						className='share_row__count'
						disabled={!anyVersion || !choice.addons}
						aria-expanded={expanded === series}
						aria-label={`Choose the addons of Blender ${series}`}
						title={expanded === series ? "Hide the addons" : "Choose which addons go"}
						onClick={() => toggleExpanded(series, versions)}
					>
						<span>{countLabel}</span>
						{/* One chevron, rotated like the Recent Files groups: right when closed, down when open. */}
						<span className={`share_row__chevron ${expanded === series ? "" : "share_row__chevron--collapsed"}`}>
							<ChevronDown />
						</span>
					</button>
				)}
			</div>
		);
		return (
			<Fragment key={series}>
				<div className={`share_row ${anyVersion ? "" : "share_row--inactive"}`}>
					<div className='share_row__main'>
						<span className='share_row__label'>Blender {series}</span>
						<div className='share_row__versions'>
							{versions.map((v) => {
								const included = !selection.excluded_version_ids.includes(v.id);
								const label = blenderVersionLabel(v);
								return (
									<button
										key={v.id}
										type="button"
										className='share_row__version'
										aria-pressed={included}
										title={included ? `${label} goes · click to leave it out` : `${label} stays out · click to include it`}
										onClick={() => setVersionIncluded(v.id, !included)}
									>
										{included && <Checkmark size={12} />}
										{label}
									</button>
								);
							})}
						</div>
					</div>
					{switchFor('preferences', 'Preferences')}
					{switchFor('theme', 'Theme')}
					{switchFor('keymap', 'Keymap')}
					{switchFor('addons', 'Addons')}
				</div>
				{expanded === series && renderAddons(series)}
			</Fragment>
		);
	};

	return (
		<div className='share_panel'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title'>What to share</span>
					<span className='column_header__subtitle'>{describeShare(selection, installedBuilds)}</span>
				</div>
			</div>
			<div className='column_actions share_panel__toolbar'>
				<Button kind="secondary" size="lg" className='install_button' title="Back to Sync" onClick={close}>
					<ArrowLeft /> Done
				</Button>
				<Button kind="secondary" size="lg" className='install_button' title="Tick everything again" disabled={isEverything(selection)} onClick={reset}>
					<Reset /> Everything
				</Button>
			</div>
			<div className='list_header share_panel__list_header'>
				<span>Blender versions</span>
				<span className='centered'>Preferences</span>
				<span className='centered'>Theme</span>
				<span className='centered'>Keymap</span>
				<span className='centered'>Addons</span>
			</div>
			<div className='share_panel__list'>
				{groups.length === 0 ? (
					<div className='share_panel__empty'>No installed Blender version to share yet.</div>
				) : groups.map(renderRow)}
				<div className='share_panel__option'>
					<div className='share_panel__option_main'>
						<span className='share_panel__option_label'>Include addon files</span>
						<span className='share_panel__option_text'>Packs addons installed from a file, so they restore without the download; the setup grows by their size</span>
					</div>
					<Toggle
						id="share-addon-files"
						size="sm"
						hideLabel
						labelA=""
						labelB=""
						labelText="Include addon files"
						toggled={selection.include_addon_files}
						onToggle={(checked: boolean) => setIncludeAddonFiles(checked)}
					/>
				</div>
				<div className='share_panel__note'>
					Untick a version and it stays out of the setup; a series with no version ticked is not read at all, which makes sharing quicker. The other computer still chooses what to apply.
				</div>
			</div>
		</div>
	)
}

export default ShareSetupPanel

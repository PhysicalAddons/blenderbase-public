import { useEffect, useMemo, useRef, useState } from 'react';
import { Button, Dropdown, InlineLoading, Search, Toggle } from '@carbon/react';
import { Add, Link, Renew, TrashCan } from '@carbon/react/icons';
import { ask, open } from '@tauri-apps/plugin-dialog';
import { IAddon } from '../../models';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { useAddonStore } from '../../store/addonStore';
import { resolveSelectedBlenderVersion } from '../../utility';
import { usePagedScroll } from '../../utility/usePagedScroll';

interface ITypeFilter {
	id: string,
	text: string,
}

const TYPE_FILTERS: ITypeFilter[] = [
	{ id: 'all', text: 'All types' },
	{ id: 'extension', text: 'Extensions' },
	{ id: 'addon', text: 'Addons' },
	{ id: 'core', text: 'Core' },
];

const kindLabel = (a: IAddon): string => {
	switch (a.variant_type) {
		case 'extension':
			return 'Extension';
		case 'core':
			return 'Core';
		default:
			return 'Addon';
	}
}

const AddonPanel = () => {
	const { installedBuilds } = useBlenderManagerStore()
	const { selectedBlenderVersionId } = useUiControlsStore()
	const { addons, isBusy, loadedForBlenderVersionId, loadAddons, refreshAddons, toggleAddon, installAddon, symlinkAddon, deleteAddon, clear } = useAddonStore()
	const [searchText, setSearchText] = useState<string>("")
	const [typeFilter, setTypeFilter] = useState<ITypeFilter>(TYPE_FILTERS[0])
	const listRef = useRef<HTMLDivElement>(null)
	usePagedScroll(listRef)

	const selectedVersion = resolveSelectedBlenderVersion(installedBuilds, selectedBlenderVersionId);
	const selectedId = selectedVersion?.id ?? null;

	useEffect(() => {
		if (selectedId === null) {
			clear();
			return;
		}
		loadAddons(selectedId);
	}, [selectedId]);

	const visibleAddons = useMemo(() => {
		const q = searchText.trim().toLowerCase();
		return addons.filter((a) => {
			if (typeFilter.id !== 'all' && (a.variant_type ?? 'addon') !== typeFilter.id) {
				return false;
			}
			if (q.length === 0) {
				return true;
			}
			return [a.name, a.functional_name, a.author, a.category, a.description]
				.some((v) => (v ?? "").toLowerCase().includes(q));
		});
	}, [addons, searchText, typeFilter]);

	const isCurrent = loadedForBlenderVersionId === selectedId;

	const pickAndInstallAddon = async () => {
		if (selectedId === null) {
			return;
		}
		try {
			const selected = await open({
				multiple: false,
				directory: false,
				title: "Select an addon file (.zip or .py)",
				filters: [{ name: "Blender addon", extensions: ["zip", "py"] }],
			});
			if (typeof selected === "string" && selected.length > 0) {
				await installAddon(selectedId, selected);
			}
		} catch (e) {
			console.error(e);
		}
	}

	const pickAndSymlinkAddon = async () => {
		if (selectedId === null) {
			return;
		}
		try {
			const selected = await open({
				multiple: false,
				directory: true,
				title: "Select an addon directory to symlink",
			});
			if (typeof selected === "string" && selected.length > 0) {
				await symlinkAddon(selectedId, selected);
			}
		} catch (e) {
			console.error(e);
		}
	}

	const confirmAndDelete = async (a: IAddon) => {
		if (selectedId === null) {
			return;
		}
		const label = a.name || a.functional_name || "this addon";
		const confirmed = await ask(
			a.is_symbolic_link
				? `Remove the symlink for ${label}? The linked directory itself is kept.`
				: `Remove ${label} from Blender ${selectedVersion?.version ?? ""}? Its files are deleted.`,
			{ title: "Remove addon", kind: "warning", okLabel: "Remove", cancelLabel: "Cancel" }
		);
		if (confirmed) {
			await deleteAddon(selectedId, a.id);
		}
	}

	const subtitle = selectedVersion
		? (isCurrent && !isBusy ? `${addons.length} addons` : " ")
		: "Select a Blender version to see its addons";

	return (
		<div className='addon_panel'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title'>Addons</span>
					<span className='column_header__subtitle'>{subtitle}</span>
				</div>
				{isBusy ? (
					<InlineLoading className="column_header__loading" iconDescription="Working" />
				) : (
					<Button
						kind="ghost"
						renderIcon={Renew}
						iconDescription="Re-read addons from Blender"
						title="Re-read addons from Blender"
						hasIconOnly
						disabled={selectedId === null}
						onClick={() => selectedId !== null && refreshAddons(selectedId)}
					/>
				)}
			</div>
			<div className='column_actions addon_panel__toolbar'>
				<Button
					kind="secondary"
					size="lg"
					className='install_button'
					title="Install an addon from a .zip or .py file"
					disabled={isBusy || selectedId === null}
					onClick={pickAndInstallAddon}
				>
					<Add /> Install Addon
				</Button>
				<Button
					kind="secondary"
					size="lg"
					className='install_button'
					title="Symlink an addon directory"
					disabled={isBusy || selectedId === null}
					onClick={pickAndSymlinkAddon}
				>
					<Link /> Symlink Addon
				</Button>
				<Search
					size="lg"
					labelText="Search addons"
					placeholder="Search addons"
					value={searchText}
					onChange={(e: any) => setSearchText(e.target.value ?? "")}
				/>
				<Dropdown
					size="lg"
					id="addon-type-filter"
					label="All types"
					titleText=""
					items={TYPE_FILTERS}
					itemToString={(item: ITypeFilter | null) => item ? item.text : ''}
					selectedItem={typeFilter}
					onChange={(e: any) => setTypeFilter(e.selectedItem ?? TYPE_FILTERS[0])}
				/>
			</div>
			<div className='list_header addon_panel__list_header'>
				<span>Name</span>
				<span>Version</span>
				<span>Type</span>
				<span>Enabled</span>
				<span></span>
			</div>
			<div className='addon_panel__list' ref={listRef}>
				{selectedId === null ? (
					<div className='addon_panel__empty'>Install or select a Blender version first.</div>
				) : !isCurrent ? (
					<div className='addon_panel__empty'>Loading…</div>
				) : visibleAddons.length === 0 ? (
					<div className='addon_panel__empty'>
						{addons.length === 0
							? (isBusy ? "Reading addons from Blender…" : "No addons found for this Blender version.")
							: "No addons match the current filter."}
					</div>
				) : visibleAddons.map((a) => (
					<div key={a.id} className='addon_row' title={a.description ?? a.name ?? ""}>
						<div className='addon_row__name'>
							<span className='addon_row__label'>{a.name || a.functional_name}</span>
							{a.is_symbolic_link && (
								<span className='addon_row__link' title={`Symlinked from ${a.installation_directory}`}><Link /></span>
							)}
						</div>
						<span className='addon_row__version'>{a.version ?? ""}</span>
						<span className='addon_row__type'>{kindLabel(a)}</span>
						<div className='addon_row__toggle'>
							<Toggle
								id={`addon-enabled-${a.id}`}
								size="sm"
								hideLabel
								labelA=""
								labelB=""
								labelText={`Enable ${a.name ?? ""}`}
								toggled={a.is_enabled}
								disabled={isBusy}
								onToggle={(checked: boolean) => toggleAddon(a.id, checked)}
							/>
						</div>
						<div className='addon_row__menu'>
							{a.variant_type !== 'core' && (
								<Button
									kind="ghost"
									size="md"
									className='addon_row__delete'
									renderIcon={TrashCan}
									iconDescription={a.is_symbolic_link ? "Remove symlink" : "Remove addon"}
									title={a.is_symbolic_link ? "Remove symlink" : "Remove addon"}
									hasIconOnly
									disabled={isBusy}
									onClick={() => confirmAndDelete(a)}
								/>
							)}
						</div>
					</div>
				))}
			</div>
		</div>
	)
}

export default AddonPanel

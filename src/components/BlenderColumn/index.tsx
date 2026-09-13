import { useRef, useState } from 'react';
import { Button, InlineLoading } from '@carbon/react';
import { Add, Renew, Star, StarFilled, TrashCan } from '@carbon/react/icons';
import { useShallow } from 'zustand/react/shallow';
import { IBlenderVersion } from '../../models';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { BlenderService } from '../../services/blenderService';
import { blenderVersionLabel, buildChannel, describeBuildVariant, formatBuildDate, resolveSelectedBlenderVersion, shortHash } from '../../utility';
import { usePagedScroll } from '../../utility/usePagedScroll';
import { showContextMenu } from '../../utility/contextMenu';
import { postStatus, postStatusError } from '../../store/statusStore';

const blenderService = new BlenderService();

const revealBlenderVersion = async (id: string): Promise<void> => {
	try {
		await blenderService.revealBlenderVersion(id);
	} catch (e) {
		console.error(e);
		postStatusError(`Could not open the installation folder: ${e}`);
	}
};

const BlenderColumn = () => {
	const { installedBuilds, setInstalledBuilds, refreshInstalledBuilds } = useBlenderManagerStore(
		useShallow((s) => ({
			installedBuilds: s.installedBuilds,
			setInstalledBuilds: s.setInstalledBuilds,
			refreshInstalledBuilds: s.refreshInstalledBuilds,
		}))
	)
	const {
		isInstallBlenderOpen,
		selectedBlenderVersionId,
		newlyInstalledBlenderIds,
		setIsInstallBlenderOpen,
		setSelectedBlenderVersionId,
	} = useUiControlsStore(
		useShallow((s) => ({
			isInstallBlenderOpen: s.isInstallBlenderOpen,
			selectedBlenderVersionId: s.selectedBlenderVersionId,
			newlyInstalledBlenderIds: s.newlyInstalledBlenderIds,
			setIsInstallBlenderOpen: s.setIsInstallBlenderOpen,
			setSelectedBlenderVersionId: s.setSelectedBlenderVersionId,
		}))
	)

	const listRef = useRef<HTMLDivElement>(null)
	usePagedScroll(listRef, { rowSelector: '.blender_row' })
	const [isRefreshing, setIsRefreshing] = useState<boolean>(false)

	/** Rescans the installation locations on disk and reloads the list. */
	const refreshInstalled = async () => {
		setIsRefreshing(true);
		const startedAt = Date.now();
		postStatus("Refreshing installed Blender versions…", true);
		try {
			await refreshInstalledBuilds();
			postStatus(`${useBlenderManagerStore.getState().installedBuilds.length} Blender versions installed`);
		} catch (e) {
			console.error(e);
			postStatusError(`Refreshing installed versions failed: ${e}`);
		} finally {
			// Keep the spinner visible long enough to register as feedback.
			const remaining = 500 - (Date.now() - startedAt);
			setTimeout(() => setIsRefreshing(false), Math.max(0, remaining));
		}
	}

	/** Reloads the list after a change; reports a failure without hiding the caller's own status. */
	const reloadInstalled = async () => {
		try {
			await setInstalledBuilds();
		} catch (e) {
			console.error(e);
			postStatusError(`Loading installed versions failed: ${e}`);
		}
	}

	const selectedVersion = resolveSelectedBlenderVersion(installedBuilds, selectedBlenderVersionId);

	const selectVersion = (id: string) => {
		setSelectedBlenderVersionId(id);
	}

	const setAsDefault = async (id: string) => {
		const target = installedBuilds.find((x) => x.id === id);
		try {
			await blenderService.setBlenderVersionAsDefault(id)
			await reloadInstalled();
			postStatus(`Blender ${blenderVersionLabel(target)} is now the default`);
		} catch (e) {
			console.error(e);
			postStatusError(`Could not set the default version: ${e}`);
		}
	}

	const deleteVersion = async (id: string) => {
		const target = installedBuilds.find((x) => x.id === id);
		postStatus(`Deleting Blender ${blenderVersionLabel(target)}…`, true);
		try {
			await blenderService.deleteInstalledBlender(id)
			if (selectedBlenderVersionId === id) {
				setSelectedBlenderVersionId(null);
			}
			await reloadInstalled();
			postStatus(`Deleted Blender ${blenderVersionLabel(target)}`);
		} catch (e) {
			console.error(e);
			postStatusError(`Deleting Blender failed: ${e}`);
		}
	}

	const metaLine = (x: IBlenderVersion) =>
		[x.architecture, formatBuildDate(x.file_mtime), buildChannel(x), shortHash(x.hash)]
			.filter((v) => v !== null && v !== undefined && v !== "")
			.join(" · ");

	return (
		<div className='blender_column'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title'>Blender</span>
					<span className='column_header__subtitle'>
						{installedBuilds.length === 0
							? "No versions installed"
							: `${installedBuilds.length} installed`}
					</span>
				</div>
				{isRefreshing ? (
					<InlineLoading className="column_header__loading" iconDescription="Refreshing" />
				) : (
					<Button
						kind="ghost"
						renderIcon={Renew}
						iconDescription="Refresh installed versions"
						title="Refresh installed versions"
						hasIconOnly
						onClick={refreshInstalled}
					/>
				)}
			</div>
			<div className='column_actions'>
				<Button
					kind="secondary"
					size="lg"
					className={`install_button ${isInstallBlenderOpen ? "install_button--active" : ""}`}
					title={isInstallBlenderOpen ? "Back to addons" : "Install a new Blender version"}
					onClick={() => setIsInstallBlenderOpen(!isInstallBlenderOpen)}
				>
					<Add /> Install Blender
				</Button>
			</div>
			<div className='list_header blender_column__list_header'>
				<span>Version</span>
				<span className='centered'>Default</span>
				<span></span>
			</div>
			<div className='blender_column__list' ref={listRef}>
				{installedBuilds.length === 0 ? (
					<div className='blender_column__empty'>
						Nothing installed yet.
					</div>
				) : installedBuilds.map((x) => {
					const isSelected = selectedVersion?.id === x.id;
					const isNew = newlyInstalledBlenderIds.includes(x.id);
					const variant = describeBuildVariant(x.release_cycle, x.risk_id, x.series);
					return (
						<div
							key={x.id}
							className={`blender_row ${isSelected ? "blender_row--selected" : ""}`}
							title={`Blender ${blenderVersionLabel(x)}`}
							onClick={() => selectVersion(x.id)}
							onContextMenu={(e) => showContextMenu(e, [
								{ text: 'Open file location', action: () => { void revealBlenderVersion(x.id); } },
							])}
						>
							<div className='blender_row__main'>
								<div className='blender_row__title'>
									<span className='blender_row__version'>{x.version}</span>
									{variant && (
										<span className={`blender_tag blender_tag--small blender_tag--${variant.kind}`}>{variant.label}</span>
									)}
									{isNew && (
										<span className='blender_tag blender_tag--small blender_tag--new'>New</span>
									)}
								</div>
								<span className='blender_row__meta'>{metaLine(x)}</span>
							</div>
							<Button
								kind="ghost"
								size="md"
								className={`blender_row__star ${x.is_default ? "blender_row__star--default" : ""}`}
								renderIcon={x.is_default ? StarFilled : Star}
								iconDescription={x.is_default ? "Default version" : "Set as default"}
								title={x.is_default ? "Default version" : "Set as default"}
								hasIconOnly
								onClick={async (e: React.MouseEvent) => {
									e.stopPropagation();
									if (!x.is_default) {
										await setAsDefault(x.id);
									}
								}}
							/>
							<Button
								kind="ghost"
								size="md"
								className='blender_row__delete'
								renderIcon={TrashCan}
								iconDescription={`Delete Blender ${blenderVersionLabel(x)}`}
								title={`Delete Blender ${blenderVersionLabel(x)}`}
								hasIconOnly
								onClick={async (e: React.MouseEvent) => {
									e.stopPropagation();
									await deleteVersion(x.id);
								}}
							/>
						</div>
					)
				})}
			</div>
		</div>
	)
}

export default BlenderColumn

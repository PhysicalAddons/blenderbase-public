import { useEffect, useMemo, useRef, useState } from 'react'
import { Button, InlineLoading, Modal, Search } from '@carbon/react';
import { NavLink } from 'react-router-dom';
import { open } from '@tauri-apps/plugin-dialog';
import { join } from '@tauri-apps/api/path';
import { ArrowLeft, Checkmark, Download, Renew } from '@carbon/react/icons';
import { invoke } from '@tauri-apps/api/core';
import { emit, listen } from '@tauri-apps/api/event';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { useNetworkInformationStore } from '../../store/networkInformationStore';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { IBlenderInstallationLocation, IBlenderVersion, IBlenderVersionBuildType, IBlenderVersionDownloadBuildTypeFilter, IDownloadableBlenderVersion, IDownloadFileRef, IDownloadDataSelectedEvent } from '../../models';
import { BlenderBuildKind } from '../../enums';
import { fromStrBlenderBuildTypeKind } from '../../enums/helpers';
import { COMPLETED_LOWERCASE, DAILY_LOWERCASE, DOWNLOADING_LOWERCASE, FAILED_LOWERCASE, PATCH_LOWERCASE, RELEASE_LOWERCASE } from '../../constants';
import { describeBuildVariant, downloadFile, formatBuildDate, IBuildVariant, isDownloadableBuildInstalled } from '../../utility';
import { usePagedScroll } from '../../utility/usePagedScroll';
import { postStatus, postStatusError } from '../../store/statusStore';
import { BlenderService } from '../../services/blenderService';
import { SettingsService } from '../../services/settingsService';
import { useShallow } from 'zustand/react/shallow';

const blenderService = new BlenderService();
const settingsService = new SettingsService();

const formatFileSize = (bytes: number): string => {
	if (!bytes) {
		return "";
	}
	const mb = bytes / (1024 * 1024);
	return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${Math.round(mb)} MB`;
}

const InstallBlenderPanel = () => {
	const [buildTypes, setBuildTypes] = useState<IBlenderVersionBuildType[]>([])
	const [searchText, setSearchText] = useState<string>("")
	const [downloadDirectory, setDownloadDirectory] = useState<string>("")
	// First-download prompt: the location waiting for confirmation and the build that triggered it.
	const [locationPrompt, setLocationPrompt] = useState<{ location: IBlenderInstallationLocation, path: string, build: IDownloadableBlenderVersion, buttonId: string } | null>(null)
	const [isConfirmingLocation, setIsConfirmingLocation] = useState<boolean>(false)
	const [isFetching, setIsFetching] = useState<boolean>(false)
	const listRef = useRef<HTMLDivElement>(null)
	usePagedScroll(listRef, { rowSelector: '.download_row' })
	const { installedBuilds, downloadableBuilds, activeDownloadBuildType, setActiveDownloadBuildType, setDownloadableBuilds, setInstalledBuilds } = useBlenderManagerStore(
		useShallow((s) => ({
			installedBuilds: s.installedBuilds,
			downloadableBuilds: s.downloadableBuilds,
			activeDownloadBuildType: s.activeDownloadBuildType,
			setActiveDownloadBuildType: s.setActiveDownloadBuildType,
			setDownloadableBuilds: s.setDownloadableBuilds,
			setInstalledBuilds: s.setInstalledBuilds,
		}))
	)
	const hasInternetConnection = useNetworkInformationStore((s) => s.hasInternetConnection)
	const { setIsInstallBlenderOpen, addNewlyInstalledBlenderId } = useUiControlsStore(
		useShallow((s) => ({ setIsInstallBlenderOpen: s.setIsInstallBlenderOpen, addNewlyInstalledBlenderId: s.addNewlyInstalledBlenderId }))
	)
	const pendingDownloadRef = useRef<IDownloadFileRef | null>(null);

	useEffect(() => {
		let cancelled = false;
		if (hasInternetConnection === true && buildsForList.length === 0) {
			// Results land in the store; only the local fetching flag needs the mounted guard.
			fetchBlenderVersionBuilds(() => cancelled);
		}
		return () => {
			cancelled = true;
		};
	}, [hasInternetConnection])

	useEffect(() => {
		let cancelled = false;
		const ifMounted = <T,>(setter: (v: T) => void) => (v: T) => {
			if (!cancelled) {
				setter(v);
			}
		};
		fetchBlenderVersionBuildTypes(ifMounted(setBuildTypes));
		fetchDownloadDirectory(ifMounted(setDownloadDirectory));

		const unlisten = listen("download-data-selected", async (event) => {
			const payload = event.payload as IDownloadDataSelectedEvent;
			const selectedPath = payload?.blenderInstallationLocation.directory_path;
			const pending = pendingDownloadRef.current;
			if (!selectedPath || !pending) {
				return;
			}
			const { build, url, fileName, buttonId } = pending;
			pendingDownloadRef.current = null;
			const label = `Blender ${build.version} ${build.risk_id ?? ""}`.trim();
			// Platform separator: a hard-coded backslash put the download in the
			// wrong place on macOS and Linux.
			const archiveFilePath = await join(selectedPath, fileName);
			try {
				postStatus(`Downloading ${label}…`, true);
				await blenderService.updateBlenderVersionDownloadStatusType(payload.blenderVersion, DOWNLOADING_LOWERCASE);
				const downloaded = await downloadFile(url, archiveFilePath, buttonId, (percent) => postStatus(`Downloading ${label} · ${percent}%`, true));
				if (!downloaded) {
					await blenderService.updateBlenderVersionDownloadStatusType(payload.blenderVersion, FAILED_LOWERCASE);
					postStatusError(`Download of ${label} failed`);
					return;
				}
				await blenderService.updateBlenderVersionDownloadStatusType(payload.blenderVersion, COMPLETED_LOWERCASE);
				postStatus(`Installing ${label}…`, true);
				// The backend reports where it unpacked the version (the archive's
				// top-level folder, or the folder holding Blender.app on macOS).
				const installedDirectory = await blenderService.installBlenderVersion(payload.blenderVersion.id, archiveFilePath);
				await blenderService.writeBlenderVersionDownloadData(build, installedDirectory);
				// Show the freshly installed version in the left column with its "New" tag.
				addNewlyInstalledBlenderId(payload.blenderVersion.id);
				await setInstalledBuilds();
				postStatus(`Installed ${label}`);
			} catch (e) {
				console.error(e);
				postStatusError(`Installing ${label} failed: ${e}`);
			}
		});

		return () => {
			cancelled = true;
			unlisten.then((off) => off());
		};
	}, []);

	const buildsForList = useMemo(() => {
		switch (fromStrBlenderBuildTypeKind(activeDownloadBuildType?.text!)) {
			case BlenderBuildKind.Release:
				return downloadableBuilds.releaseBuilds
			case BlenderBuildKind.Daily:
				return downloadableBuilds.dailyBuilds
			case BlenderBuildKind.Patch:
				return downloadableBuilds.patchBuilds
			default:
				return []
		}
	}, [downloadableBuilds, activeDownloadBuildType])

	const filteredBuilds = useMemo(() => {
		const q = searchText.trim().toLowerCase();
		if (q.length === 0) {
			return buildsForList;
		}
		return buildsForList.filter((b) =>
			[b.version, b.risk_id, b.release_cycle, b.patch, b.hash, b.branch]
				.some((v) => (v ?? "").toString().toLowerCase().includes(q))
		);
	}, [buildsForList, searchText])

	const activeBuildKind = fromStrBlenderBuildTypeKind(activeDownloadBuildType?.text!);

	const buildCounts = () => {
		const d = useBlenderManagerStore.getState().downloadableBuilds;
		return `${d.releaseBuilds.length} release, ${d.dailyBuilds.length} daily, ${d.patchBuilds.length} patch`;
	}

	/** @param isCancelled tells whether the component has unmounted since the call started. */
	const fetchBlenderVersionBuilds = async (isCancelled: () => boolean = () => false) => {
		setIsFetching(true);
		postStatus("Fetching release builds from download.blender.org and daily and patch builds from builder.blender.org…", true);
		try {
			if (buildsForList.length === 0) {
				// The three sources are independent; fetch them at the same time.
				await Promise.all([
					setDownloadableBuilds(RELEASE_LOWERCASE),
					setDownloadableBuilds(DAILY_LOWERCASE),
					setDownloadableBuilds(PATCH_LOWERCASE),
				]);
			}
			postStatus(`Blender versions updated · ${buildCounts()}`);
		} catch (e) {
			console.error(e);
			postStatusError(`Fetching Blender versions failed: ${e}`);
		} finally {
			if (!isCancelled()) {
				setIsFetching(false);
			}
		}
	}

	const fetchBlenderVersionBuildTypes = async (apply: (types: IBlenderVersionBuildType[]) => void) => {
		try {
			const types: IBlenderVersionBuildType[] = await blenderService.fetchBlenderVersionBuildTypes(null, null, null);
			const defaultBuildType = types.find((x) => x.is_default) ?? types[0];
			setActiveDownloadBuildType(defaultBuildType ? { id: defaultBuildType.id, text: defaultBuildType.text } as IBlenderVersionDownloadBuildTypeFilter : null);
			apply(types);
		} catch (e) {
			setActiveDownloadBuildType(null);
			apply([]);
			console.error(e);
			postStatusError(`Loading build types failed: ${e}`);
		}
	}

	const fetchDownloadDirectory = async (apply: (directoryPath: string) => void) => {
		try {
			const locations: IBlenderInstallationLocation[] = await settingsService.fetchBlenderInstallationPaths(null, null, null, true);
			apply(locations[0]?.directory_path ?? "");
		} catch (e) {
			console.error(e);
			postStatusError(`Loading the download location failed: ${e}`);
		}
	}

	const changeBuildType = async (selectedItem: IBlenderVersionBuildType) => {
		try {
			await blenderService.updateDownloadBlenderBuildType(selectedItem.text);
			setActiveDownloadBuildType({ id: selectedItem.id, text: selectedItem.text })
		} catch (e) {
			console.error(e);
			postStatusError(`Switching build type failed: ${e}`);
		}
	}

	const refresh = async () => {
		if (!activeDownloadBuildType) {
			return;
		}
		setIsFetching(true);
		const kind = activeDownloadBuildType.text.toLowerCase();
		const source = kind === RELEASE_LOWERCASE ? "download.blender.org" : "builder.blender.org";
		postStatus(`Fetching ${kind} builds from ${source}…`, true);
		try {
			await setDownloadableBuilds(activeDownloadBuildType.text);
			postStatus(`${kind.charAt(0).toUpperCase() + kind.slice(1)} builds updated · ${buildsForList.length} listed · ${buildCounts()}`);
		} catch (e) {
			console.error(e);
			postStatusError(`Fetching ${kind} builds failed: ${e}`);
		} finally {
			setIsFetching(false);
		}
	}

	const startDownload = async (build: IDownloadableBlenderVersion, buttonId: string, location: IBlenderInstallationLocation) => {
		pendingDownloadRef.current = { build, url: build.url, fileName: build.file_name, buttonId };
		try {
			const blenderVersion: IBlenderVersion = await invoke("cmd_init_blender_version", {
				downloadableBlenderVersion: build,
				blenderInstallationLocation: location
			});
			await emit("download-data-selected", {
				blenderInstallationLocation: location,
				blenderVersion: blenderVersion
			});
		} catch (e) {
			console.error(e);
			postStatusError(`Could not start the download: ${e}`);
		}
	};

	/**
	 * Downloads right away, or asks where to install first if the location was never confirmed.
	 * With no location at all, the folder picker opens directly: choosing a folder registers
	 * it, confirms it and starts the download; cancelling simply does nothing.
	 */
	const processDownload = async (build: IDownloadableBlenderVersion, buttonId: string) => {
		try {
			const locations: IBlenderInstallationLocation[] = await settingsService.fetchBlenderInstallationPaths(null, null, null, true);
			let location = locations[0];
			if (!location) {
				const picked = await settingsService.insertBlenderInstallationLocation();
				if (!picked) {
					return;
				}
				if (!picked.is_default) {
					// The toggle takes the current state; passing false makes this the default.
					await settingsService.setBlenderInstallationLocationAsDefault(picked.id, picked.is_default);
				}
				const confirmed = picked.is_confirmed
					? picked
					: await settingsService.confirmBlenderInstallationLocation(picked.id, picked.directory_path);
				setDownloadDirectory(confirmed.directory_path);
				postStatus(`Blender versions will be installed in ${confirmed.directory_path}`);
				await startDownload(build, buttonId, confirmed);
				return;
			}
			if (!location.is_confirmed) {
				setLocationPrompt({ location, path: location.directory_path, build, buttonId });
				return;
			}
			await startDownload(build, buttonId, location);
		} catch (e) {
			console.error(e);
			postStatusError(`Could not start the download: ${e}`);
		}
	};

	const pickPromptDirectory = async () => {
		if (!locationPrompt) {
			return;
		}
		try {
			const selected = await open({ multiple: false, directory: true, title: "Choose where Blender versions are installed", defaultPath: locationPrompt.path });
			if (typeof selected === "string" && selected.length > 0) {
				setLocationPrompt({ ...locationPrompt, path: selected });
			}
		} catch (e) {
			console.error(e);
		}
	};

	const confirmPromptAndDownload = async () => {
		if (!locationPrompt) {
			return;
		}
		setIsConfirmingLocation(true);
		try {
			const confirmed = await settingsService.confirmBlenderInstallationLocation(locationPrompt.location.id, locationPrompt.path);
			setDownloadDirectory(confirmed.directory_path);
			postStatus(`Blender versions will be installed in ${confirmed.directory_path}`);
			const { build, buttonId } = locationPrompt;
			setLocationPrompt(null);
			await startDownload(build, buttonId, confirmed);
		} catch (e) {
			console.error(e);
			postStatusError(`Could not use that location: ${e}`);
		} finally {
			setIsConfirmingLocation(false);
		}
	};

	const buildTag = (b: IDownloadableBlenderVersion): IBuildVariant | null => {
		if (activeBuildKind === BlenderBuildKind.Patch && b.patch) {
			return { label: b.patch, kind: "neutral" };
		}
		return describeBuildVariant(b.release_cycle, b.risk_id);
	}

	const metaLine = (b: IDownloadableBlenderVersion) =>
		[b.architecture, formatBuildDate(b.file_mtime), b.hash, b.file_extension, formatFileSize(b.file_size)]
			.filter((v) => v !== null && v !== undefined && v !== "")
			.join(" · ");

	const selectedBuildTypeIndex = Math.max(0, buildTypes.findIndex((t) => t.id === activeDownloadBuildType?.id));

	return (
		<div className='install_blender_panel'>
			<Modal
				open={locationPrompt !== null}
				size="sm"
				modalHeading="Where should Blender versions be installed?"
				modalLabel="First download"
				primaryButtonText={isConfirmingLocation ? "Checking…" : "Continue"}
				secondaryButtonText="Cancel"
				primaryButtonDisabled={isConfirmingLocation}
				onRequestClose={() => !isConfirmingLocation && setLocationPrompt(null)}
				onRequestSubmit={confirmPromptAndDownload}
			>
				<p className='location_prompt__text'>
					Every version you download is unpacked into its own folder here. You can change it later in Settings.
				</p>
				<div className='location_prompt__row'>
					<code className='location_prompt__path' title={locationPrompt?.path ?? ""}>{locationPrompt?.path ?? ""}</code>
					<Button kind="tertiary" size="md" onClick={pickPromptDirectory} disabled={isConfirmingLocation}>
						Change…
					</Button>
				</div>
			</Modal>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title'>Install Blender</span>
					<span className='column_header__subtitle' title={downloadDirectory}>
						{downloadDirectory
							? <>Downloads go to {downloadDirectory} (<NavLink to="/settings">change in Settings</NavLink>)</>
							: <>Set a download location in <NavLink to="/settings">Settings</NavLink></>}
					</span>
				</div>
				<Button
					kind="ghost"
					size="lg"
					className='install_blender_panel__back'
					title="Back to addons"
					onClick={() => setIsInstallBlenderOpen(false)}
				>
					<ArrowLeft /> Back to Addons
				</Button>
			</div>
			<div className='column_actions install_blender_panel__toolbar'>
				{buildTypes.length > 0 && (
					<div className='build_type_switch' role="tablist" aria-label="Build type">
						{buildTypes.map((t, i) => (
							<Button
								key={t.id}
								kind="secondary"
								size="lg"
								role="tab"
								aria-selected={i === selectedBuildTypeIndex}
								className={`build_type_switch__option ${i === selectedBuildTypeIndex ? "build_type_switch__option--selected" : ""}`}
								title={`${t.text} builds`}
								onClick={() => changeBuildType(t)}
							>
								{t.text.charAt(0).toUpperCase() + t.text.slice(1).toLowerCase()}
							</Button>
						))}
					</div>
				)}
				<div className='install_blender_panel__filters'>
					<Search
						size="lg"
						labelText="Search versions"
						placeholder="Search versions"
						value={searchText}
						onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSearchText(e.target.value ?? "")}
					/>
					{isFetching ? (
						<InlineLoading className='install_blender_panel__fetching' iconDescription="Fetching" />
					) : (
						<Button
							kind="ghost"
							renderIcon={Renew}
							iconDescription="Retrieve downloadable Blender data"
							title="Retrieve downloadable Blender data"
							hasIconOnly
							disabled={!hasInternetConnection}
							onClick={() => refresh()}
						/>
					)}
				</div>
			</div>
			<div className='list_header install_blender_panel__list_header'>
				<span>Version</span>
				<span></span>
			</div>
			<div className='install_blender_panel__list' ref={listRef}>
				{!hasInternetConnection ? (
					<div className='install_blender_panel__empty'>
						No internet connection. Connect to see downloadable Blender versions.
					</div>
				) : filteredBuilds.length === 0 ? (
					<div className='install_blender_panel__empty'>
						{isFetching
							? "Fetching available Blender versions from blender.org…"
							: searchText.trim().length > 0
								? "No versions match the search."
								: "No downloadable Blender versions found. Use the refresh button to fetch them."}
					</div>
				) : filteredBuilds.map((b) => {
					const installed = isDownloadableBuildInstalled(installedBuilds, b);
					const tag = buildTag(b);
					const buttonId = `download-btn-${b.file_name}`;
					return (
						<div key={b.file_name} className='download_row' title={`Blender ${b.version} ${b.risk_id}`}>
							<div className='download_row__main'>
								<div className='download_row__title'>
									<span className='download_row__version'>{b.version}</span>
									{tag && (
										<span className={`blender_tag blender_tag--small blender_tag--${tag.kind}`}>{tag.label}</span>
									)}
								</div>
								<span className='download_row__meta'>{metaLine(b)}</span>
							</div>
							<div className='download_row__action'>
								{installed ? (
									<span className='download_row__installed'>
										<Checkmark /> Installed
									</span>
								) : (
									<Button
										kind="secondary"
										size="lg"
										className='download_button'
										title={`Download Blender ${b.version} ${b.risk_id}`}
										renderIcon={Download}
										disabled={!hasInternetConnection}
										onClick={() => processDownload(b, buttonId)}
									>
										<span id={buttonId} className='progress'>Download</span>
									</Button>
								)}
							</div>
						</div>
					)
				})}
			</div>
		</div>
	)
}

export default InstallBlenderPanel

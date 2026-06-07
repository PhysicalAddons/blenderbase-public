import { useEffect, useMemo, useRef, useState } from 'react'
import { useLoadingManagerStore } from '../../../store/loadingManagerStore'
import { Button, DataTable, Dropdown, Table, TableBody, TableCell, TableContainer, TableHead, TableHeader, TableRow, TableToolbar, TableToolbarContent, TableToolbarSearch } from '@carbon/react';
import { Download, Renew } from '@carbon/react/icons';
import { IBlenderInstallationLocation, IBlenderVersion, IBlenderVersionBuildType, IBlenderVersionDownloadBuildTypeFilter, IBuildColumn, IDownloadableBlenderVersion, IDownloadFileRef, IDownloadDataSelectedEvent } from '../../../models';
import { invoke } from '@tauri-apps/api/core';
import { BlenderBuildKind } from '../../../enums';
import { useBlenderManagerStore } from '../../../store/blenderManagerStore';
import { useNetworkInformationStore } from '../../../store/networkInformationStore';
import { fromStrBlenderBuildTypeKind } from '../../../enums/helpers';
import { COMPLETED_LOWERCASE, DAILY_LOWERCASE, DOWNLOADING_LOWERCASE, FAILED_LOWERCASE, PATCH_LOWERCASE, RELEASE_LOWERCASE } from '../../../constants';
import { downloadFile } from '../../../utility';
import { emit, listen } from '@tauri-apps/api/event';
import { BlenderService } from '../../../services/blenderService';
import { SettingsService } from '../../../services/settingsService';

const blenderService = new BlenderService();
const settingsService = new SettingsService();

const DownloadBlenderTable = () => {
	const [blenderVersionBuildTypesFilters, setblenderVersionBuildTypesFilters] = useState<IBlenderVersionDownloadBuildTypeFilter[]>([])
	const loadingManagerStore = useLoadingManagerStore();
	const { downloadableBuilds, activeDownloadBuildType, setActiveDownloadBuildType, setDownloadableBuilds } = useBlenderManagerStore()
	const { hasInternetConnection } = useNetworkInformationStore()
	const pendingDownloadRef = useRef<IDownloadFileRef | null>(null);
	useEffect(() => {
		async function init() {
			if (hasInternetConnection === true && buildsForTable.length === 0) {
				await fetchBlenderVersionBuilds();
			}
		}
		init();
	}, [hasInternetConnection])
	useEffect(() => {
		async function init() {
			await fetchBlenderVersionBuildTypes();
		}
		init();

		const unlisten = listen("download-data-selected", async (event) => {
			const payload = event.payload as IDownloadDataSelectedEvent;
			const selectedPath = payload?.blenderInstallationLocation.directory_path;
			const pending = pendingDownloadRef.current;
			if (selectedPath && pending) {
				const { build, url, fileName, buttonId } = pending;
				pendingDownloadRef.current = null;
				await blenderService.updateBlenderVersionDownloadStatusType(payload?.blenderVersion, DOWNLOADING_LOWERCASE);
				let archiveFilePath = `${selectedPath}\\${fileName}`;
				let resultStatus = await downloadFile(url, archiveFilePath, buttonId);
				if (resultStatus === true) {
					await blenderService.updateBlenderVersionDownloadStatusType(payload?.blenderVersion, COMPLETED_LOWERCASE);
				} else {
					await blenderService.updateBlenderVersionDownloadStatusType(payload?.blenderVersion, FAILED_LOWERCASE);
				}
				await blenderService.installBlenderVersion(payload?.blenderVersion.id, archiveFilePath);
				await blenderService.writeBlenderVersionDownloadData(build, archiveFilePath.replace(".zip", ""));
			}
		});

		return () => {
			unlisten.then((off) => off());
		};
	}, []);
	const buildsForTable = useMemo(() => {
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

	const fetchBlenderVersionBuilds = async () => {
		try {
			if (buildsForTable.length === 0) {
				await setDownloadableBuilds(RELEASE_LOWERCASE);
				await setDownloadableBuilds(DAILY_LOWERCASE);
				await setDownloadableBuilds(PATCH_LOWERCASE);
			}
		} catch (e) {
			console.error(e);
		}
	}

	const fetchBlenderVersionBuildTypes = async () => {
		try {
			const buildTypes: IBlenderVersionBuildType[] = await blenderService.fetchBlenderVersionBuildTypes(null, null, null);
			const defaultBuiltType = buildTypes.filter(x => x.is_default == true)[0];
			setActiveDownloadBuildType({ id: defaultBuiltType.id, text: defaultBuiltType.text } as IBlenderVersionDownloadBuildTypeFilter);
			setblenderVersionBuildTypesFilters(buildTypes);
		} catch (e) {
			setActiveDownloadBuildType(null);
			setblenderVersionBuildTypesFilters([]);
			console.error(e);
		}
	}

	const changeFilter = async (selectedItem: IBlenderVersionDownloadBuildTypeFilter) => {
		try {
			await blenderService.updateDownloadBlenderBuildType(selectedItem.text);
			setActiveDownloadBuildType(selectedItem)
		} catch (e) {
			console.error(e);
		}
	}

	const refresh = async () => {
		if (!activeDownloadBuildType) {
			return;
		}
		await setDownloadableBuilds(activeDownloadBuildType.text);
	}

	const processDownload = async (build: IDownloadableBlenderVersion, url: string, fileName: string, buttonId: string) => {
		pendingDownloadRef.current = { build, url, fileName, buttonId };
		try {
			// await handleOpenPopup(); // ? skip popup.
			const defaultBlenderInstallationLocation: IBlenderInstallationLocation[] = await settingsService.fetchBlenderInstallationPaths(null, null, null, true);

			const blenderVersion: IBlenderVersion = await invoke("cmd_init_blender_version", {
				downloadableBlenderVersion: build,
				blenderInstallationLocation: defaultBlenderInstallationLocation[0]
			});
			await emitDownload(defaultBlenderInstallationLocation[0], blenderVersion);
		} catch (e) {
			console.error(e);
		}
	};

	const emitDownload = async (blenderInstallationLocation: IBlenderInstallationLocation, blenderVersion: IBlenderVersion) => {
		try {
			await emit("download-data-selected",
				{
					blenderInstallationLocation: blenderInstallationLocation,
					blenderVersion: blenderVersion
				}
			);
		} catch (e) {
			console.error(e);
		}
	};

	// const handleOpenPopup = async () => {
	// 	try {
	// 		await fileSystemService.instancePopupWindow(
	// 			"download-file-popup",
	// 			"Choose Download Location",
	// 			"/#/standalone/DownloadFilePopup",
	// 			600.0,
	// 			200.0,
	// 			false,
	// 			true,
	// 			true,
	// 			true
	// 		);
	// 	} catch (e) {
	// 		console.error(e);
	// 	}
	// };

	return (
		<div>
			<div className="blender_version_table">
				<DataTable
					isSortable={true}
					rows={
						buildsForTable.map((data: IDownloadableBlenderVersion, index: number) => ({
							id: index.toString(),
							version: data.version,
							build: {
								release_cycle: data.release_cycle,
								patch: data.patch
							} as IBuildColumn,
							metadata: JSON.stringify({
								"build_date": new Date(data.file_mtime * 1000).toLocaleDateString('en-US', {
									day: '2-digit',
									month: 'long',
									year: 'numeric'
								}),
								"download_data": data,
							}),
							actions: data
						}))
					}
					headers={[
						{ key: 'version', header: 'Version' },
						{ key: 'build', header: 'Build' },
						{ key: 'metadata', header: 'Metadata' },
						{ key: 'actions', header: 'Actions' }
					]}>
					{({
						rows,
						headers,
						getHeaderProps,
						getRowProps,
						getTableProps,
						getToolbarProps,
						onInputChange,
						getTableContainerProps
					}) => (
						<TableContainer {...getTableContainerProps()}>
							<TableToolbar {...getToolbarProps()} aria-label="data table toolbar">
								<TableToolbarContent>
									<TableToolbarSearch onChange={onInputChange as any} title="Search table" />
									<Button
										kind="ghost"
										renderIcon={Renew}
										iconDescription="Retrieve Downloadable Blender data"
										title="Retrieve Downloadable Blender data"
										hasIconOnly
										disabled={!hasInternetConnection}
										onClick={() => refresh()}
									/>
									<Dropdown
										size="lg"
										id="default"
										label="Filter"
										titleText=""
										items={blenderVersionBuildTypesFilters}
										itemToString={(item: { text: any; }) => item ? item.text : ''}
										initialSelectedItem={activeDownloadBuildType} // ({ text: selectedBlenderVersionBuildTypeFilter?.text })
										onChange={(e: any) => changeFilter(e.selectedItem)}
										disabled={
											(
												loadingManagerStore.isRegisteringDownloadableBlenderData === true ||
												loadingManagerStore.isDownloadingBlender === true
											) ? true : false
										}
									/>
								</TableToolbarContent>
							</TableToolbar>
							<Table {...getTableProps()}>
								<TableHead>
									<TableRow>
										{headers.map((header) => {
											const { key, ...rest } = getHeaderProps({ header });
											return (
												<TableHeader
													key={key}
													{...rest}
												>
													{header.header}
												</TableHeader>
											);
										})}
									</TableRow>
								</TableHead>
								{rows.length === 0 && activeDownloadBuildType !== null ?
									<TableBody>
										<TableRow className='empty_downloadable_blender_table'>
											<TableCell key={1}>
												<>No downloadable Blender versions found.</>
											</TableCell>
											<TableCell key={2}>
											</TableCell>
											<TableCell key={3}>
											</TableCell>
											<TableCell key={4}>
											</TableCell>
										</TableRow>
									</TableBody>
									:
									<TableBody>
										{rows.map((row) => (
											<TableRow {...getRowProps({ row })} key={row.id}>
												{row.cells.map((cell) => (
													<TableCell key={cell.id}>
														{cell.info.header === 'version' ? (
															<div
																className='blender_title'
																title={`Blender version ${cell.value}`}
															>
																{cell.value}
																<svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
																	<path d="M16.1596 17.1545C16.2176 16.1206 16.7239 15.2096 17.4878 14.5639C18.2369 13.9296 19.2451 13.542 20.3452 13.542C21.4443 13.542 22.4525 13.9296 23.2022 14.5639C23.9655 15.2096 24.4718 16.1206 24.5303 17.1534C24.5882 18.2158 24.1612 19.2027 23.4121 19.9343C22.6483 20.6785 21.5618 21.1454 20.3452 21.1454C19.1286 21.1454 18.04 20.6785 17.2767 19.9343C16.5271 19.2027 16.1011 18.2158 16.1596 17.1545Z" fill="white" />
																	<path d="M9.49505 19.248C9.50216 19.6526 9.6312 20.4389 9.82476 21.0529C10.2316 22.3526 10.9216 23.555 11.8817 24.6146C12.867 25.7038 14.0804 26.5786 15.4818 27.1998C16.9548 27.8521 18.5508 28.1845 20.2087 28.1818C21.8638 28.1796 23.4598 27.8406 24.9329 27.1834C26.3342 26.5562 27.5464 25.6776 28.5301 24.5878C29.4897 23.5238 30.1787 22.3193 30.5866 21.0196C30.7916 20.3629 30.9212 19.6964 30.9731 19.0277C31.024 18.3688 31.0027 17.7089 30.9092 17.0495C30.7265 15.7645 30.282 14.5589 29.5974 13.4599C28.9714 12.45 28.1643 11.5659 27.2047 10.8217L27.2069 10.8201L17.5229 3.38451C17.5141 3.37795 17.507 3.37085 17.4977 3.36483C16.8624 2.87711 15.794 2.87876 15.0952 3.36757C14.3887 3.86185 14.3078 4.67927 14.9366 5.19488L14.9339 5.19761L18.9729 8.48209L6.66219 8.4952H6.64578C5.62822 8.4963 4.65004 9.1639 4.45648 10.0076C4.25745 10.8671 4.94857 11.5801 6.00659 11.5839L6.00496 11.5878L12.2448 11.5757L1.11018 20.1223C1.09596 20.1327 1.08065 20.1437 1.06752 20.154C0.0171601 20.9584 -0.322383 22.2958 0.339222 23.1422C1.01067 24.0028 2.43831 24.0044 3.49961 23.1471L9.57652 18.1736C9.57652 18.1736 9.48795 18.8451 9.49505 19.248ZM25.1105 21.4964C23.8584 22.772 22.1054 23.4954 20.2087 23.4992C18.3091 23.5025 16.5562 22.7857 15.304 21.5122C14.6922 20.8916 14.2427 20.1776 13.9655 19.417C13.6938 18.6696 13.5882 17.8762 13.6582 17.0757C13.7244 16.2933 13.9573 15.5469 14.3291 14.8717C14.6944 14.2079 15.1974 13.6081 15.8186 13.1007C17.0357 12.1088 18.5853 11.5719 20.2059 11.5697C21.8282 11.5675 23.3767 12.0995 24.5949 13.0881C25.215 13.5933 25.7175 14.1909 26.0827 14.8536C26.4562 15.5283 26.6875 16.272 26.7564 17.0566C26.8253 17.856 26.7197 18.6482 26.448 19.3962C26.1702 20.159 25.7224 20.873 25.1105 21.4964Z" fill="white" />
																</svg>
															</div>
														) : cell.info.header === 'build' &&
															(
																fromStrBlenderBuildTypeKind(activeDownloadBuildType!.text) === BlenderBuildKind.Daily ||
																fromStrBlenderBuildTypeKind(activeDownloadBuildType!.text) === BlenderBuildKind.Release
															) ? (
															<div
																className={`blender_tag ${cell.value}`}
																title={`${cell.value} build`}
															>
																{cell.value.release_cycle}
															</div>
														) : cell.info.header === 'build' &&
															fromStrBlenderBuildTypeKind(activeDownloadBuildType!.text) === BlenderBuildKind.Patch
															? (
																<div
																	className={`blender_tag ${cell.value}`}
																	title={`${cell.value} build`}
																>
																	{cell.value.patch}
																</div>
															) : cell.info.header === 'metadata' ? (
																<div className='blender_meta'>
																	<div>
																		{JSON.parse(cell.value).download_data.architecture}
																	</div>
																	<div>
																		<span>
																			{JSON.parse(cell.value).build_date}
																		</span>
																	</div>
																	<div>
																		{JSON.parse(cell.value).download_data.commit_link && (
																			<a target="_blank" href={JSON.parse(cell.value).download_data.commit_link}>
																				{JSON.parse(cell.value).download_data.commit_hash}
																			</a>
																		)}
																	</div>
																	<div>
																		{JSON.parse(cell.value).download_data.pull_request_link && (
																			<a target="_blank" href={JSON.parse(cell.value).download_data.pull_request_link}>
																				{JSON.parse(cell.value).download_data.pull_request_number}
																			</a>
																		)}
																	</div>
																	<div>
																		<div>
																			{JSON.parse(cell.value).download_data.download_link && (
																				<a target="_blank" href={JSON.parse(cell.value).download_data.download_link}>
																					{JSON.parse(cell.value).download_data.file_type}
																				</a>
																			)}
																			{JSON.parse(cell.value).download_data.file_size && (
																				<a target="_blank" href={JSON.parse(cell.value).download_data.download_link}>
																					{JSON.parse(cell.value).download_data.file_size + " B"}
																				</a>
																			)}
																		</div>
																	</div>
																</div>
															) : cell.info.header === 'actions' ? (
																<div>
																	<Button
																		kind="secondary"
																		title={`Download ${cell.value.version} ${cell.value.variant}`}
																		className='download_button'
																		renderIcon={Download}
																		disabled={!hasInternetConnection}
																		onClick={() => processDownload(cell.value, cell.value.url, cell.value.file_name, `download-btn-${cell.value.file_name}`)}
																	>
																		<span
																			id={`download-btn-${cell.value.file_name}`}
																			className='progress'
																		>
																			Download
																		</span>
																	</Button>
																</div>
															) : (
																<div>{cell.value}</div>
															)}
													</TableCell>
												))}
											</TableRow>
										))}
									</TableBody>
								}
							</Table>
						</TableContainer>
					)}
				</DataTable>
			</div>
		</div>

	)
}

export default DownloadBlenderTable
import { useEffect, useState } from 'react'
import { IBlenderInstallationLocation } from '../../../models'
import { Button, DataTable, Table, TableBody, TableCell, TableContainer, TableHead, TableHeader, TableRow, TableToolbar, TableToolbarContent, TableToolbarSearch, Tile, Toggle } from '@carbon/react'
import { FolderAdd, TrashCan } from '@carbon/react/icons'
import { SettingsService } from '../../../services/settingsService'
import { useBlenderManagerStore } from '../../../store/blenderManagerStore'

const settingsService = new SettingsService();

const BlenderInstallationLocationTable = () => {
	const [blenderInstallationLocations, setBlenderInstallationLocations] = useState<IBlenderInstallationLocation[]>([])
	const { setInstalledBuilds } = useBlenderManagerStore()

	useEffect(() => {
		async function init() {
			setBlenderInstallationLocations(await settingsService.fetchBlenderInstallationPaths(null, null, null, null));
		}
		init();
	}, []);

	const handleAddBlenderInstallationPath = async () => {
		try {
			await settingsService.insertBlenderInstallationLocation();
		} catch (e) {
			console.error(e);
		} finally {
			setBlenderInstallationLocations(await settingsService.fetchBlenderInstallationPaths(null, null, null, null));
			await setInstalledBuilds();
		}
	};

	const handleSetBlenderInstallationLocationAsDefault = async (id: string, is_default: boolean) => {
		try {
			await settingsService.setBlenderInstallationLocationAsDefault(id, is_default);
		} catch (e) {
			console.error(e);
		} finally {
			setBlenderInstallationLocations(await settingsService.fetchBlenderInstallationPaths(null, null, null, null));
		}
	};

	const handleDeleteBlenderVersionInstallationPath = async (id: string) => {
		try {
			await settingsService.deleteBlenderInstallationLocation(id);
		} catch (e) {
			console.error(e);
		} finally {
			setBlenderInstallationLocations(await settingsService.fetchBlenderInstallationPaths(null, null, null, null));
		}
	};

	return (
		<div>
			<Tile className='settings_section' style={{ "borderRadius": "8px" }}>
				<div className='blender_installation_directory_settings'>
					<h4 className='blender_installation_directory_heading'>Blender Installation Locations</h4>
					<DataTable
						isSortable={true}
						rows={blenderInstallationLocations}
						headers={[
							{ key: 'directory_path', header: 'Directory path' },
							{ key: 'is_default', header: 'Default', },
							{ key: 'actions', header: 'Actions', },
						]}>
						{({
							rows,
							headers,
							getTableProps,
							getHeaderProps,
							getRowProps,
							getTableContainerProps,
							getToolbarProps,
							onInputChange
						}) => (
							<TableContainer {...getTableContainerProps()}>
								<TableToolbar {...getToolbarProps()} aria-label="data table toolbar">
									<TableToolbarContent>
										<TableToolbarSearch onChange={onInputChange as any} title="Search table" />
										<Button

											kind="ghost"
											renderIcon={FolderAdd}
											iconDescription="Add new installation location"
											title="Add new installation location"
											hasIconOnly
											onClick={() => handleAddBlenderInstallationPath()}
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
									{rows.length === 0 ?
										<TableBody>
											<TableRow className='empty_installed_addon_table'>
												<TableCell key={1}>
													No Blender installation location found
												</TableCell>
												<TableCell key={2}>
												</TableCell>
												<TableCell key={3}>
												</TableCell>
											</TableRow>
										</TableBody>
										:
										<TableBody>
											{rows.map((row) => (
												<TableRow {...getRowProps({ row })} key={row.id}>
													{row.cells.map((cell) => (
														<TableCell key={cell.id}>
															{cell.info.header === "is_default" ? (
																<div>
																	<Toggle
																		id={cell.id}
																		hideLabel={true}
																		toggled={cell.value}
																		onClick={() => handleSetBlenderInstallationLocationAsDefault(row.id, cell.value)}
																	/>
																</div>
															) : cell.info.header === "actions" ? (
																<div>
																	<Button
																		className='delete_blender_installation_location_button'
																		id={cell.id}
																		iconDescription='Remove installation location'
																		title={`Remove installation location`}
																		renderIcon={TrashCan}
																		hasIconOnly
																		onClick={() => handleDeleteBlenderVersionInstallationPath(row.id)}
																	>
																		Del
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
			</Tile>
		</div>
	)
}

export default BlenderInstallationLocationTable
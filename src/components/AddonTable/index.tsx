import { useLoadingManagerStore } from '../../store/loadingManagerStore';
import { Button, DataTable, Dropdown, Table, TableBody, TableCell, TableContainer, TableHead, TableHeader, TableRow, TableToolbar, TableToolbarContent, TableToolbarSearch } from '@carbon/react';
import { IbmCloudDirectLink_1Connect, IntentRequestCreate, Reset } from '@carbon/react/icons';

const AddonTable = () => {
	//#region STORES
	const loadingManagerStore = useLoadingManagerStore();
	//#endregion STORES

	/**
	 * Refreshes the registered Blender and addon data.
	 */
    const refreshInstalledAddonList = () => {
		// await loadingManagerStore.setIsRegisteringInstalledBlenderData(true);
		// try {
		// 	// Recent files data.
		// 	await blendFileManagerStore.processData(await invoke('instance_blend_file_database'))
		// 	// Installed Blender & Addon data.
		// 	await installedBlenderManagerStore.processData(await invoke('refresh_blender_data'));
		// 	// // Installed Blender addon data.
		// 	let activeBlenderVersionObj = await installedBlenderManagerStore.getActiveBlenderVersionObj();
		// 	if (Object.keys(activeBlenderVersionObj).length === 0 || activeBlenderVersionObj.id.length === 0) {
		// 		await addonManagerStore.processData(
		// 			{"installed_addons":[]}
		// 		);
		// 	} else {
		// 		await addonManagerStore.processData(
		// 			await invoke('instance_addon_database', { 
		// 				activeBlenderVersion: activeBlenderVersionObj, 
		// 				addonFilter: await addonManagerStore.getSelectedAddonFilter()
		// 			})
		// 		);
		// 	}
		// } catch (err) {
		// 	await outputMessageObjDialog(err, true); 
		// }
		// await loadingManagerStore.setIsRegisteringInstalledBlenderData(false);
    }

	const installAddon = () => {
		// await loadingManagerStore.setIsInstallingExternalAddon(true);
		// try {
		// 	let selectedPath = await outputFilePathSelectionDialog(false, false, "Select an addon file (.zip or .py).");
		// 	if (selectedPath === null) {
		// 		await loadingManagerStore.setIsInstallingExternalAddon(false);
		// 		return undefined;
		// 	}
		// 	let activeBlenderVersionObjOriginal = await installedBlenderManagerStore.getActiveBlenderVersionObj();
		// 	await invoke('install_external_addon', { 
		// 		blenderVersion: activeBlenderVersionObjOriginal,
		// 		blenderVersionDataTableRows: {"installed_blenders": await installedBlenderManagerStore.getInstalledBlenderDataTableRows()},
		// 		selectedPath: selectedPath,
		// 	});
		// 	let activeBlenderVersionObjNext = await installedBlenderManagerStore.getActiveBlenderVersionObj();
		// 	if (activeBlenderVersionObjOriginal.id === activeBlenderVersionObjNext.id) {
		// 		await addonManagerStore.processData(
		// 			await invoke('instance_addon_database', { 
		// 				activeBlenderVersion: activeBlenderVersionObjOriginal, 
		// 				addonFilter: await addonManagerStore.getSelectedAddonFilter()
		// 			})
		// 		);
		// 	}
		// } catch (err) {
		// 	await outputMessageObjDialog(err, true); 
		// }
		// await loadingManagerStore.setIsInstallingExternalAddon(false);
	}

	const symLinkAddon = () => {
		// await loadingManagerStore.setIsSymlinkingExternalAddon(true);
		// try {
		// 	let selectedPath = await outputFilePathSelectionDialog(true, false, "Select an addon directory.");
		// 	if (selectedPath === null) {
		// 		await loadingManagerStore.setIsSymlinkingExternalAddon(false);
		// 		return undefined;
		// 	}
		// 	let activeBlenderVersionObjOriginal = await installedBlenderManagerStore.getActiveBlenderVersionObj();
		// 	await invoke('symlink_external_addon', { 
		// 		blenderVersion: activeBlenderVersionObjOriginal,
		// 		blenderVersionDataTableRows: {"installed_blenders": await installedBlenderManagerStore.getInstalledBlenderDataTableRows()},
		// 		selectedPath: selectedPath,
		// 	});
		// 	let activeBlenderVersionObjNext = await installedBlenderManagerStore.getActiveBlenderVersionObj();
		// 	if (activeBlenderVersionObjOriginal.id === activeBlenderVersionObjNext.id) {
		// 		await addonManagerStore.processData(
		// 			await invoke('instance_addon_database', { 
		// 				activeBlenderVersion: activeBlenderVersionObjOriginal, 
		// 				addonFilter: await addonManagerStore.getSelectedAddonFilter()
		// 			})
		// 		);
		// 	}
		// } catch (err) {
		// 	await outputMessageObjDialog(err, "error", true); 

		// }
		// await loadingManagerStore.setIsSymlinkingExternalAddon(false);
	}

	/**
	 * Changes the addon filter and displays the filtered addons.
	 * 
	 * @param {JSON} selectedItem - contains the collection for the addon filter. 
	 */
	const changeAddonFilter = () => { // selectedItem: any
		// try {
		// 	let activeBlenderVersionObj = await installedBlenderManagerStore.getActiveBlenderVersionObj();
		// 	if (Object.keys(activeBlenderVersionObj).length === 0 || activeBlenderVersionObj.id.length === 0) {
		// 		await addonManagerStore.processData(
		// 			{"installed_addons":[]}
		// 		);
		// 	} else {
		// 		await addonManagerStore.processData(
		// 			await invoke('instance_addon_database', { 
		// 				activeBlenderVersion: activeBlenderVersionObj, 
		// 				addonFilter: selectedItem
		// 			})
		// 		);
		// 	}
		// 	await invoke('change_active_addon_filter', { 
		// 		addonFilter: selectedItem
		// 	})
		// 	await addonManagerStore.setSelectedAddonFilter(selectedItem);
		// } catch (err) {
		// 	await outputMessageObjDialog(err, "error", true);
		// }
	}

	return (
		<div>
            <div className='addon_table'>
				<DataTable 
					isSortable={true}
					rows={[]}
					headers={[
					    { key: 'name', header: 'Name' },
                        { key: 'version', header: 'Version' },
                        { key: 'type', header: 'Type', },
                        { key: 'enabled', header: 'Enabled' },
                        { key: 'actions', header: '' }
					]}
					render={({
                        headers,
                        getHeaderProps,
                        getTableProps,
                        getToolbarProps,
                        onInputChange,
                        getTableContainerProps
                    }) => (
                        <TableContainer {...getTableContainerProps()}>
                            <TableToolbar {...getToolbarProps()} aria-label="data table toolbar">
                                <TableToolbarContent>
                                    <TableToolbarSearch onChange={() => onInputChange} title="Search table" />
									{/* <InlineLoading
										status="active"
										iconDescription="Refreshing database"
									/> */}
									<Button 
										kind="ghost"
										renderIcon={Reset}
										iconDescription="Refresh installed add-on list"
										title="Refresh installed add-on list"
										hasIconOnly
										disabled={
											loadingManagerStore.isInstallingExternalAddon === true ||
											loadingManagerStore.isSymlinkingExternalAddon === true ||
											loadingManagerStore.isUninstallingExternalAddon === true ||
											loadingManagerStore.isRegisteringInstalledBlenderData === true ||
											loadingManagerStore.isDeletingBlender === true
											? true : false                                                                
										} 
										onClick={refreshInstalledAddonList}
									/>
									<Button 
										kind="ghost"
										renderIcon={IntentRequestCreate}
										iconDescription="Install add-on"
										title="Install addon"
										hasIconOnly
										disabled={
											(
												loadingManagerStore.isInstallingExternalAddon === true ||
												loadingManagerStore.isSymlinkingExternalAddon === true ||
												loadingManagerStore.isUninstallingExternalAddon === true ||
												loadingManagerStore.isRegisteringInstalledBlenderData === true ||
												loadingManagerStore.isDeletingBlender === true
											) 
											// ||
											// (
											// 	installedBlenderManagerStore.installedBlenderDataTableRows.length === 0 &&
											// 	Object.keys(installedBlenderManagerStore.activeBlenderVersionObj).length === 0
											// ) 
											? true : false
										}
										onClick={installAddon}
									/>
									<Button 
										kind="ghost"
										renderIcon={IbmCloudDirectLink_1Connect}
										iconDescription="Symlink add-on"
										title="Symlink add-on"
										hasIconOnly
                                     	disabled={
											(
												loadingManagerStore.isInstallingExternalAddon === true ||
												loadingManagerStore.isSymlinkingExternalAddon === true ||
												loadingManagerStore.isUninstallingExternalAddon === true ||
												loadingManagerStore.isRegisteringInstalledBlenderData === true ||
												loadingManagerStore.isDeletingBlender === true
											) 
											// ||
											// (
											// 	installedBlenderManagerStore.installedBlenderDataTableRows.length === 0 &&
											// 	Object.keys(installedBlenderManagerStore.activeBlenderVersionObj).length === 0
											// ) 
											? true : false
										}
										onClick={symLinkAddon}
									/>
									<Dropdown
										size="lg"
										id="initialized-addon-selector"
										label="Filter"
										titleText=""
										// items={addonManagerStore.addonFilterArr}
										items={[]}
										// itemToString={(item: { text: any; }) => item ? item.text : ''}
										// initialSelectedItem={addonManagerStore.selectedAddonFilter}
										initialSelectedItem={{}}
										disabled={
											(
												loadingManagerStore.isRegisteringInstalledBlenderData === true || 
												loadingManagerStore.isDeletingBlender === true
											) ? true : false
										}
										onChange={changeAddonFilter}
									/>
                                </TableToolbarContent>
                            </TableToolbar>
                            <Table {...getTableProps()}>
							{/* 
							//TODO 
							If no addons found, add empty table and hide both the TableHead and TableBody.
							*/}
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
								<TableBody>
									<TableRow className='empty_installed_addon_table'>
										<TableCell key={1}>
											No addons found or no addons for this type found.
										</TableCell>
										{/* <TableCell key={2}>
										</TableCell>
										<TableCell key={3}>
										</TableCell>
										<TableCell key={4}>
										</TableCell>
										<TableCell key={5}>
										</TableCell> */}
									</TableRow>
								</TableBody>
                            </Table>
                        </TableContainer>
                    )}
                />
			</div>
		</div>
	)
}

export default AddonTable
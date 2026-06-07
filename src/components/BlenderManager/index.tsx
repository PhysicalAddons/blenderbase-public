import { Tab, TabList, TabPanel, TabPanels, Tabs } from '@carbon/react'
import InstallBlenderTable from './Installed'
import DownloadBlenderTable from './Download'
import { useUiControlsStore } from '../../store/uiControlsStore'
import { useBlenderManagerStore } from '../../store/blenderManagerStore';

const BlenderManager = () => {
	const { selectedLibrarySubTabIndex, setSelectedLibrarySubTabIndex } = useUiControlsStore()
	const { setInstalledBuilds } = useBlenderManagerStore()

	const onClikcInstalled = async () => {
		try {
			setSelectedLibrarySubTabIndex(0)
			setInstalledBuilds();
		} catch (e) {
			console.error(e);
		}
	}

	return (
		<div className="blender_manager">
			<Tabs defaultSelectedIndex={selectedLibrarySubTabIndex}>
				<TabList aria-label="BlenderManager Tabs" contained>
					<Tab
						title="Installed Blender versions"
						onClick={async () => await onClikcInstalled()}
					>
						Installed
					</Tab>
					<Tab
						title="Downloadable Blender versions"
						onClick={() => setSelectedLibrarySubTabIndex(1)}
					>
						Download
					</Tab>
				</TabList>
				<TabPanels>
					<TabPanel>
						<InstallBlenderTable />
					</TabPanel>
					<TabPanel>
						<DownloadBlenderTable />
					</TabPanel>
				</TabPanels>
			</Tabs>
		</div>
	)
}

export default BlenderManager
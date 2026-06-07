import { SidePanelClose } from '@carbon/icons-react';
import { Button, Tab, TabList, TabPanel, TabPanels, Tabs } from '@carbon/react';
import RecentFiles from '../../components/RecentFiles';
import LauncherBar from '../../components/LauncherBar/index';
import BlenderManager from '../../components/BlenderManager';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { useBlendFileStore } from '../../store/blendFileStore';

const Library = () => {
    const { isSidebarExpanded, selectedLibraryTabIndex, setIsSidebarExpanded, setSelectedLibraryTabIndex } = useUiControlsStore()
    const { setBlenderSeries } = useBlendFileStore()

    const ToggleRecentFilesPanel = async () => {
        try {
            setIsSidebarExpanded(!isSidebarExpanded)
			setBlenderSeries();
        } catch (e) {
            console.error(e);
        }
    }

    return (
        <>
            <div className={`library ${isSidebarExpanded ? '' : 'without_sidebar'}`}>
                <div className='sidebar_toggle__open'>
                    <Button
                        renderIcon={SidePanelClose} 
                        kind="ghost"
                        iconDescription="Open recent files"
                        title="Open recent files"
                        hasIconOnly 
                        onClick={ToggleRecentFilesPanel}
                    />
                </div>
                <div className={`library_content ${isSidebarExpanded ? '' : 'without_sidebar'}`}>
                    <Tabs defaultSelectedIndex={selectedLibraryTabIndex}>
                        <TabList aria-label="Tab navigation">
                            {/* <Tab 
                                title="Addons"
                                onClick={() => setSelectedLibraryTabIndex(0)}
                            >
                                Addons
                            </Tab> */}
                            <Tab 
                                title="Blenders"
                                onClick={() => setSelectedLibraryTabIndex(1)}
                            >
                                Blender
                            </Tab>
                        </TabList>
                        <div className='container'>
                            <hr></hr>
                        </div>
                        <TabPanels>
                            {/* <TabPanel>
                                <AddonTable />
                            </TabPanel> */}
                            <TabPanel>
                                <BlenderManager />
                            </TabPanel>
                        </TabPanels>
                    </Tabs>
                </div>
                <RecentFiles />
            </div>
            <LauncherBar />
        </>
    )
}

export default Library
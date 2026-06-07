import BlenderInstallationLocationTable from './BlenderInstallationLocationTable'
import Sections from './Sections'
import Footer from './Footer'

const SettingsManager = () => {
  return (
    <div className='settings_page'>
      {/* <div className='section_searchbar'>
        <SectionSearchbar />
      </div>
      <div className='sidebar_container'>
        <NavigationPanel />
      </div> */}
      <div className='content_container'>
        <h2>Settings</h2>
        <BlenderInstallationLocationTable />
        <Sections />
        <Footer />
      </div>
    </div>
  )
}

export default SettingsManager
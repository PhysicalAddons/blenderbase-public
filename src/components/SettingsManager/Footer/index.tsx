import { Tile } from "@carbon/react";

const Footer = () => {
    return (
        <div className='page_container'>
            <Tile 
                className='settings_section settings_subsection center_content' 
            >
                <h4 
                    className='settings_subsection_heading'                 
                >
                    Made by <a 
                        className='setting_description_hyperlink'
                        href="https://www.physicaladdons.com/psa/"
                        target="_blank" 
                        title="https://www.physicaladdons.com/psa/"
                        rel="noopener"
                    >
                        Physical Addons
                    </a>
                </h4> 

                <div 
                    className='setting_description_content'
                >
                    <a
                        className='setting_description_hyperlink'
                        href="https://discord.gg/4pseCn9pys" 
                        target="_blank" 
                        title="https://discord.gg/4pseCn9pys"
                        rel="noopener"
                    >
                        Get Help
                    </a>
                    |
                    <a 
                        className='setting_description_hyperlink'
                        href="https://github.com/PhysicalAddons/blenderbase-public/wiki" 
                        target="_blank" 
                        title="https://github.com/PhysicalAddons/blenderbase-public/wiki"
                        rel="noopener"
                    >
                        Documentation
                    </a>
                    |
                    <a 
                        className='setting_description_hyperlink'
                        href="https://github.com/PhysicalAddons/blenderbase-public/releases" 
                        target="_blank" 
                        title="https://github.com/PhysicalAddons/blenderbase-public/releases"
                        rel="noopener"
                    >
                        Releases
                    </a>
                </div>
            </Tile>
        </div>
    )
}

export default Footer
import { useEffect, useRef, useState } from 'react'
import { Button } from '@carbon/react';
import { Checkmark, ChevronDown, ChevronUp, Rocket } from '@carbon/react/icons';
import { useDisplayInformationStore } from '../../store/displayInformationStore';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { IBlenderVersion } from '../../models';
import { BlenderService } from '../../services/blenderService';
import { generateTitle } from '../../utility';

const blenderService = new BlenderService();

const LauncherBar = () => {
    const { appVersion } = useDisplayInformationStore()
    const [defaultBlenderVersion, setDefaultBlenderVersion] = useState<IBlenderVersion | null | undefined>(null);
    const { installedBuilds, setInstalledBuilds } = useBlenderManagerStore()

    const [dropdownOpen, setDropdownOpen] = useState<boolean>(false);
    const dropdownRef = useRef<HTMLDivElement>(null);

    const toggleDropdown = async () => {
        setInstalledBuilds();
        setDropdownOpen(!dropdownOpen);
    };

    const closeDropdown = () => {
        setDropdownOpen(false);
    };

    useEffect(() => {
        setDefaultBlenderVersion(installedBuilds.find((x) => x.is_default === true))
    }, [installedBuilds]);

    // useEffect(() => {
    //     async function init() {
    //         setDropdownOpen(false);
    //     }
    //     init();
    // }, [defaultBlenderVersion]);

    useEffect(() => {
        // Handles pressing ESC, which collapses the Blender version dropdown.
        const handleKeyDown = (e: any) => {
            if (e.key === 'Escape') {
                closeDropdown();
            }
        };
        // Handles clicking anywhere other than the dropdown, 
        // which collapses the Blender version dropdown.
        const handleClickOutside = (e: any) => {
            if (dropdownRef.current && !dropdownRef.current.contains(e.target)) {
                closeDropdown();
            }
        };
        document.addEventListener('keydown', handleKeyDown);
        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('keydown', handleKeyDown);
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, []);

    const setBlenderVersionAsDefault = async (id: string) => {
        try {
            await blenderService.setBlenderVersionAsDefault(id)
        } catch (e) {
            console.error(e);
        } finally {
            setInstalledBuilds();
            setDropdownOpen(false);
        }
    }

    const launchInstalledBlender = async (id: string | undefined) => {
        try {
            if (id === undefined) {
                throw new Error("No default Blender version set");
            }
            await blenderService.launchInstalledBlender(id)
        } catch (e) {
            console.error(e);
        } finally {
            setInstalledBuilds();
        }
    };

    return (
        <div className='launcher_bar'>
            <div
                className='launcher_bar__logo'
            >
                <a
                    className='launcher_bar__logo_text'
                    target='_blank'
                    title='Physical Addons: https://www.physicaladdons.com'
                    href="https://www.physicaladdons.com"
                >
                    <svg
                        className="library_logo"
                        id="Layer_1"
                        data-name="Layer 1"
                        xmlns="http://www.w3.org/2000/svg"
                        width="80"
                        height="12"
                        viewBox="0 0 80 12"
                    >
                        <path d="M9.38,11.01h1.8V.84h-1.8v10.16ZM79.31,7.13c0-.67-.09-1.25-.28-1.73-.19-.48-.44-.88-.76-1.2-.32-.31-.69-.55-1.11-.7-.42-.15-.86-.23-1.33-.23-.73,0-1.36.17-1.9.5-.54.33-.96.8-1.26,1.39-.3.59-.45,1.28-.45,2.07s.15,1.49.45,2.08.72,1.04,1.28,1.36,1.21.48,1.98.48c.59,0,1.12-.09,1.58-.27.46-.18.84-.44,1.13-.76.3-.33.5-.72.6-1.16l-1.68-.19c-.08.22-.2.4-.35.54-.16.15-.34.26-.55.33-.21.07-.45.11-.71.11-.39,0-.73-.08-1.02-.25-.29-.17-.52-.41-.68-.72-.16-.3-.23-.67-.24-1.09h5.31v-.55ZM74.02,6.47c.02-.3.09-.58.22-.83.15-.29.37-.52.65-.69.28-.18.6-.27.97-.27.34,0,.65.08.91.23s.46.37.61.64c.15.27.22.58.22.92h-3.57ZM18.23,4.21c-.32-.31-.69-.55-1.11-.7-.42-.15-.86-.23-1.33-.23-.73,0-1.36.17-1.9.5-.54.33-.96.8-1.26,1.39-.3.59-.45,1.28-.45,2.07s.15,1.49.45,2.08.72,1.04,1.28,1.36,1.21.48,1.98.48c.59,0,1.12-.09,1.58-.27.46-.18.84-.44,1.13-.76.3-.33.5-.72.6-1.16l-1.68-.19c-.08.22-.2.4-.35.54-.16.15-.34.26-.55.33-.21.07-.45.11-.71.11-.39,0-.73-.08-1.02-.25-.29-.17-.52-.41-.68-.72-.16-.3-.23-.67-.24-1.09h5.31v-.55c0-.67-.09-1.25-.28-1.73-.19-.48-.44-.88-.76-1.2ZM13.97,6.47c.02-.3.09-.58.22-.83.15-.29.37-.52.65-.69.28-.18.6-.27.97-.27.34,0,.65.08.91.23.26.16.46.37.61.64.15.27.22.58.22.92h-3.57ZM7.14,6.07c-.33-.2-.68-.31-1.04-.33v-.1c.33-.08.63-.21.9-.39.27-.18.48-.42.64-.71.16-.29.24-.65.24-1.06,0-.5-.12-.95-.36-1.35-.24-.4-.61-.71-1.1-.94-.49-.23-1.1-.34-1.83-.34H.69v10.16h4.11c.78,0,1.42-.12,1.94-.36s.9-.57,1.15-.99.38-.89.38-1.42-.11-.96-.32-1.32c-.21-.36-.48-.65-.82-.85ZM2.53,2.36h1.86c.54,0,.95.13,1.23.38s.41.58.41.97c0,.3-.07.55-.22.77-.15.22-.35.38-.6.5-.25.12-.54.18-.86.18h-1.82v-2.8ZM5.96,9.08c-.3.26-.78.38-1.45.38h-1.98v-2.98h2.03c.38,0,.71.07.98.21.27.14.49.33.64.58.15.24.22.52.22.82,0,.4-.15.73-.44.99ZM48.19,3.27c-.44,0-.82.12-1.16.36-.34.24-.58.58-.72,1.02h-.08v-1.27h-1.74v7.62h1.8v-4.48c0-.32.07-.61.22-.86s.35-.44.61-.58.55-.21.88-.21c.15,0,.31.01.47.03.16.02.28.05.36.07v-1.65c-.08-.02-.19-.03-.31-.04s-.24-.01-.34-.01ZM42.37,4.21c-.32-.31-.69-.55-1.11-.7-.42-.15-.86-.23-1.33-.23-.73,0-1.36.17-1.9.5-.54.33-.96.8-1.26,1.39-.3.59-.45,1.28-.45,2.07s.15,1.49.45,2.08c.3.59.72,1.04,1.28,1.36s1.21.48,1.98.48c.59,0,1.12-.09,1.58-.27.46-.18.84-.44,1.13-.76.3-.33.5-.72.6-1.16l-1.68-.19c-.08.22-.2.4-.35.54-.16.15-.34.26-.55.33-.21.07-.45.11-.71.11-.39,0-.73-.08-1.02-.25-.29-.17-.52-.41-.68-.72-.16-.3-.23-.67-.24-1.09h5.31v-.55c0-.67-.09-1.25-.28-1.73-.19-.48-.44-.88-.76-1.2ZM38.12,6.47c.02-.3.09-.58.22-.83.15-.29.37-.52.65-.69.28-.18.6-.27.97-.27.34,0,.65.08.91.23s.46.37.61.64c.15.27.22.58.22.92h-3.57ZM55.36,3.73c-.48-.3-1.01-.45-1.6-.45-.45,0-.81.08-1.1.23-.29.15-.52.33-.69.54-.17.21-.3.41-.39.59h-.07V.84h-1.8v10.16h1.77v-1.2h.1c.1.19.23.38.4.59.17.21.4.38.69.53s.65.22,1.09.22c.6,0,1.13-.15,1.61-.46s.85-.75,1.12-1.34c.27-.59.41-1.3.41-2.13s-.14-1.56-.42-2.15-.66-1.03-1.13-1.33ZM54.88,8.47c-.14.37-.34.67-.61.88s-.6.32-1,.32-.7-.1-.97-.31c-.27-.21-.47-.5-.61-.87s-.21-.8-.21-1.3.07-.92.21-1.29c.14-.36.34-.65.61-.85.27-.2.59-.3.98-.3s.73.1,1,.31c.27.21.47.5.61.86.14.37.2.79.2,1.26s-.07.9-.21,1.27ZM63.14,3.84c-.3-.2-.63-.34-1-.42-.36-.09-.73-.13-1.1-.13-.53,0-1.02.08-1.46.24s-.81.39-1.12.7c-.3.31-.52.7-.66,1.16l1.68.24c.09-.26.26-.49.52-.68s.6-.29,1.04-.29c.42,0,.74.1.96.31.22.21.33.49.33.87v.03c0,.17-.06.3-.19.38s-.33.14-.61.18c-.28.04-.64.08-1.09.13-.37.04-.73.1-1.07.19-.35.09-.66.22-.93.38-.27.17-.49.39-.65.67-.16.28-.24.64-.24,1.07,0,.5.11.92.33,1.26.22.34.53.6.91.77.39.17.82.26,1.3.26.4,0,.74-.06,1.04-.17.3-.11.54-.26.74-.44s.35-.38.46-.59h.06v1.05h1.73v-5.1c0-.51-.09-.93-.28-1.27s-.43-.6-.73-.8ZM62.35,8.27c0,.28-.07.54-.22.78s-.35.43-.61.57c-.26.14-.58.22-.94.22s-.68-.08-.92-.25c-.24-.17-.36-.42-.36-.75,0-.23.06-.42.18-.57.12-.15.29-.26.5-.34.21-.08.45-.14.72-.18.12-.02.26-.04.42-.06.16-.02.33-.05.49-.08.17-.03.32-.06.45-.1.13-.04.23-.08.29-.13v.9ZM25.82,3.62c-.39-.23-.85-.34-1.38-.34-.57,0-1.05.13-1.43.38-.39.25-.67.59-.84,1.02h-.09v-1.3h-1.72v7.62h1.8v-4.47c0-.37.07-.68.21-.94.14-.26.33-.46.57-.59s.52-.21.84-.21c.46,0,.83.14,1.09.43.26.29.39.69.39,1.2v4.58h1.8v-4.85c0-.61-.1-1.13-.32-1.55-.22-.43-.52-.75-.91-.98ZM69.43,6.66l-1.3-.28c-.39-.09-.66-.2-.83-.34-.17-.14-.25-.32-.25-.54,0-.26.12-.47.38-.63.25-.16.57-.24.94-.24.28,0,.51.04.7.13s.35.21.46.35c.11.15.19.3.24.47l1.64-.18c-.12-.65-.44-1.16-.96-1.54s-1.22-.57-2.11-.57c-.61,0-1.15.1-1.61.29-.46.19-.83.46-1.09.8s-.39.75-.38,1.22c0,.56.17,1.01.52,1.38s.89.62,1.62.77l1.3.27c.35.08.61.19.78.33s.25.32.25.54c0,.26-.13.47-.39.65-.26.18-.6.26-1.03.26s-.75-.09-1.01-.26c-.26-.18-.43-.44-.51-.78l-1.75.17c.11.7.45,1.25,1.02,1.64.57.39,1.32.59,2.25.59.64,0,1.2-.1,1.69-.31.49-.21.87-.49,1.15-.86.28-.37.42-.79.42-1.27,0-.55-.18-.99-.53-1.33-.35-.34-.89-.58-1.61-.74ZM33.5,4.64h-.07c-.09-.19-.22-.38-.39-.59-.17-.21-.4-.39-.69-.54-.29-.15-.65-.23-1.1-.23-.59,0-1.12.15-1.59.45s-.85.74-1.13,1.33c-.28.58-.42,1.3-.42,2.15s.14,1.55.41,2.13c.27.59.65,1.03,1.12,1.34.47.31,1.01.46,1.61.46.44,0,.8-.07,1.09-.22.29-.15.52-.32.69-.53.18-.21.31-.4.4-.59h.11v1.2h1.77V.84h-1.8v3.8ZM33.33,8.49c-.14.37-.34.66-.61.87-.27.21-.59.31-.97.31s-.73-.11-.99-.32c-.27-.21-.47-.51-.61-.88-.14-.37-.21-.8-.21-1.27s.07-.89.2-1.26.34-.66.61-.86.6-.31,1-.31.71.1.98.3c.27.2.47.48.61.85s.21.79.21,1.29-.07.92-.21,1.3Z" />
                    </svg>

                </a>

                <span className='app_version'>v{appVersion}</span>
            </div>
            <div className='launcher_bar__actions'>
                <div className='launcher_bar__dropdown' ref={dropdownRef}>
                    <div>
                        {defaultBlenderVersion
                            ? (
                                <Button
                                    title={generateTitle(defaultBlenderVersion)}
                                    onClick={toggleDropdown}
                                    renderIcon={dropdownOpen ? ChevronDown : ChevronUp}
                                    kind="ghost"
                                    size="xl"
                                    className={`launcher_bar__main_button ${dropdownOpen ? 'launcher_bar__main_button--active' : ''}`}
                                >
                                    <div className='launcher_bar__version'>
                                        {defaultBlenderVersion!.version || ""}
                                    </div>
                                    <div className='launcher_bar__meta'>
                                        <span className='launcher_bar__type'>
                                            {defaultBlenderVersion!.risk_id || ""}
                                        </span>
                                        {defaultBlenderVersion!.hash || ""}
                                    </div>
                                </Button>
                            ) : (
                                <div className='launcher_bar__no_versions'>
                                    <span>No Blender versions found</span>
                                </div>
                            )}
                    </div>

                    <div className={`launcher_bar__select ${dropdownOpen ? '' : 'hidden'}`}>
                        {installedBuilds.length !== 0 ? (
                            <>
                                {installedBuilds.slice().reverse().map((blenderVersion, index) => (
                                    <div key={index}>
                                        <Button
                                            title={generateTitle(blenderVersion)}
                                            renderIcon={defaultBlenderVersion?.id === blenderVersion.id ? Checkmark : undefined}
                                            className={defaultBlenderVersion?.id === blenderVersion.id ? 'launcher_bar__active' : ''}
                                            kind="ghost"
                                            size="xl"
                                            onClick={async () => { setBlenderVersionAsDefault(blenderVersion.id) }}
                                        >
                                            <div className='launcher_bar__version'>
                                                {blenderVersion.version || ""}
                                            </div>
                                            <div className='launcher_bar__meta'>
                                                <span className='launcher_bar__type'>
                                                    {blenderVersion.risk_id || ""}
                                                </span>
                                                {blenderVersion.hash || ""}
                                            </div>
                                        </Button>
                                    </div>
                                ))}
                            </>
                        ) : (
                            <div></div>
                        )}
                    </div>
                </div>
                <div
                    className='launcher_bar__button'
                >
                    <Button
                        title={
                            installedBuilds.length !== 0
                                ? installedBuilds.find((x) => x.is_default === true) !== null
                                    ? `Launch Blender ${installedBuilds.find((x) => x.is_default === true)?.version} ${installedBuilds.find((x) => x.is_default === true)?.risk_id}`
                                    : "No Blender versions"
                                : "No Blender versions"
                        }
                        renderIcon={Rocket}
                        kind="primary"
                        size="xl"
                        disabled={defaultBlenderVersion == null || defaultBlenderVersion == undefined}
                        onClick={async () => await launchInstalledBlender(defaultBlenderVersion?.id)}
                    >
                        Launch
                    </Button>
                </div>
            </div>
        </div>
    );
}

export default LauncherBar
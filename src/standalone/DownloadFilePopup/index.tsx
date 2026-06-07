import { invoke } from '@tauri-apps/api/core';
import { useEffect, useMemo, useState } from 'react'
// import { getCurrentWindow } from '@tauri-apps/api/window';
import { Dropdown } from '@carbon/react';
import { IBlenderInstallationLocation } from '../../models/index';

const DownloadFilePopup = () => {
    // const closeWindow = async () => {
    //     const appWindow = getCurrentWindow();
    //     appWindow.close();
    // };
    const [blenderInstallationLocations, setBlenderInstallationLocations] = useState<IBlenderInstallationLocation[]>([])
    const defaultInstallationLocation = useMemo(() => {
        const defaultEntry: IBlenderInstallationLocation | undefined = blenderInstallationLocations.find((e) => e.is_default = true)
        return defaultEntry !== undefined
            ? defaultEntry
            : blenderInstallationLocations.length > 0
                ? blenderInstallationLocations[0]
                : { directory_path: "No Blender installation locations found" } as IBlenderInstallationLocation
    }, [blenderInstallationLocations])

    useEffect(() => {
        fetchBlenderInstallationPaths();
    }, []);

    const fetchBlenderInstallationPaths = async () => {
        try {
            const paths: IBlenderInstallationLocation[] = await invoke("cmd_fetch_blender_installation_locations", {
                id: null,
                limit: null,
                directoryPath: null,
                isDefault: null
            });
            setBlenderInstallationLocations(paths);
        } catch (e) {
            setBlenderInstallationLocations([]);
            console.error(e);
        }
    };
    return (
        <>
                <Dropdown
                    id="default"
                    itemToString={(item) => item?.directory_path || ''}
                    items={blenderInstallationLocations}
                    initialSelectedItem={blenderInstallationLocations.find(e => e.id === defaultInstallationLocation.id)}
                    label={defaultInstallationLocation.directory_path}
                    titleText="Blender installation locations"
                    type="default"
                />
            {/* <button
                className="mb-2"
                onClick={handleUseDefault}
            >
                Use Default Directory {repoPaths?.find((e) => e?.is_default === true)?.repo_directory_path ? repoPaths?.find((e) => e?.is_default === true)?.repo_directory_path : repoPaths[0]?.repo_directory_path}
            </button> */}
            {/* <ul>
                {blenderInstallationLocations.map((blenderInstallationLocation) => (
                    <li key={blenderInstallationLocation.id}>
                        <button 
                            onClick={() => handleSelect(path.repo_directory_path)}
                        >
                            {blenderInstallationLocation.directory_path}
                        </button>
                    </li>
                ))}
            </ul> */}
            <button
                disabled={blenderInstallationLocations.length == 0}
            >
                Ok
            </button>
            <button
            >
                Cancel
            </button>
        </>
    );
}

export default DownloadFilePopup
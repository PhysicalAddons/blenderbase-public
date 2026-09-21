import { Button } from '@carbon/react';
import { Add, FolderOpen } from '@carbon/react/icons';
import { open } from '@tauri-apps/plugin-dialog';
import BlenderLogo from '../BlenderLogo';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { useSetupRestoreStore } from '../../store/setupRestoreStore';
import { postStatusError } from '../../store/statusStore';
import { SETUP_FILE_FILTER } from '../../constants';

/**
 * Shown in the middle column when no Blender version is installed yet.
 */
const EmptyState = () => {
    const setIsInstallBlenderOpen = useUiControlsStore((s) => s.setIsInstallBlenderOpen)
    const openSetup = useSetupRestoreStore((s) => s.open)

    // A new computer usually starts here: a setup file saved on the old one brings the
    // Blender versions to install and, once they are, the preferences, theme and keymaps.
    const pickSetupFile = async () => {
        try {
            const selected = await open({ multiple: false, directory: false, title: "Open a setup file", filters: SETUP_FILE_FILTER });
            if (typeof selected === "string" && selected.length > 0) {
                await openSetup(selected);
            }
        } catch (e) {
            console.error(e);
            postStatusError(`Opening the setup file failed: ${e instanceof Error ? e.message : String(e)}`);
        }
    };

    return (
        <div className='empty_state'>
            <BlenderLogo className='empty_state__mark' />
            <h3 className='empty_state__title'>No Blender installed</h3>
            <p className='empty_state__text'>
                Install a version to manage its addons and open your recent files with it.
            </p>
            <p className='empty_state__text'>
                Blender already installed somewhere else? Add its folder under Settings › Locations.
            </p>
            <Button
                kind="primary"
                size="lg"
                className='install_button empty_state__button'
                title="Install a Blender version"
                onClick={() => setIsInstallBlenderOpen(true)}
            >
                <Add /> Install Blender
            </Button>
            <Button
                kind="secondary"
                size="lg"
                className='install_button empty_state__button empty_state__button--secondary'
                title="Restore a setup saved on another computer"
                onClick={() => void pickSetupFile()}
            >
                <FolderOpen /> Restore from a setup file
            </Button>
        </div>
    )
}

export default EmptyState

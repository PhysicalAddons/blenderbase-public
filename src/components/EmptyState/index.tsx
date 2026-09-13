import { Button } from '@carbon/react';
import { Add } from '@carbon/react/icons';
import BlenderLogo from '../BlenderLogo';
import { useUiControlsStore } from '../../store/uiControlsStore';

/**
 * Shown in the middle column when no Blender version is installed yet.
 */
const EmptyState = () => {
    const setIsInstallBlenderOpen = useUiControlsStore((s) => s.setIsInstallBlenderOpen)

    return (
        <div className='empty_state'>
            <BlenderLogo className='empty_state__mark' />
            <h3 className='empty_state__title'>No Blender installed</h3>
            <p className='empty_state__text'>
                Install a version to manage its addons and open your recent files with it.
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
        </div>
    )
}

export default EmptyState

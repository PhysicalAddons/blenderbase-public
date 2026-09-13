import { Button } from '@carbon/react'
import { SidePanelOpen } from '@carbon/react/icons'

type Props = {
    isSidebarExpanded: boolean, 
    setIsSidebarExpanded: (isExpanded: boolean) => void | Promise<void>
}

const ButtonSideBarToggle = (props: Props) => {
    return (
        <div className='sidebar_toggle__close'>
            <Button
                renderIcon={SidePanelOpen}
                kind="ghost"
                iconDescription="Close recent files"
                title="Close recent files"
                hasIconOnly
                onClick={() => props.setIsSidebarExpanded(!props.isSidebarExpanded)}
            />
        </div>
    )
}

export default ButtonSideBarToggle
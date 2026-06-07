import { Close } from '@carbon/react/icons'

const StandaloneWindowControls = () => {
    return (
        <>
            <div
                className="windows_app_window_control"
            >
                <div
                    id="close-win"
                    className="window_control_options close-win"
                >
                    <Close />
                </div>
            </div>
        </>
    )
}

export default StandaloneWindowControls
import { IAppSetting } from '../../../../models'

interface Props {
    appSetting: IAppSetting,
    hideLabel: boolean,
    handleSetting: any
}

const InputButtonSettingControl = (props: Props) => {
    return (
        <div
            className={`settings_subsection_row ${props.appSetting.is_enabled ? 'enabled' : 'disabled'}`}
        >
            <div
                className="setting_row"
            >
                <div
                    className="setting_title"
                >
                    {props.appSetting.name}
                </div>
                <div
                    className="setting_toggle"
                    title={props.appSetting.name}
                >
                    <button
                        id={props.appSetting.code}
                        className='setting_button'
                        title={props.appSetting.control_title!}
                        onClick={() => props.handleSetting(props.appSetting)}
                    >
                        {props.appSetting.control_title}
                    </button>
                </div>
            </div>
            <div
                className="setting_description"
            >
                {props.appSetting.description}
            </div>
        </div>
    )
}

export default InputButtonSettingControl



{/* 
                 async () => {
                            await loadingManagerStore.setIsCheckingForBlenderbaseVersionUpdate(true);
                            await checkForVersionUpdate();
                            await loadingManagerStore.setIsCheckingForBlenderbaseVersionUpdate(false);
                        }
                {loadingManagerStore.isCheckingForBlenderbaseVersionUpdate ? (
                        <InlineLoading
                            className='setting_button_inline_loading'
                            status="active"
                            iconDescription="Checking for Blenderbase version update"
                        />
                    ) : (
                    <button
                        id={"check_for_blenderbase_update_manually"}
                        className='setting_button'
                        onClick={
                            async () => {
                                await loadingManagerStore.setIsCheckingForBlenderbaseVersionUpdate(true);
                                await checkForVersionUpdate();
                                await loadingManagerStore.setIsCheckingForBlenderbaseVersionUpdate(false);
                            }
                        }
                    >
                        Check
                    </button>
                )} */}
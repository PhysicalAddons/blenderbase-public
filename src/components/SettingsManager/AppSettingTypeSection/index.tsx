import { Tile } from '@carbon/react'
import { useEffect, useState } from 'react'
import { IAppSetting, IAppSettingType } from '../../../models'
import InputButtonSettingControl from '../Actions/InputButtonSettingControl'
import InputToggleSettingControl from '../Actions/InputToggleSettingControl'
import { AppSettingActionKind } from '../../../enums'
import { SettingsService } from '../../../services/settingsService'
import NotImplementedSettingControl from '../Actions/NotImplementedSettingControl'
import InputRangeSettingControl from '../Actions/InputRangeSettingControl'
import InputDecimalSettingControl from '../Actions/InputDecimalSettingControl'

type Props = {
    appSettingType: IAppSettingType,
}

const settingsService = new SettingsService();

const AppSettingTypeSection = (props: Props) => {
    const [appSettings, setAppSettings] = useState<IAppSetting[]>([])
    useEffect(() => {
        async function init() {
            setAppSettings(await settingsService.fetchAppSetting(null, null, null, null, props.appSettingType.id));
        }
        init();
    }, []);
    const handleSetting = async (appSetting: IAppSetting): Promise<string | undefined> => {
        let err = undefined;
        try {
            err = await settingsService.handleSetting(appSetting)
        } catch (e: any) {
            console.error(e);
            return err + " " + e?.message || String(e);
        } finally {
            setAppSettings(await settingsService.fetchAppSetting(null, null, null, null, props.appSettingType.id));
            return err;
        }
    };
    return (
        <>
            {appSettings.length > 0 ? (
                <Tile
                    key={""}
                    className="settings_section settings_subsection"
                >
                    <h4
                        className='settings_subsection_heading'
                    >
                        {props.appSettingType.name}
                    </h4>
                    {appSettings.map((entry: IAppSetting, index: number) => (
                        <div key={entry.id}>
                            {entry.app_setting_action_type_id === AppSettingActionKind.INPUT_BUTTON ? (
                                <>
                                    <InputButtonSettingControl appSetting={entry} hideLabel={true} handleSetting={handleSetting} />
                                    {index === appSettings.length - 1 ? (
                                        <></>
                                    ) : (
                                        <hr className="divider" />
                                    )}
                                </>
                            ) : entry.app_setting_action_type_id === AppSettingActionKind.INPUT_TOGGLE ? (
                                <>
                                    <InputToggleSettingControl appSetting={entry} hideLabel={true} handleSetting={handleSetting} />
                                    {index === appSettings.length - 1 ? (
                                        <></>
                                    ) : (
                                        <hr className="divider" />
                                    )}
                                </>
                            ) : entry.app_setting_action_type_id === AppSettingActionKind.INPUT_DECIMAL ? (
                                <>
                                    <InputDecimalSettingControl 
                                        appSetting={entry} 
                                        hideLabel={true} 
                                        handleSetting={handleSetting}
                                    />
                                    {index === appSettings.length - 1 ? (
                                        <></>
                                    ) : (
                                        <hr className="divider" />
                                    )}
                                </>
                            ) : entry.app_setting_action_type_id === AppSettingActionKind.INPUT_RANGE ? (
                                <>
                                    <InputRangeSettingControl 
                                        appSetting={entry} 
                                        hideLabel={true} 
                                        handleSetting={handleSetting}
                                    />
                                    {index === appSettings.length - 1 ? (
                                        <></>
                                    ) : (
                                        <hr className="divider" />
                                    )}
                                </>
                            ) : (
                                <>
                                    <NotImplementedSettingControl appSetting={entry} />
                                    {index === appSettings.length - 1 ? (
                                        <></>
                                    ) : (
                                        <hr className="divider" />
                                    )}
                                </>
                            )}
                        </div>
                    ))}
                </Tile>
            ) : (
                <></>
            )}
        </>
    )
}

export default AppSettingTypeSection
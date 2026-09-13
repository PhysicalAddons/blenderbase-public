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
import { postStatusError } from '../../../store/statusStore'

type Props = {
    appSettingType: IAppSettingType,
}

const settingsService = new SettingsService();

const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e));

const AppSettingTypeSection = (props: Props) => {
    const [appSettings, setAppSettings] = useState<IAppSetting[]>([])

    useEffect(() => {
        let cancelled = false;
        settingsService.fetchAppSetting(null, null, null, null, props.appSettingType.id)
            .then((settings) => {
                if (!cancelled) {
                    setAppSettings(settings);
                }
            })
            .catch((e) => {
                console.error(e);
                postStatusError(`Loading settings failed: ${errorText(e)}`);
            });
        return () => {
            cancelled = true;
        };
    }, [props.appSettingType.id]);

    /** Applies a setting and reloads the section; resolves with the error message if it was refused. */
    const handleSetting = async (appSetting: IAppSetting): Promise<string | undefined> => {
        let err: string | undefined;
        try {
            await settingsService.handleSetting(appSetting);
        } catch (e) {
            console.error(e);
            err = errorText(e);
        }
        try {
            setAppSettings(await settingsService.fetchAppSetting(null, null, null, null, props.appSettingType.id));
        } catch (e) {
            console.error(e);
            postStatusError(`Reloading settings failed: ${errorText(e)}`);
        }
        return err;
    };

    const controlFor = (entry: IAppSetting) => {
        switch (entry.app_setting_action_type_id) {
            case AppSettingActionKind.INPUT_BUTTON:
                return <InputButtonSettingControl appSetting={entry} hideLabel={true} handleSetting={handleSetting} />;
            case AppSettingActionKind.INPUT_TOGGLE:
                return <InputToggleSettingControl appSetting={entry} hideLabel={true} handleSetting={handleSetting} />;
            case AppSettingActionKind.INPUT_DECIMAL:
                return <InputDecimalSettingControl appSetting={entry} hideLabel={true} handleSetting={handleSetting} />;
            case AppSettingActionKind.INPUT_RANGE:
                return <InputRangeSettingControl appSetting={entry} hideLabel={true} handleSetting={handleSetting} />;
            default:
                return <NotImplementedSettingControl appSetting={entry} />;
        }
    };

    if (appSettings.length === 0) {
        return null;
    }
    return (
        <Tile className="settings_section settings_subsection">
            <h4 className='settings_subsection_heading'>
                {props.appSettingType.name}
            </h4>
            {appSettings.map((entry: IAppSetting, index: number) => (
                <div key={entry.id}>
                    {controlFor(entry)}
                    {index < appSettings.length - 1 && <hr className="divider" />}
                </div>
            ))}
        </Tile>
    )
}

export default AppSettingTypeSection

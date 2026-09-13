import { useEffect, useState } from 'react';
import { IAppSetting, SettingHandler } from '../../../../models';
import { MeasurementUnitKindName } from '../../../../enums/helpers';
import { MeasurementUnitKind } from '../../../../enums';

interface Props {
    appSetting: IAppSetting;
    hideLabel: boolean;
    handleSetting: SettingHandler;
}

/**
 * A number setting. Typing edits a local draft; the value is saved on blur or Enter.
 */
const InputDecimalSettingControl = (props: Props) => {
    const [validationError, setValidationError] = useState<string | null>(null);
    const [draft, setDraft] = useState<string>(String(props.appSetting.int_value ?? 0));

    // Follow the stored value when it changes elsewhere (e.g. after a refetch).
    useEffect(() => {
        setDraft(String(props.appSetting.int_value ?? 0));
    }, [props.appSetting.int_value]);

    const commit = async () => {
        const value = Number(draft);
        if (draft.trim() === "" || Number.isNaN(value)) {
            setValidationError("Enter a number");
            return;
        }
        if (value === (props.appSetting.int_value ?? 0)) {
            setValidationError(null);
            return;
        }
        const min = props.appSetting.min_int_value;
        const max = props.appSetting.max_int_value;
        if (min !== null && value < min) {
            setValidationError(`Min allowed value is ${min}`);
            return;
        }
        if (max !== null && value > max) {
            setValidationError(`Max allowed value is ${max}`);
            return;
        }
        try {
            const err = await props.handleSetting({ ...props.appSetting, int_value: value });
            setValidationError(err ?? null);
        } catch (e) {
            console.error(e);
            setValidationError(e instanceof Error ? e.message : String(e));
        }
    };

    return (
        <div
            className={`settings_subsection_row ${props.appSetting.is_enabled ? 'enabled' : 'disabled'}`}
        >
            <div className="setting_row">
                <div className="setting_title">
                    {props.appSetting.name} ({MeasurementUnitKindName[props.appSetting.measurement_unit_type_id as MeasurementUnitKind]})
                </div>
                <div
                    className="sub_setting_decimal"
                    title={props.appSetting.name}
                >
                    <input
                        className="setting_decimal_input"
                        type="number"
                        id={`${props.appSetting.code}-num-1`}
                        min={props.appSetting.min_int_value!}
                        max={props.appSetting.max_int_value!}
                        placeholder="From"
                        disabled={!props.appSetting.is_enabled}
                        value={draft}
                        onChange={(e) => setDraft(e.target.value)}
                        onBlur={commit}
                        onKeyDown={(e) => {
                            if (e.key === "Enter") {
                                e.currentTarget.blur();
                            }
                        }}
                    />
                </div>
            </div>

            <div className="setting_validation_error">
                {validationError}
            </div>

            <div className="setting_description">
                {props.appSetting.description}
            </div>
        </div>
    );
};

export default InputDecimalSettingControl;

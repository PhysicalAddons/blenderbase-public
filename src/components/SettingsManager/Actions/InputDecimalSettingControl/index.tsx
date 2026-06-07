import { useState } from 'react';
import { IAppSetting } from '../../../../models';
import { MeasurementUnitKindName } from '../../../../enums/helpers';
import { MeasurementUnitKind } from '../../../../enums';

interface Props {
    appSetting: IAppSetting;
    hideLabel: boolean;
    handleSetting: any;
}

const InputDecimalSettingControl = (props: Props) => {
    const [validationError, setValidationError] = useState<string | null>(null);
    const innerHandleSetting = async (rawValue?: number) => {
        try {
            let newAppSetting = { ...props.appSetting };
            const value = rawValue ?? newAppSetting.int_value ?? 0;
            const min = newAppSetting.min_int_value;
            const max = newAppSetting.max_int_value;
            if (min !== null && value < min) {
                const err = `Min allowed value is ${min}`;
                setValidationError(err);
                return err;
            }
            if (max !== null && value > max) {
                const err = `Max allowed value is ${max}`;
                setValidationError(err);
                return err;
            }
            newAppSetting.int_value = value;
            const err = await props.handleSetting(newAppSetting);
            setValidationError(err ?? null);
            return err ?? null;
        } catch (e: any) {
            console.error(e);
            const msg = e?.message || String(e);
            setValidationError(msg);
            return msg;
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
                        value={props.appSetting.int_value ?? 0}
                        onChange={(e) => {
                            const value = Number(e.target.value);
                            innerHandleSetting(value);
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
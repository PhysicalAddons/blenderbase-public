import { IAppSetting, IRangeValues } from '../../../../models'
import { InputValueKind } from '../../../../enums'
import { useEffect, useRef, useState } from 'react'

interface Props {
    appSetting: IAppSetting,
    hideLabel: boolean,
    handleSetting: any,
}

const InputRangeSettingControl = (props: Props) => {
    const isInitialized = useRef(false);
    const [validationError, setValidationError] = useState<string | null>(null);
    const [rangeValues, setRangeValues] = useState<IRangeValues>({
        range_int_value_from:
            props.appSetting.range_int_value_from ??
            props.appSetting.default_range_int_value_from,

        range_int_value_to:
            props.appSetting.range_int_value_to ??
            props.appSetting.default_range_int_value_to,

        range_text_value_from:
            props.appSetting.range_text_value_from ??
            props.appSetting.default_range_text_value_from,

        range_text_value_to:
            props.appSetting.range_text_value_to ??
            props.appSetting.default_range_text_value_to,
    } as IRangeValues);
    useEffect(() => {
        async function run() {
            await innerHandleSetting();
        }
        if (isInitialized.current === false) {
            isInitialized.current = true;
            return;
        }
        run();
    }, [rangeValues]);
    const innerHandleSetting = async () => {
        try {
            let newAppSetting = { ...props.appSetting };
            switch (props.appSetting.input_value_type_id) {
                case InputValueKind.INTEGER:
                    newAppSetting.range_int_value_from = rangeValues.range_int_value_from;
                    newAppSetting.range_int_value_to = rangeValues.range_int_value_to;
                    break;
                case InputValueKind.STRING:
                    newAppSetting.range_text_value_from = rangeValues.range_text_value_from;
                    newAppSetting.range_text_value_to = rangeValues.range_text_value_to;
                    break;
                default:
                    console.warn("Not implement range value input");
            }
            let err = await props.handleSetting(newAppSetting);
            setValidationError(err ?? null);
        } catch (e: any) {
            console.error(e);
            setValidationError(e?.message || String(e));
        }
    };
    return (
        <div
            className={`settings_subsection_row ${props.appSetting.is_enabled ? 'enabled' : 'disabled'}`}
        >
            <div className="setting_row">
                <div className="setting_title">
                    {props.appSetting.name}
                </div>

                <div
                    className="setting_range"
                    title={props.appSetting.name}
                >
                    {props.appSetting.input_value_type_id === InputValueKind.INTEGER ? (
                        <>
                            <div className='setting_range_from'>
                                <input
                                    className='setting_range_from_input'
                                    type="number"
                                    id={`${props.appSetting.code}-num-1`}
                                    min={props.appSetting.min_range_int_value_from!}
                                    max={props.appSetting.max_range_int_value_from!}
                                    placeholder="From"
                                    disabled={!props.appSetting.is_enabled}
                                    value={rangeValues.range_int_value_from ?? ''}
                                    onChange={(e) =>
                                        setRangeValues((prev) => ({
                                            ...prev,
                                            range_int_value_from: Number(e.target.value),
                                        }))
                                    }
                                />
                            </div>
                            <div className='setting_range_to'>
                                <input
                                    className='setting_range_to_input'
                                    type="number"
                                    id={`${props.appSetting.code}-num-2`}
                                    min={props.appSetting.min_range_int_value_to!}
                                    max={props.appSetting.max_range_int_value_to!}
                                    placeholder="To"
                                    disabled={!props.appSetting.is_enabled}
                                    value={rangeValues.range_int_value_to ?? ''}
                                    onChange={(e) =>
                                        setRangeValues((prev) => ({
                                            ...prev,
                                            range_int_value_to: Number(e.target.value),
                                        }))
                                    }
                                />
                            </div>
                        </>
                    ) : props.appSetting.input_value_type_id === InputValueKind.STRING ? (
                        <>
                            <div className='setting_range_from' >
                                <input
                                    className='setting_range_from_input'
                                    type="text"
                                    id={`${props.appSetting.code}-text-1`}
                                    placeholder="From"
                                    disabled={!props.appSetting.is_enabled}
                                    value={rangeValues.range_text_value_from ?? ''}
                                    onChange={(e) =>
                                        setRangeValues((prev) => ({
                                            ...prev,
                                            range_text_value_from: e.target.value,
                                        }))
                                    }
                                />
                            </div>
                            <div className='setting_range_to' >
                                <input
                                    className='setting_range_to_input'
                                    type="text"
                                    id={`${props.appSetting.code}-text-2`}
                                    placeholder="To"
                                    disabled={!props.appSetting.is_enabled}
                                    value={rangeValues.range_text_value_to ?? ''}
                                    onChange={(e) =>
                                        setRangeValues((prev) => ({
                                            ...prev,
                                            range_text_value_to: e.target.value,
                                        }))
                                    }
                                />
                            </div>
                        </>
                    ) : (
                        <>Not implement range value input</>
                    )}
                </div>
            </div>
            <div className="setting_validation_error">
                {validationError}
            </div>
            <div className="setting_description">
                {props.appSetting.description}
            </div>
        </div>
    )
}

export default InputRangeSettingControl
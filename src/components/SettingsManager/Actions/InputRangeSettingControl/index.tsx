import { IAppSetting, IRangeValues, SettingHandler } from '../../../../models'
import { InputValueKind } from '../../../../enums'
import { useEffect, useState } from 'react'

interface Props {
    appSetting: IAppSetting,
    hideLabel: boolean,
    handleSetting: SettingHandler,
}

const rangeFromSetting = (s: IAppSetting): IRangeValues => ({
    range_int_value_from: s.range_int_value_from ?? s.default_range_int_value_from,
    range_int_value_to: s.range_int_value_to ?? s.default_range_int_value_to,
    range_text_value_from: s.range_text_value_from ?? s.default_range_text_value_from,
    range_text_value_to: s.range_text_value_to ?? s.default_range_text_value_to,
});

const isSameRange = (a: IRangeValues, b: IRangeValues): boolean =>
    a.range_int_value_from === b.range_int_value_from
    && a.range_int_value_to === b.range_int_value_to
    && a.range_text_value_from === b.range_text_value_from
    && a.range_text_value_to === b.range_text_value_to;

/**
 * A from/to setting. Typing edits a local draft; the range is saved on blur or Enter.
 */
const InputRangeSettingControl = (props: Props) => {
    const [validationError, setValidationError] = useState<string | null>(null);
    const [rangeValues, setRangeValues] = useState<IRangeValues>(() => rangeFromSetting(props.appSetting));

    // Follow the stored values when they change elsewhere (e.g. after a refetch).
    useEffect(() => {
        setRangeValues(rangeFromSetting(props.appSetting));
    }, [
        props.appSetting.range_int_value_from,
        props.appSetting.range_int_value_to,
        props.appSetting.range_text_value_from,
        props.appSetting.range_text_value_to,
    ]);

    const commit = async () => {
        if (isSameRange(rangeValues, rangeFromSetting(props.appSetting))) {
            return;
        }
        const newAppSetting = { ...props.appSetting };
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
                return;
        }
        try {
            const err = await props.handleSetting(newAppSetting);
            setValidationError(err ?? null);
        } catch (e) {
            console.error(e);
            setValidationError(e instanceof Error ? e.message : String(e));
        }
    };

    const blurOnEnter = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Enter") {
            e.currentTarget.blur();
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
                                    onBlur={commit}
                                    onKeyDown={blurOnEnter}
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
                                    onBlur={commit}
                                    onKeyDown={blurOnEnter}
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
                                    onBlur={commit}
                                    onKeyDown={blurOnEnter}
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
                                    onBlur={commit}
                                    onKeyDown={blurOnEnter}
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

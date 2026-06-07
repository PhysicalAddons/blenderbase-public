import { Toggle } from '@carbon/react'
import { IAppSetting } from '../../../../models'

interface Props {
	appSetting: IAppSetting,
	hideLabel: boolean,
	handleSetting: any
}

const InputToggleSettingControl = (props: Props) => {
	const innerhandleSetting = async () => {
		try {
			let newAppSetting = { ...props.appSetting };
			newAppSetting.int_value = +!newAppSetting.int_value;
			await props.handleSetting(newAppSetting)
		} catch (e) {
			console.error(e);
		}
	};
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
					<Toggle
						id={props.appSetting.code}
						hideLabel={props.hideLabel}
						toggled={Boolean(props.appSetting.int_value)}
						title={props.appSetting.control_title!}
						disabled={!props.appSetting.is_enabled}
						onClick={() => innerhandleSetting()}
					>
					</Toggle>
				</div>
			</div>
			<div className="setting_description">
				{props.appSetting.description}
			</div>
		</div>
	)
}

export default InputToggleSettingControl
import { useEffect, useState } from 'react'
import { IAppSettingType } from '../../../models'
import AppSettingTypeSection from '../AppSettingTypeSection'
import { SettingsService } from '../../../services/settingsService';
import { postStatusError } from '../../../store/statusStore';


const settingsService = new SettingsService();

const Sections = () => {
	const [appSettingsTypes, setAppSettingsTypes] = useState<IAppSettingType[]>([])

	useEffect(() => {
		let cancelled = false;
		settingsService.fetchAppSettingType(null, null, null)
			.then((types) => {
				if (!cancelled) {
					setAppSettingsTypes(types);
				}
			})
			.catch((e) => {
				console.error(e);
				postStatusError(`Loading settings failed: ${e}`);
			});
		return () => {
			cancelled = true;
		};
	}, [])

	return (
		<>
			{appSettingsTypes.map((entry: IAppSettingType) => (
				<AppSettingTypeSection key={entry.id} appSettingType={entry} />
			))}
		</>
	)
}

export default Sections
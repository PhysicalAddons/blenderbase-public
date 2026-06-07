import { useEffect, useState } from 'react'
import { IAppSettingType } from '../../../models'
import AppSettingTypeSection from '../AppSettingTypeSection'
import { SettingsService } from '../../../services/settingsService';


const settingsService = new SettingsService();

const Sections = () => {
	const [appSettingsTypes, setAppSettingsTypes] = useState<IAppSettingType[]>([])

	useEffect(() => {
        async function init() {
            setAppSettingsTypes(await settingsService.fetchAppSettingType(null, null, null));
        }
        init();
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
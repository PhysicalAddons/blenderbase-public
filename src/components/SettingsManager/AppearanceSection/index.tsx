import { RadioButton, RadioButtonGroup, Tile } from '@carbon/react'
import { ThemePreference, useThemeStore } from '../../../store/themeStore'

const OPTIONS: { value: ThemePreference, label: string, description: string }[] = [
    { value: 'dark', label: 'Dark', description: 'Always dark.' },
    { value: 'light', label: 'Light', description: 'Always light.' },
    { value: 'auto', label: 'Auto', description: 'Follows the Windows theme.' },
]

const AppearanceSection = () => {
    const { preference, setPreference } = useThemeStore()
    const current = OPTIONS.find((o) => o.value === preference) ?? OPTIONS[0]

    return (
        <Tile className="settings_section settings_subsection appearance_section">
            <h4 className='settings_subsection_heading'>Appearance</h4>
            <RadioButtonGroup
                legendText="Theme"
                name="appearance-theme"
                orientation="horizontal"
                valueSelected={preference}
                onChange={(value: string | number | undefined) => {
                    if (value === 'dark' || value === 'light' || value === 'auto') {
                        setPreference(value);
                    }
                }}
            >
                {OPTIONS.map((o) => (
                    <RadioButton key={o.value} id={`theme-${o.value}`} labelText={o.label} value={o.value} />
                ))}
            </RadioButtonGroup>
            <div className='sub_setting_description'>{current.description}</div>
        </Tile>
    )
}

export default AppearanceSection

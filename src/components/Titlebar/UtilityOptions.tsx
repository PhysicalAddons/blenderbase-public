import { NavLink } from 'react-router-dom';
import { Home, LogoDiscord, Settings } from '@carbon/react/icons';
import { useShallow } from 'zustand/react/shallow';
import { COLON_DELIMITER, DISCORD_COM_INVITE, JOIN_THE_COMMUNITY_SENTANCE_CASE } from '../../constants';
import { useUiControlsStore } from '../../store/uiControlsStore';

const UtilityOptions = () => {
	const { isSettingsOpen, setIsSettingsOpen } = useUiControlsStore(
		useShallow((s) => ({ isSettingsOpen: s.isSettingsOpen, setIsSettingsOpen: s.setIsSettingsOpen }))
	)
	// Settings is a panel in the middle column, not a route: the Home tab reads as active
	// only while the panel is closed, and the gear takes the active look while it is open.
	const homeClassName = ({ isActive }: { isActive: boolean }) =>
		`navigation_bar_utilities_option navigation_bar_tab${isActive && !isSettingsOpen ? " active" : ""}`;

	return (
		<div
			// Meant for buttons, that are not window controllers or navigation links.
			className="navigation_bar_utilities"
		>
			<NavLink
				className={homeClassName}
				title="Home"
				to="/"
				end
				onClick={() => setIsSettingsOpen(false)}
			>
				<Home/>
			</NavLink>
			<button
				type="button"
				className={`navigation_bar_utilities_option navigation_bar_tab${isSettingsOpen ? " active" : ""}`}
				title={isSettingsOpen ? "Close settings" : "Settings"}
				aria-label={isSettingsOpen ? "Close settings" : "Settings"}
				aria-pressed={isSettingsOpen}
				onClick={() => setIsSettingsOpen(!isSettingsOpen)}
			>
				<Settings/>
			</button>
			<a
				className="navigation_bar_utilities_option get_help_navbar"
				title={`${JOIN_THE_COMMUNITY_SENTANCE_CASE}${COLON_DELIMITER}${DISCORD_COM_INVITE}`}
				target='_blank'
				href={`${DISCORD_COM_INVITE}`}
				rel="noopener"

			>
				<LogoDiscord/>
			</a>

		</div>
	)
}

export default UtilityOptions

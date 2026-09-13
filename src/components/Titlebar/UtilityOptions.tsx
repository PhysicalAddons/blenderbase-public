import { NavLink } from 'react-router-dom';
import { Home, LogoDiscord, Settings } from '@carbon/react/icons';
import { COLON_DELIMITER, DISCORD_COM_INVITE, JOIN_THE_COMMUNITY_SENTANCE_CASE } from '../../constants';

const UtilityOptions = () => {
	const tabClassName = ({ isActive }: { isActive: boolean }) =>
		`navigation_bar_utilities_option navigation_bar_tab${isActive ? " active" : ""}`;

	return (
		<div
			// Meant for buttons, that are not window controllers or navigation links.
			className="navigation_bar_utilities"
		>
			<NavLink
				className={tabClassName}
				title="Home"
				to="/"
				end
			>
				<Home/>
			</NavLink>
			<NavLink
				className={tabClassName}
				title="Settings"
				to="/settings"
			>
				<Settings/>
			</NavLink>
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

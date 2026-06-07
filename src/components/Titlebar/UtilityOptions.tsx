import { LogoDiscord } from '@carbon/react/icons';
import { COLON_DELIMITER, DISCORD_COM_INVITE, JOIN_THE_COMMUNITY_SENTANCE_CASE } from '../../constants';

const UtilityOptions = () => {
	return (
		<div
			// Meant for buttons, that are not window controllers or navigation links.
			className="navigation_bar_utilities"
		>
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
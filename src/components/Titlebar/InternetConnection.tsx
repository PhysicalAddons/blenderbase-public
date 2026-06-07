import { Wifi, WifiOff } from '@carbon/react/icons';
import { useNetworkInformationStore } from '../../store/networkInformationStore';
import { cmd_check_internet_connection_STATUS_SENTANCE_CASE, COLON_DELIMITER, STATUS_OFF, STATUS_ON } from '../../constants';
import { WebUtilityService } from '../../services/webUtilityService';

const webUtilityService = new WebUtilityService();

const StatusInformation = () => {
    const { hasInternetConnection, setHasInternetConnection } = useNetworkInformationStore()

    const checkInternetConnectionOverride = async () => {
        try {
            const t = await webUtilityService.checkInternetConnection(true)
            setHasInternetConnection(t as boolean);
        } catch (e) {
            console.error(e);
        }
    }
    return (
        <div
            // Meant for buttons, that are not window controllers or navigation links.
            className="navigation_bar_utilities"
        >
            <span
                className={`navigation_bar_utilities_option ${hasInternetConnection ? "internet_connection_positive_navbar" : "internet_connection_negative_navbar"}`}
                title={`${cmd_check_internet_connection_STATUS_SENTANCE_CASE}${COLON_DELIMITER}${hasInternetConnection ? STATUS_ON : STATUS_OFF}`}
                onClick={() => checkInternetConnectionOverride()}
            >
                {hasInternetConnection ? (
                    <Wifi/>
                ) : (
                    <WifiOff/>
                )}
            </span>
        </div>
    )
}

export default StatusInformation
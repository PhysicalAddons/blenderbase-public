import { useRef } from 'react';
import { NavLink } from 'react-router-dom';
import { Home, LogoDiscord, Renew, Settings } from '@carbon/react/icons';
import { useShallow } from 'zustand/react/shallow';
import { COLON_DELIMITER, DISCORD_COM_INVITE, JOIN_THE_COMMUNITY_SENTANCE_CASE } from '../../constants';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { useSetupSyncStore } from '../../store/setupSyncStore';
import { postStatus, useStatusStore } from '../../store/statusStore';

const UtilityOptions = () => {
	const { isSettingsOpen, isSyncOpen, setIsSettingsOpen, setIsSyncOpen } = useUiControlsStore(
		useShallow((s) => ({ isSettingsOpen: s.isSettingsOpen, isSyncOpen: s.isSyncOpen, setIsSettingsOpen: s.setIsSettingsOpen, setIsSyncOpen: s.setIsSyncOpen }))
	)
	const hasNewerSetup = useSetupSyncStore((s) => Boolean(s.status?.is_newer))
	// Settings and Sync are panels in the middle column, not routes: the Home tab reads as
	// active only while both are closed, and the open panel's button takes the active look.
	const homeClassName = ({ isActive }: { isActive: boolean }) =>
		`navigation_bar_utilities_option navigation_bar_tab${isActive && !isSettingsOpen && !isSyncOpen ? " active" : ""}`;

	// The Sync button explains itself in the status line rather than a tooltip bubble; what
	// was shown before the hover comes back on leave, unless something else posted meanwhile.
	const statusBeforeHint = useRef<{ message: string, isBusy: boolean, isError: boolean } | null>(null);
	const syncHint = hasNewerSetup
		? "Sync · a newer setup is waiting in your sync folder"
		: "Sync · save your Blender setup for another computer, or apply one";
	const showSyncHint = () => {
		const s = useStatusStore.getState();
		if (s.isBusy) {
			return;
		}
		statusBeforeHint.current = { message: s.message, isBusy: s.isBusy, isError: s.isError };
		postStatus(syncHint);
	};
	const hideSyncHint = () => {
		const before = statusBeforeHint.current;
		statusBeforeHint.current = null;
		if (before && useStatusStore.getState().message === syncHint) {
			useStatusStore.getState().setStatus(before.message, before.isBusy, before.isError);
		}
	};

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
				onClick={() => { setIsSettingsOpen(false); setIsSyncOpen(false); }}
			>
				<Home/>
			</NavLink>
			<button
				type="button"
				className={`navigation_bar_utilities_option navigation_bar_tab${isSyncOpen ? " active" : ""}${hasNewerSetup ? " navigation_bar_tab--badge" : ""}`}
				aria-label={isSyncOpen ? "Close sync" : "Sync"}
				aria-pressed={isSyncOpen}
				onMouseEnter={showSyncHint}
				onMouseLeave={hideSyncHint}
				onFocus={showSyncHint}
				onBlur={hideSyncHint}
				onClick={() => { statusBeforeHint.current = null; setIsSyncOpen(!isSyncOpen); }}
			>
				<Renew/>
			</button>
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

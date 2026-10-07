import { useRef } from 'react';
import { NavLink } from 'react-router-dom';
import { Home, LogoDiscord, Renew, Settings, Trophy } from '@carbon/react/icons';
import { useShallow } from 'zustand/react/shallow';
import { COLON_DELIMITER, DISCORD_COM_INVITE, JOIN_THE_COMMUNITY_SENTANCE_CASE } from '../../constants';
import { useUiControlsStore } from '../../store/uiControlsStore';
import { useSetupSyncStore } from '../../store/setupSyncStore';
import { useActivityStore } from '../../store/activityStore';
import { postStatus, useStatusStore } from '../../store/statusStore';

/**
 * A title-bar button explains itself in the status line rather than a tooltip bubble; what
 * was shown before the hover comes back on leave, unless something else posted meanwhile.
 */
const useStatusHint = (hint: string) => {
	const statusBefore = useRef<{ message: string, isBusy: boolean, isError: boolean } | null>(null);
	const show = () => {
		const s = useStatusStore.getState();
		if (s.isBusy) {
			return;
		}
		statusBefore.current = { message: s.message, isBusy: s.isBusy, isError: s.isError };
		postStatus(hint);
	};
	const hide = () => {
		const before = statusBefore.current;
		statusBefore.current = null;
		if (before && useStatusStore.getState().message === hint) {
			useStatusStore.getState().setStatus(before.message, before.isBusy, before.isError);
		}
	};
	/** A click puts the earlier message back at once, so a hint about a badge the click clears does not linger. */
	const forget = () => { hide(); };
	/** Keyboard focus shows the hint like a hover does; the focus a mouse click leaves behind does not. */
	const showOnKeyboardFocus = (e: React.FocusEvent<HTMLElement>) => {
		if (e.currentTarget.matches(':focus-visible')) {
			show();
		}
	};
	return { show, hide, forget, showOnKeyboardFocus };
};

const UtilityOptions = () => {
	const { isSettingsOpen, isSyncOpen, isStatsOpen, setIsSettingsOpen, setIsSyncOpen, setIsStatsOpen } = useUiControlsStore(
		useShallow((s) => ({ isSettingsOpen: s.isSettingsOpen, isSyncOpen: s.isSyncOpen, isStatsOpen: s.isStatsOpen, setIsSettingsOpen: s.setIsSettingsOpen, setIsSyncOpen: s.setIsSyncOpen, setIsStatsOpen: s.setIsStatsOpen }))
	)
	const hasNewerSetup = useSetupSyncStore((s) => Boolean(s.status?.is_newer))
	const unseenUnlocks = useActivityStore((s) => s.unseenUnlocks)
	// Settings, Sync and Stats are panels in the middle column, not routes: the Home tab reads as
	// active only while all are closed, and the open panel's button takes the active look.
	const homeClassName = ({ isActive }: { isActive: boolean }) =>
		`navigation_bar_utilities_option navigation_bar_tab${isActive && !isSettingsOpen && !isSyncOpen && !isStatsOpen ? " active" : ""}`;

	const syncHint = useStatusHint(hasNewerSetup
		? "Sync · a newer setup is waiting in your sync folder"
		: "Sync · save your Blender setup for another computer, or apply one");
	const statsHint = useStatusHint(unseenUnlocks > 0
		? `Stats · ${unseenUnlocks === 1 ? "a new achievement is" : `${unseenUnlocks} new achievements are`} waiting`
		: "Stats · hours in each Blender version, and your achievements");

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
				onClick={() => { setIsSettingsOpen(false); setIsSyncOpen(false); setIsStatsOpen(false); }}
			>
				<Home/>
			</NavLink>
			<button
				type="button"
				className={`navigation_bar_utilities_option navigation_bar_tab${isSyncOpen ? " active" : ""}${hasNewerSetup ? " navigation_bar_tab--badge" : ""}`}
				aria-label={isSyncOpen ? "Close sync" : "Sync"}
				aria-pressed={isSyncOpen}
				onMouseEnter={syncHint.show}
				onMouseLeave={syncHint.hide}
				onFocus={syncHint.showOnKeyboardFocus}
				onBlur={syncHint.hide}
				onClick={() => { syncHint.forget(); setIsSyncOpen(!isSyncOpen); }}
			>
				<Renew/>
			</button>
			<button
				type="button"
				className={`navigation_bar_utilities_option navigation_bar_tab${isStatsOpen ? " active" : ""}${unseenUnlocks > 0 ? " navigation_bar_tab--badge" : ""}`}
				aria-label={isStatsOpen ? "Close stats" : "Stats"}
				aria-pressed={isStatsOpen}
				onMouseEnter={statsHint.show}
				onMouseLeave={statsHint.hide}
				onFocus={statsHint.showOnKeyboardFocus}
				onBlur={statsHint.hide}
				onClick={() => { statsHint.forget(); setIsStatsOpen(!isStatsOpen); }}
			>
				<Trophy/>
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

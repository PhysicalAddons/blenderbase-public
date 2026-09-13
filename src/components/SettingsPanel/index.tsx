import { useEffect, useRef, useState } from 'react';
import { Button, Dropdown, NumberInput, Toggle } from '@carbon/react';
import { Add, Star, StarFilled, TrashCan } from '@carbon/react/icons';
import { getVersion } from '@tauri-apps/api/app';
import { ask } from '@tauri-apps/plugin-dialog';
import { useShallow } from 'zustand/react/shallow';
import { IAppSetting, IBlenderInstallationLocation } from '../../models';
import { AppSettingCode } from '../../enums';
import { SettingsService } from '../../services/settingsService';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { ThemePreference, useThemeStore } from '../../store/themeStore';
import { postStatus, postStatusError } from '../../store/statusStore';
import { usePagedScroll } from '../../utility/usePagedScroll';

const settingsService = new SettingsService();

type SettingsSection = 'locations' | 'launch' | 'updates' | 'appearance';

const SECTIONS: { id: SettingsSection, label: string }[] = [
	{ id: 'locations', label: 'Locations' },
	{ id: 'launch', label: 'Launch' },
	{ id: 'updates', label: 'Updates' },
	{ id: 'appearance', label: 'Appearance' },
];

type ThemeOption = { id: ThemePreference, label: string };

const THEME_OPTIONS: ThemeOption[] = [
	{ id: 'auto', label: 'System' },
	{ id: 'light', label: 'Light' },
	{ id: 'dark', label: 'Dark' },
];

const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e));

type RowProps = {
	id: string,
	label: string,
	description: string,
	children: React.ReactNode,
}

/** One setting: label and description on the left, the control on the right. */
const SettingsRow = ({ id, label, description, children }: RowProps) => (
	<div className='settings_row'>
		<div className='settings_row__main'>
			<span className='settings_row__label' id={`${id}-label`}>{label}</span>
			<span className='settings_row__description' title={description}>{description}</span>
		</div>
		<div className='settings_row__control'>
			{children}
		</div>
	</div>
);

const SettingsPanel = () => {
	const [activeSection, setActiveSection] = useState<SettingsSection>('locations')
	const [appVersion, setAppVersion] = useState<string>("")
	const [appSettings, setAppSettings] = useState<IAppSetting[]>([])
	const [locations, setLocations] = useState<IBlenderInstallationLocation[]>([])
	const [hasLoadedLocations, setHasLoadedLocations] = useState<boolean>(false)
	const [isCheckingForUpdates, setIsCheckingForUpdates] = useState<boolean>(false)
	// The cooldown field edits a local draft; the value is saved on blur or Enter.
	const [cooldownDraft, setCooldownDraft] = useState<string>("")
	const [isCooldownInvalid, setIsCooldownInvalid] = useState<boolean>(false)
	const listRef = useRef<HTMLDivElement>(null)
	usePagedScroll(listRef, { rowSelector: '.settings_row, .settings_location_row' })

	const { installedBuilds, refreshInstalledBuilds } = useBlenderManagerStore(
		useShallow((s) => ({ installedBuilds: s.installedBuilds, refreshInstalledBuilds: s.refreshInstalledBuilds }))
	)
	const { preference, setPreference } = useThemeStore(
		useShallow((s) => ({ preference: s.preference, setPreference: s.setPreference }))
	)

	const settingByCode = (code: AppSettingCode): IAppSetting | undefined =>
		appSettings.find((s) => s.code === code);

	const cooldownSetting = settingByCode(AppSettingCode.SET_CHECK_INTERNET_CONNECTION_TIMEOUT);

	useEffect(() => {
		let cancelled = false;
		const ifMounted = <T,>(setter: (v: T) => void) => (v: T) => {
			if (!cancelled) {
				setter(v);
			}
		};
		getVersion()
			.then(ifMounted(setAppVersion))
			.catch((e) => console.error(e));
		settingsService.fetchAppSetting(null, null, null, null, null)
			.then(ifMounted(setAppSettings))
			.catch((e) => {
				console.error(e);
				postStatusError(`Loading settings failed: ${errorText(e)}`);
			});
		settingsService.fetchBlenderInstallationPaths(null, null, null, null)
			.then(ifMounted((v: IBlenderInstallationLocation[]) => {
				setLocations(v);
				setHasLoadedLocations(true);
			}))
			.catch((e) => {
				console.error(e);
				postStatusError(`Loading installation locations failed: ${errorText(e)}`);
			});
		return () => {
			cancelled = true;
		};
	}, []);

	// Follow the stored cooldown when it changes elsewhere (e.g. after a refetch).
	useEffect(() => {
		setCooldownDraft(String(cooldownSetting?.int_value ?? ""));
		setIsCooldownInvalid(false);
	}, [cooldownSetting?.int_value]);

	const reloadSettings = async () => {
		try {
			setAppSettings(await settingsService.fetchAppSetting(null, null, null, null, null));
		} catch (e) {
			console.error(e);
			postStatusError(`Reloading settings failed: ${errorText(e)}`);
		}
	};

	/** Applies a setting and reloads the list; resolves with the error message if it was refused. */
	const handleSetting = async (appSetting: IAppSetting): Promise<string | undefined> => {
		let err: string | undefined;
		try {
			await settingsService.handleSetting(appSetting);
		} catch (e) {
			console.error(e);
			err = errorText(e);
		}
		await reloadSettings();
		return err;
	};

	const toggleSetting = async (setting: IAppSetting) => {
		const err = await handleSetting({ ...setting, int_value: +!setting.int_value });
		if (err) {
			postStatusError(`Changing "${setting.name}" failed: ${err}`);
		}
	};

	const checkForUpdates = async () => {
		const setting = settingByCode(AppSettingCode.CHECK_FOR_UPDATE);
		if (!setting) {
			return;
		}
		setIsCheckingForUpdates(true);
		postStatus("Checking for updates…", true);
		try {
			const err = await handleSetting(setting);
			if (err) {
				postStatusError(`Checking for updates failed: ${err}`);
			} else {
				postStatus("Update check finished");
			}
		} finally {
			setIsCheckingForUpdates(false);
		}
	};

	const openReleaseNotes = async () => {
		const setting = settingByCode(AppSettingCode.OPEN_APP_VERSION_ONLINE_REPOSITORY);
		if (!setting) {
			return;
		}
		const err = await handleSetting(setting);
		if (err) {
			postStatusError(`Opening the releases page failed: ${err}`);
		}
	};

	const commitCooldown = async () => {
		if (!cooldownSetting) {
			return;
		}
		const value = Number(cooldownDraft);
		if (cooldownDraft.trim() === "" || Number.isNaN(value)) {
			setIsCooldownInvalid(true);
			postStatusError("Enter a number of seconds for the internet check cooldown");
			return;
		}
		if (value === (cooldownSetting.int_value ?? 0)) {
			setIsCooldownInvalid(false);
			return;
		}
		const min = cooldownSetting.min_int_value;
		const max = cooldownSetting.max_int_value;
		if (min !== null && value < min) {
			setIsCooldownInvalid(true);
			postStatusError(`The internet check cooldown must be at least ${min} seconds`);
			return;
		}
		if (max !== null && value > max) {
			setIsCooldownInvalid(true);
			postStatusError(`The internet check cooldown must be at most ${max} seconds`);
			return;
		}
		const err = await handleSetting({ ...cooldownSetting, int_value: value });
		setIsCooldownInvalid(err !== undefined);
		if (err) {
			postStatusError(`Changing the internet check cooldown failed: ${err}`);
		} else {
			postStatus(`Internet check cooldown set to ${value} seconds`);
		}
	};

	const reloadLocations = async () => {
		try {
			setLocations(await settingsService.fetchBlenderInstallationPaths(null, null, null, null));
		} catch (e) {
			console.error(e);
			postStatusError(`Loading installation locations failed: ${errorText(e)}`);
		}
	};

	const addLocation = async () => {
		try {
			const added = await settingsService.insertBlenderInstallationLocation();
			if (!added) {
				// The folder picker was cancelled: nothing changed, nothing to report.
				return;
			}
			await reloadLocations();
			// A new location may already hold Blender versions: rescan the disk.
			await refreshInstalledBuilds();
			postStatus("Installation location added");
		} catch (e) {
			console.error(e);
			postStatusError(`Adding the installation location failed: ${errorText(e)}`);
		}
	};

	const setLocationAsDefault = async (location: IBlenderInstallationLocation) => {
		try {
			// The command toggles from the current state: passing the present value makes it the default.
			await settingsService.setBlenderInstallationLocationAsDefault(location.id, location.is_default);
			await reloadLocations();
			postStatus(`New Blender versions are installed in ${location.directory_path}`);
		} catch (e) {
			console.error(e);
			postStatusError(`Changing the default installation location failed: ${errorText(e)}`);
		}
	};

	const removeLocation = async (location: IBlenderInstallationLocation) => {
		const confirmed = await ask(
			"Remove this location from Blenderbase? Versions inside it stay on disk.",
			{ title: "Remove location", kind: "warning", okLabel: "Remove", cancelLabel: "Cancel" }
		);
		if (!confirmed) {
			return;
		}
		try {
			await settingsService.deleteBlenderInstallationLocation(location.id);
			await reloadLocations();
			await refreshInstalledBuilds();
			postStatus("Installation location removed");
		} catch (e) {
			console.error(e);
			postStatusError(`Removing the installation location failed: ${errorText(e)}`);
		}
	};

	const locationMeta = (location: IBlenderInstallationLocation): string => {
		const count = installedBuilds.filter((b) => (b.installation_directory_path ?? "").startsWith(location.directory_path)).length;
		const versions = `${count} ${count === 1 ? "version" : "versions"}`;
		return `${versions} · ${location.write ? "writable" : "read-only"}`;
	};

	const toggleRow = (code: AppSettingCode, id: string, label: string, description: string) => {
		const setting = settingByCode(code);
		return (
			<SettingsRow id={id} label={label} description={description}>
				<Toggle
					id={id}
					size="sm"
					hideLabel
					aria-labelledby={`${id}-label`}
					toggled={Boolean(setting?.int_value)}
					disabled={!setting}
					onToggle={() => setting && toggleSetting(setting)}
				/>
			</SettingsRow>
		);
	};

	const renderLocations = () => {
		if (locations.length === 0) {
			return (
				<div className='settings_panel__empty'>
					{hasLoadedLocations
						? "No installation location yet. Use \"Add location\" to choose where Blender versions are installed."
						: "Loading installation locations…"}
				</div>
			);
		}
		return locations.map((location) => (
			<div key={location.id} className='settings_location_row' title={location.directory_path}>
				<div className='settings_location_row__main'>
					<span className='settings_location_row__path'>{location.directory_path}</span>
					<span className='settings_location_row__meta'>{locationMeta(location)}</span>
				</div>
				<Button
					kind="ghost"
					size="md"
					className={`settings_location_row__star ${location.is_default ? "settings_location_row__star--default" : ""}`}
					renderIcon={location.is_default ? StarFilled : Star}
					iconDescription={location.is_default ? "Default location" : "Set as default"}
					title={location.is_default ? "Default location" : "Set as default"}
					hasIconOnly
					onClick={() => {
						if (!location.is_default) {
							void setLocationAsDefault(location);
						}
					}}
				/>
				<Button
					kind="ghost"
					size="md"
					className='settings_location_row__delete'
					renderIcon={TrashCan}
					iconDescription="Remove location"
					title="Remove location"
					hasIconOnly
					onClick={() => void removeLocation(location)}
				/>
			</div>
		));
	};

	const renderLaunch = () => (
		toggleRow(
			AppSettingCode.MINIMIZE_BLENDERBASE_ON_LAUNCH,
			"setting-minimize-on-launch",
			"Minimise Blenderbase when launching Blender",
			"The window minimises after Blender starts",
		)
	);

	const renderUpdates = () => (
		<>
			{toggleRow(
				AppSettingCode.CHECK_FOR_UPDATE_ON_LAUNCH,
				"setting-check-for-update-on-launch",
				"Check for updates on launch",
				"Asks before installing anything",
			)}
			<SettingsRow id="setting-check-for-update" label="Check now" description="Looks for a newer release on GitHub">
				<Button
					kind="tertiary"
					size="md"
					className='settings_row__button'
					disabled={isCheckingForUpdates || !settingByCode(AppSettingCode.CHECK_FOR_UPDATE)}
					onClick={() => void checkForUpdates()}
				>
					{isCheckingForUpdates ? "Checking…" : "Check for updates"}
				</Button>
			</SettingsRow>
			<SettingsRow id="setting-release-notes" label="Release notes" description="Opens the releases page">
				<Button
					kind="tertiary"
					size="md"
					className='settings_row__button'
					disabled={!settingByCode(AppSettingCode.OPEN_APP_VERSION_ONLINE_REPOSITORY)}
					onClick={() => void openReleaseNotes()}
				>
					Open
				</Button>
			</SettingsRow>
			<SettingsRow id="setting-internet-cooldown" label="Internet check cooldown" description="Seconds between connectivity checks">
				<NumberInput
					id="setting-internet-cooldown-input"
					className='settings_row__number'
					size="sm"
					hideLabel
					hideSteppers
					allowEmpty
					label="Internet check cooldown"
					aria-labelledby="setting-internet-cooldown-label"
					min={cooldownSetting?.min_int_value ?? undefined}
					max={cooldownSetting?.max_int_value ?? undefined}
					value={cooldownDraft}
					invalid={isCooldownInvalid}
					disabled={!cooldownSetting}
					onChange={(_e, { value }) => setCooldownDraft(String(value ?? ""))}
					onBlur={() => void commitCooldown()}
					onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => {
						if (e.key === "Enter") {
							e.currentTarget.blur();
						}
					}}
				/>
			</SettingsRow>
		</>
	);

	const renderAppearance = () => (
		<SettingsRow id="setting-theme" label="Theme" description="Follows the system, or a fixed light or dark look">
			<Dropdown<ThemeOption>
				id="setting-theme-dropdown"
				className='settings_row__dropdown'
				size="sm"
				hideLabel
				titleText="Theme"
				label="Theme"
				items={THEME_OPTIONS}
				itemToString={(item) => item?.label ?? ""}
				selectedItem={THEME_OPTIONS.find((o) => o.id === preference) ?? THEME_OPTIONS[0]}
				onChange={({ selectedItem }) => {
					if (selectedItem) {
						setPreference(selectedItem.id);
					}
				}}
			/>
		</SettingsRow>
	);

	const renderSection = () => {
		switch (activeSection) {
			case 'locations':
				return renderLocations();
			case 'launch':
				return renderLaunch();
			case 'updates':
				return renderUpdates();
			case 'appearance':
				return renderAppearance();
		}
	};

	return (
		<div className='settings_panel'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title'>Settings</span>
					<span className='column_header__subtitle'>{appVersion ? `Blenderbase ${appVersion}` : ""}</span>
				</div>
			</div>
			<div className='column_actions settings_panel__toolbar'>
				<div className='build_type_switch' role="tablist" aria-label="Settings section">
					{SECTIONS.map((section) => (
						<Button
							key={section.id}
							kind="secondary"
							size="lg"
							role="tab"
							aria-selected={section.id === activeSection}
							className={`build_type_switch__option ${section.id === activeSection ? "build_type_switch__option--selected" : ""}`}
							title={`${section.label} settings`}
							onClick={() => setActiveSection(section.id)}
						>
							{section.label}
						</Button>
					))}
				</div>
				{activeSection === 'locations' && (
					<div className='settings_panel__toolbar_actions'>
						<Button
							kind="ghost"
							size="lg"
							className='settings_panel__add'
							renderIcon={Add}
							title="Add an installation location"
							onClick={() => void addLocation()}
						>
							Add location
						</Button>
					</div>
				)}
			</div>
			{activeSection === 'locations' ? (
				<div className='list_header settings_panel__list_header settings_panel__list_header--locations'>
					<span>Folder</span>
					<span className='centered'>Default</span>
					<span></span>
				</div>
			) : (
				<div className='list_header settings_panel__list_header'>
					<span>Setting</span>
					<span></span>
				</div>
			)}
			<div className={`settings_panel__list ${activeSection === 'locations' ? "settings_panel__list--scroll" : ""}`} ref={listRef}>
				{renderSection()}
			</div>
		</div>
	)
}

export default SettingsPanel

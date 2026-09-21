import { useEffect, useState } from 'react';
import { Button, InlineLoading, TextInput, Toggle } from '@carbon/react';
import { Copy, TrashCan } from '@carbon/react/icons';
import { listen } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useShallow } from 'zustand/react/shallow';
import { ISetupBundleInfo } from '../../models';
import { SETUP_FILE_FILTER } from '../../constants';
import { SetupService } from '../../services/setupService';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { useSetupRestoreStore } from '../../store/setupRestoreStore';
import { describeSyncFile, useSetupSyncStore } from '../../store/setupSyncStore';
import { postStatus, postStatusError } from '../../store/statusStore';

const setupService = new SetupService();

type SyncSection = 'folder' | 'transfer' | 'file';

/** One way to move a setup: the tab label and the one-line explanation under the title. */
const SECTIONS: { id: SyncSection, label: string, subtitle: string }[] = [
	{ id: 'folder', label: 'Sync folder', subtitle: "A shared folder in a cloud drive keeps every computer's setup in step" },
	{ id: 'transfer', label: 'Transfer code', subtitle: 'Send the setup through an encrypted relay to a computer that shares no folder' },
	{ id: 'file', label: 'Setup file', subtitle: 'One .bbsetup file to carry yourself: a USB stick, an email, any drive' },
];

const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e)).replace(/^cmd_\w+: /, "");

const formatSetupSize = (bytes: number): string => {
	const kb = bytes / 1024;
	return kb >= 1024 ? `${(kb / 1024).toFixed(1)} MB` : `${Math.max(1, Math.round(kb))} KB`;
};

const plural = (count: number, one: string, many: string): string => `${count} ${count === 1 ? one : many}`;

/** One status line for a setup file: what it holds and how much of it restores on its own. */
const describeSetup = (info: ISetupBundleInfo): string => {
	const sections = Object.values(info.manifest.series);
	const addons = sections.flatMap((s) => s.addons).filter((a) => a.source !== "core");
	const manual = addons.filter((a) => a.source === "manual" || (a.source === "file" && !a.file)).length;
	const parts = [
		plural(info.manifest.blender.length, "Blender version", "Blender versions"),
		plural(sections.length, "configuration", "configurations"),
		plural(addons.length, "addon", "addons") + (manual > 0 ? ` (${manual} to install by hand)` : ""),
		formatSetupSize(info.file_size),
	];
	return parts.join(" · ");
};

type RowProps = {
	id: string,
	label: string,
	description: string,
	children: React.ReactNode,
}

/** One action: label and description on the left, the control on the right; the Settings row look. */
const Row = ({ id, label, description, children }: RowProps) => (
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

/**
 * Everything that moves a Blender setup between computers, one tab per way: the sync folder,
 * a transfer code, a setup file. Takes the middle column and the tab band like Settings.
 */
const SyncPanel = () => {
	const [activeSection, setActiveSection] = useState<SyncSection>('folder')
	const installedBuilds = useBlenderManagerStore((s) => s.installedBuilds)
	const openSetup = useSetupRestoreStore((s) => s.open)
	const { syncStatus, isSyncBusy, loadSync, setSyncFolder, saveToSyncFolder, openSyncFile } = useSetupSyncStore(
		useShallow((s) => ({ syncStatus: s.status, isSyncBusy: s.isBusy, loadSync: s.load, setSyncFolder: s.setFolder, saveToSyncFolder: s.save, openSyncFile: s.openForApply }))
	)
	const { sent, isSending, isReceiving, sendTransfer, receiveTransfer } = useSetupSyncStore(
		useShallow((s) => ({ sent: s.sent, isSending: s.isSending, isReceiving: s.isReceiving, sendTransfer: s.send, receiveTransfer: s.receive }))
	)
	// One choice for every tab: the folder, the transfer and the file all pack addons the same way.
	const [includeAddonFiles, setIncludeAddonFiles] = useState<boolean>(false)
	const [isSavingSetup, setIsSavingSetup] = useState<boolean>(false)
	const [codeDraft, setCodeDraft] = useState<string>("")

	useEffect(() => {
		void loadSync();
	}, []);

	const isBusy = isSyncBusy || isSavingSetup || isSending || isReceiving;

	const copyCode = async () => {
		if (!sent) {
			return;
		}
		try {
			await navigator.clipboard.writeText(sent.code);
			postStatus(`Copied ${sent.code}`);
		} catch (e) {
			console.error(e);
			postStatusError("Could not copy the code; select it and copy it by hand");
		}
	};

	const sentDescription = (): string => {
		if (!sent) {
			return "Uploads the setup, encrypted, and gives you a code to type on the other computer";
		}
		const expires = sent.expires ? new Date(sent.expires).toLocaleDateString() : "";
		const size = sent.size >= 1024 * 1024 ? `${(sent.size / 1024 / 1024).toFixed(1)} MB` : `${Math.max(1, Math.round(sent.size / 1024))} KB`;
		return `Type this code on the other computer${expires ? ` before ${expires}` : ""} · ${size}`;
	};

	const chooseSyncFolder = async () => {
		try {
			const selected = await open({ multiple: false, directory: true, title: "Choose the folder that keeps your setup in sync", defaultPath: syncStatus?.folder_path || undefined });
			if (typeof selected === "string" && selected.length > 0) {
				await setSyncFolder(selected);
			}
		} catch (e) {
			console.error(e);
			postStatusError(`Choosing the sync folder failed: ${errorText(e)}`);
		}
	};

	const saveSetup = async () => {
		let filePath: string | null = null;
		try {
			filePath = await save({ title: "Save setup", defaultPath: "My Blender setup.bbsetup", filters: SETUP_FILE_FILTER });
		} catch (e) {
			console.error(e);
			postStatusError(`Choosing where to save the setup failed: ${errorText(e)}`);
		}
		if (!filePath) {
			return;
		}
		setIsSavingSetup(true);
		postStatus("Reading the setup from every Blender series…", true);
		// The backend names each step (series being read, addon being packed) as it goes.
		const stopListening = await listen<string>("setup-progress", (event) => postStatus(event.payload, true));
		try {
			const info = await setupService.exportSetupBundle(filePath, includeAddonFiles);
			info.warnings.forEach((w) => console.warn(w));
			postStatus(`Setup saved: ${describeSetup(info)}`);
		} catch (e) {
			console.error(e);
			postStatusError(`Saving the setup failed: ${errorText(e)}`);
		} finally {
			stopListening();
			setIsSavingSetup(false);
		}
	};

	// The file opens in its own view, where each series can be ticked and applied.
	const restoreFromFile = async () => {
		try {
			const selected = await open({ multiple: false, directory: false, title: "Open a setup file", filters: SETUP_FILE_FILTER });
			if (typeof selected !== "string" || selected.length === 0) {
				return;
			}
			await openSetup(selected);
		} catch (e) {
			console.error(e);
			postStatusError(`Opening the setup file failed: ${errorText(e)}`);
		}
	};

	const folderSet = Boolean(syncStatus?.folder_path);

	// The last row of every tab: the same switch, since it shapes whatever is saved or sent.
	const renderAddonFilesRow = () => (
		<Row id="sync-addon-files" label="Include addon files" description="Packs addons installed from a file, so they restore without the download">
			<Toggle
				id="sync-addon-files"
				size="sm"
				hideLabel
				aria-labelledby="sync-addon-files-label"
				toggled={includeAddonFiles}
				disabled={isBusy}
				onToggle={(checked) => setIncludeAddonFiles(checked)}
			/>
		</Row>
	);

	const renderFolder = () => (
		<>
			<Row id="sync-folder" label="Folder" description={folderSet ? syncStatus!.folder_path : "Choose a folder inside Dropbox, OneDrive, iCloud or Google Drive"}>
				{folderSet && (
					<Button
						kind="ghost"
						size="md"
						className='settings_location_row__delete'
						renderIcon={TrashCan}
						iconDescription="Stop using this sync folder"
						title="Stop using this sync folder"
						hasIconOnly
						disabled={isBusy}
						onClick={() => void setSyncFolder(null)}
					/>
				)}
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy} onClick={() => void chooseSyncFolder()}>
					{folderSet ? "Change…" : "Choose…"}
				</Button>
			</Row>
			<Row id="sync-save" label="Save this computer's setup" description={folderSet ? "Writes the setup to the folder; other computers are told it is newer" : "Choose a folder first"}>
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || !folderSet || installedBuilds.length === 0} onClick={() => void saveToSyncFolder(includeAddonFiles)}>
					{isSyncBusy ? "Saving…" : "Save now"}
				</Button>
			</Row>
			<Row id="sync-apply" label="Setup in the folder" description={folderSet ? describeSyncFile(syncStatus!) : "Choose a folder first"}>
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || !syncStatus?.file} onClick={() => void openSyncFile()}>
					Apply…
				</Button>
			</Row>
			{renderAddonFilesRow()}
		</>
	);

	const renderTransfer = () => (
		<>
			<Row id="sync-transfer-send" label={sent ? "Your transfer code" : "Send to another computer"} description={sentDescription()}>
				{sent && (
					<>
						<code className='sync_panel__code'>{sent.code}</code>
						<Button kind="ghost" size="md" className='settings_location_row__delete' renderIcon={Copy} iconDescription="Copy the code" title="Copy the code" hasIconOnly onClick={() => void copyCode()} />
					</>
				)}
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || installedBuilds.length === 0} onClick={() => void sendTransfer(includeAddonFiles)}>
					{isSending ? "Sending…" : sent ? "Send again" : "Send…"}
				</Button>
			</Row>
			<Row id="sync-transfer-receive" label="Receive with a code" description="Fetches the setup the code stands for and shows what it would apply">
				<TextInput
					id="sync-transfer-code"
					className='sync_panel__code_input'
					size="md"
					hideLabel
					labelText="Transfer code"
					placeholder="brave-otter-4412"
					value={codeDraft}
					disabled={isBusy}
					onChange={(e: React.ChangeEvent<HTMLInputElement>) => setCodeDraft(e.target.value)}
					onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => {
						if (e.key === "Enter" && codeDraft.trim().length > 0 && !isBusy) {
							void receiveTransfer(codeDraft);
						}
					}}
				/>
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || codeDraft.trim().length === 0} onClick={() => void receiveTransfer(codeDraft)}>
					{isReceiving ? "Receiving…" : "Receive"}
				</Button>
			</Row>
			{renderAddonFilesRow()}
		</>
	);

	const renderFile = () => (
		<>
			<Row id="sync-file-save" label="Save setup to a file" description="Blender versions, preferences, theme, keymaps and the addon list of every series">
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || installedBuilds.length === 0} onClick={() => void saveSetup()}>
					{isSavingSetup ? "Saving…" : "Save…"}
				</Button>
			</Row>
			<Row id="sync-file-restore" label="Restore from a setup file" description="Opens a .bbsetup file and shows what it would apply; double-clicking one does the same">
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy} onClick={() => void restoreFromFile()}>
					Open…
				</Button>
			</Row>
			{renderAddonFilesRow()}
		</>
	);

	const renderSection = () => {
		switch (activeSection) {
			case 'folder':
				return renderFolder();
			case 'transfer':
				return renderTransfer();
			case 'file':
				return renderFile();
		}
	};

	const section = SECTIONS.find((s) => s.id === activeSection) ?? SECTIONS[0];

	return (
		<div className='settings_panel sync_panel'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title'>Sync</span>
					<span className='column_header__subtitle'>{section.subtitle}</span>
				</div>
				{isBusy && <InlineLoading className="column_header__loading" iconDescription="Working" />}
			</div>
			<div className='column_actions settings_panel__toolbar'>
				<div className='build_type_switch' role="tablist" aria-label="Way to move the setup">
					{SECTIONS.map((s) => (
						<Button
							key={s.id}
							kind="secondary"
							size="lg"
							role="tab"
							aria-selected={s.id === activeSection}
							className={`build_type_switch__option ${s.id === activeSection ? "build_type_switch__option--selected" : ""}`}
							onClick={() => setActiveSection(s.id)}
						>
							{s.label}
						</Button>
					))}
				</div>
			</div>
			<div className='list_header settings_panel__list_header'>
				<span>Action</span>
				<span></span>
			</div>
			<div className='settings_panel__list'>
				{renderSection()}
			</div>
		</div>
	)
}

export default SyncPanel

import { useEffect, useRef, useState } from 'react';
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
import { usePagedScroll } from '../../utility/usePagedScroll';

const setupService = new SetupService();

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

/** A titled group of rows. */
const Section = ({ title, text, children }: { title: string, text: string, children: React.ReactNode }) => (
	<>
		<div className='sync_panel__section'>
			<span className='sync_panel__section_title'>{title}</span>
			<span className='sync_panel__section_text'>{text}</span>
		</div>
		{children}
	</>
);

/**
 * Everything that moves a Blender setup between computers: the sync folder, setup files,
 * and later transfer codes and the local network. Takes the middle column like Settings.
 */
const SyncPanel = () => {
	const installedBuilds = useBlenderManagerStore((s) => s.installedBuilds)
	const openSetup = useSetupRestoreStore((s) => s.open)
	const { syncStatus, isSyncBusy, loadSync, setSyncFolder, saveToSyncFolder, openSyncFile } = useSetupSyncStore(
		useShallow((s) => ({ syncStatus: s.status, isSyncBusy: s.isBusy, loadSync: s.load, setSyncFolder: s.setFolder, saveToSyncFolder: s.save, openSyncFile: s.openForApply }))
	)
	const { sent, isSending, isReceiving, sendTransfer, receiveTransfer } = useSetupSyncStore(
		useShallow((s) => ({ sent: s.sent, isSending: s.isSending, isReceiving: s.isReceiving, sendTransfer: s.send, receiveTransfer: s.receive }))
	)
	const [includeAddonFiles, setIncludeAddonFiles] = useState<boolean>(false)
	const [isSavingSetup, setIsSavingSetup] = useState<boolean>(false)
	const [codeDraft, setCodeDraft] = useState<string>("")
	const listRef = useRef<HTMLDivElement>(null)
	usePagedScroll(listRef, { rowSelector: '.settings_row' })

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
			return "Uploads this computer's setup, encrypted, and gives you a code to type on the other computer";
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

	return (
		<div className='settings_panel sync_panel'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title'>Sync</span>
					<span className='column_header__subtitle'>Your Blender versions, preferences, theme, keymaps and addons on every computer</span>
				</div>
				{isBusy && <InlineLoading className="column_header__loading" iconDescription="Working" />}
			</div>
			<div className='list_header settings_panel__list_header'>
				<span>Action</span>
				<span></span>
			</div>
			<div className='settings_panel__list settings_panel__list--scroll' ref={listRef}>
				<Section title="Sync folder" text="A folder inside Dropbox, OneDrive, iCloud or Google Drive. Every computer saves its setup there and is told when another one saved a newer setup.">
					<Row id="sync-folder" label="Folder" description={folderSet ? syncStatus!.folder_path : "Not chosen yet"}>
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
				</Section>
				<Section title="Transfer code" text="For a computer that is not on the same network and shares no folder: the setup goes through a relay, encrypted with a code only the two computers know. It is removed once it is received, or after 7 days.">
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
				</Section>
				<Section title="Setup file" text="One .bbsetup file to carry yourself: a USB stick, an email, any drive. Open it on the other computer, or double-click it once Blenderbase is installed there.">
					<Row id="sync-file-save" label="Save setup to a file" description="Blender versions, preferences, theme, keymaps and the addon list of every series">
						<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || installedBuilds.length === 0} onClick={() => void saveSetup()}>
							{isSavingSetup ? "Saving…" : "Save…"}
						</Button>
					</Row>
					<Row id="sync-file-addons" label="Include addon files" description="Packs addons that were installed from a file, so they restore without the download. Applies to both the folder and a file">
						<Toggle
							id="sync-file-addons"
							size="sm"
							hideLabel
							aria-labelledby="sync-file-addons-label"
							toggled={includeAddonFiles}
							disabled={isBusy}
							onToggle={(checked) => setIncludeAddonFiles(checked)}
						/>
					</Row>
					<Row id="sync-file-restore" label="Restore from a setup file" description="Opens a .bbsetup file and shows what it would apply">
						<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy} onClick={() => void restoreFromFile()}>
							Open…
						</Button>
					</Row>
				</Section>
			</div>
		</div>
	)
}

export default SyncPanel

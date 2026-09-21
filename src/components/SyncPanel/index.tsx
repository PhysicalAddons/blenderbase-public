import { useEffect, useRef, useState } from 'react';
import { Button, InlineLoading, TextInput, Toggle } from '@carbon/react';
import { Copy, Information, TrashCan } from '@carbon/react/icons';
import { listen } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useShallow } from 'zustand/react/shallow';
import { ILanPeer, ISetupBundleInfo } from '../../models';
import { SETUP_FILE_FILTER, SYNC_DOCUMENTATION_URL } from '../../constants';
import { SetupService } from '../../services/setupService';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { useSetupRestoreStore } from '../../store/setupRestoreStore';
import { describeSyncFile, useSetupSyncStore } from '../../store/setupSyncStore';
import { formatLanSize, formatPin, platformLabel, useSetupLanStore } from '../../store/setupLanStore';
import { describeShare, exportOptions, useSetupShareStore } from '../../store/setupShareStore';
import { SyncSection, useUiControlsStore } from '../../store/uiControlsStore';
import { postStatus, postStatusError, useStatusStore } from '../../store/statusStore';

const setupService = new SetupService();

const DOCUMENTATION_HINT = 'How syncing works · opens the documentation in your browser';

/**
 * One way to share a setup: the tab label and the line under the tabs that says when it fits.
 * In order of preference: the first tab is the one that opens.
 */
const SECTIONS: { id: SyncSection, label: string, when: string }[] = [
	{ id: 'network', label: 'Local network', when: 'If your computers are on the same network. Turn on sharing here and type the PIN on the other ones. No internet needed' },
	{ id: 'transfer', label: 'Transfer code', when: 'Your computers are outside the local network. The setup travels encrypted through a relay under a short code. Requires an internet connection' },
	{ id: 'folder', label: 'Sync folder', when: 'Your own computers, kept in step through a folder in Dropbox, OneDrive, iCloud or Google Drive' },
	{ id: 'file', label: 'Setup file', when: 'A .bbsetup file you carry yourself, on a USB stick, in an email, on any drive' },
];

/** The four ways lead to one place; said once, under the rows of whichever is selected. */
const AFTER_RECEIVING = 'A received setup opens in the restore view, where you choose per Blender series what to apply. Nothing changes until you apply it.';

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
	/** The step before this one is not done yet: the row waits, dimmed, until it is. */
	inactive?: boolean,
	children: React.ReactNode,
}

/** One action: label and description on the left, the control on the right; the Settings row look. */
const Row = ({ id, label, description, inactive = false, children }: RowProps) => (
	<div className={`settings_row${inactive ? " settings_row--inactive" : ""}`}>
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
	const { activeSection, setActiveSection, setIsShareSetupOpen } = useUiControlsStore(
		useShallow((s) => ({ activeSection: s.syncSection, setActiveSection: s.setSyncSection, setIsShareSetupOpen: s.setIsShareSetupOpen }))
	)
	const installedBuilds = useBlenderManagerStore((s) => s.installedBuilds)
	const openSetup = useSetupRestoreStore((s) => s.open)
	const { syncStatus, isSyncBusy, loadSync, setSyncFolder, saveToSyncFolder, openSyncFile } = useSetupSyncStore(
		useShallow((s) => ({ syncStatus: s.status, isSyncBusy: s.isBusy, loadSync: s.load, setSyncFolder: s.setFolder, saveToSyncFolder: s.save, openSyncFile: s.openForApply }))
	)
	const { sent, isSending, isReceiving, sendTransfer, receiveTransfer } = useSetupSyncStore(
		useShallow((s) => ({ sent: s.sent, isSending: s.isSending, isReceiving: s.isReceiving, sendTransfer: s.send, receiveTransfer: s.receive }))
	)
	// One choice of what goes, for every tab; made in the What to share view.
	const shareSelection = useSetupShareStore((s) => s.selection)
	const shareOptions = exportOptions(shareSelection)
	const [isSavingSetup, setIsSavingSetup] = useState<boolean>(false)
	const [codeDraft, setCodeDraft] = useState<string>("")
	const { lan, isStartingShare, isLanReceiving, refreshLan, browseLan, shareLan, stopLanShare, receiveLan } = useSetupLanStore(
		useShallow((s) => ({ lan: s.status, isStartingShare: s.isStartingShare, isLanReceiving: s.isReceiving, refreshLan: s.refresh, browseLan: s.browse, shareLan: s.share, stopLanShare: s.stopShare, receiveLan: s.receive }))
	)
	// One PIN draft per computer on the network.
	const [pinDrafts, setPinDrafts] = useState<Record<string, string>>({})

	useEffect(() => {
		void loadSync();
	}, []);

	// Other computers are looked for only while this tab shows; a share that is on keeps announcing without it.
	useEffect(() => {
		if (activeSection !== 'network') {
			return;
		}
		void browseLan(true);
		const timer = window.setInterval(() => void refreshLan(), 2000);
		return () => {
			window.clearInterval(timer);
			void browseLan(false);
		};
	}, [activeSection]);

	const isBusy = isSyncBusy || isSavingSetup || isSending || isReceiving || isStartingShare || isLanReceiving;

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
			const info = await setupService.exportSetupBundle(filePath, shareOptions);
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

	// The first row of every tab: what goes is one choice, whichever way it travels.
	const renderWhatToShareRow = () => (
		<Row id="sync-what" label="What to share" description={describeShare(shareSelection, installedBuilds)}>
			<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || installedBuilds.length === 0} onClick={() => setIsShareSetupOpen(true)}>
				Choose…
			</Button>
		</Row>
	);

	const renderFolder = () => (
		<>
			{renderWhatToShareRow()}
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
			<Row id="sync-save" label="Save this computer's setup" description={folderSet ? "Writes the setup to the folder; other computers are told it is newer" : "Choose a folder first"} inactive={!folderSet}>
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || !folderSet || installedBuilds.length === 0} onClick={() => void saveToSyncFolder(shareOptions)}>
					{isSyncBusy ? "Saving…" : "Save now"}
				</Button>
			</Row>
			{/* Waits for a file to apply; a file that cannot be read stays readable, since the row carries the error. */}
			<Row id="sync-apply" label="Setup in the folder" description={folderSet ? describeSyncFile(syncStatus!) : "Choose a folder first"} inactive={!folderSet || (!syncStatus?.file && !syncStatus?.file_error)}>
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || !syncStatus?.file} onClick={() => void openSyncFile()}>
					Apply…
				</Button>
			</Row>
		</>
	);

	const renderTransfer = () => (
		<>
			{renderWhatToShareRow()}
			<Row id="sync-transfer-send" label={sent ? "Your transfer code" : "Send to another computer"} description={sentDescription()}>
				{sent && (
					<>
						<code className='sync_panel__code'>{sent.code}</code>
						<Button kind="ghost" size="md" className='settings_location_row__delete' renderIcon={Copy} iconDescription="Copy the code" title="Copy the code" hasIconOnly onClick={() => void copyCode()} />
					</>
				)}
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || installedBuilds.length === 0} onClick={() => void sendTransfer(shareOptions)}>
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
		</>
	);

	const renderFile = () => (
		<>
			{renderWhatToShareRow()}
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
		</>
	);

	const shareDescription = (): string => {
		if (isStartingShare) {
			return "Reading the setup…";
		}
		const share = lan?.share;
		if (!share) {
			return "Other computers on this network can receive it with the PIN shown here";
		}
		const received = share.received_by.length > 0 ? ` · received by ${share.received_by.map((r) => r.device).join(", ")}` : "";
		return `${plural(share.versions, "Blender version", "Blender versions")}, ${plural(share.series, "series", "series")} · ${formatLanSize(share.size)}${received}`;
	};

	const peerDescription = (peer: ILanPeer): string => {
		const what = peer.share
			? `sharing ${plural(peer.share.versions, "Blender version", "Blender versions")}, ${plural(peer.share.series, "series", "series")} · ${formatLanSize(peer.share.size)}`
			: "not sharing a setup";
		return `${platformLabel(peer.platform)} · Blenderbase ${peer.app_version} · ${what}`;
	};

	const renderNetwork = () => {
		const peers = lan?.peers ?? [];
		return (
			<>
				{renderWhatToShareRow()}
				<Row id="lan-share" label="Share this computer's setup" description={shareDescription()}>
					{lan?.share && <code className='sync_panel__code'>{formatPin(lan.share.pin)}</code>}
					<Toggle
						id="lan-share"
						size="sm"
						hideLabel
						aria-labelledby="lan-share-label"
						toggled={Boolean(lan?.share)}
						disabled={isBusy || (!lan?.share && installedBuilds.length === 0)}
						onToggle={(checked) => void (checked ? shareLan(shareOptions) : stopLanShare())}
					/>
				</Row>
				{peers.map((peer) => (
					<Row key={peer.id} id={`lan-peer-${peer.id}`} label={peer.device} description={peerDescription(peer)}>
						{peer.share && (
							<>
								<TextInput
									id={`lan-pin-${peer.id}`}
									className='sync_panel__code_input sync_panel__code_input--pin'
									size="md"
									hideLabel
									labelText={`PIN shown on ${peer.device}`}
									placeholder="PIN"
									value={pinDrafts[peer.id] ?? ""}
									disabled={isBusy}
									onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPinDrafts({ ...pinDrafts, [peer.id]: e.target.value })}
									onKeyDown={(e: React.KeyboardEvent<HTMLInputElement>) => {
										if (e.key === "Enter" && (pinDrafts[peer.id] ?? "").trim().length > 0 && !isBusy) {
											void receiveLan(peer.id, pinDrafts[peer.id]);
										}
									}}
								/>
								<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || (pinDrafts[peer.id] ?? "").trim().length === 0} onClick={() => void receiveLan(peer.id, pinDrafts[peer.id])}>
									{isLanReceiving ? "Receiving…" : "Receive"}
								</Button>
							</>
						)}
					</Row>
				))}
				{peers.length === 0 && (
					<Row id="lan-empty" label="No other computer found yet" description="Open Sync › Local network there too, or turn on sharing there. The first time, Windows may ask to allow Blenderbase on the network">
						<InlineLoading className="sync_panel__looking" iconDescription="Looking" description="Looking…" />
					</Row>
				)}
			</>
		);
	};

	const renderSection = () => {
		switch (activeSection) {
			case 'folder':
				return renderFolder();
			case 'network':
				return renderNetwork();
			case 'transfer':
				return renderTransfer();
			case 'file':
				return renderFile();
		}
	};

	const section = SECTIONS.find((s) => s.id === activeSection) ?? SECTIONS[0];

	// The documentation link explains itself in the status line, like the title-bar buttons;
	// what was shown before the hover comes back on leave, unless something else posted meanwhile.
	const statusBeforeHint = useRef<{ message: string, isBusy: boolean, isError: boolean } | null>(null);
	const showDocumentationHint = () => {
		const s = useStatusStore.getState();
		if (s.isBusy) {
			return;
		}
		statusBeforeHint.current = { message: s.message, isBusy: s.isBusy, isError: s.isError };
		postStatus(DOCUMENTATION_HINT);
	};
	const hideDocumentationHint = () => {
		const before = statusBeforeHint.current;
		statusBeforeHint.current = null;
		if (before && useStatusStore.getState().message === DOCUMENTATION_HINT) {
			useStatusStore.getState().setStatus(before.message, before.isBusy, before.isError);
		}
	};

	return (
		<div className='settings_panel sync_panel'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title sync_panel__title_row'>
						Sync
						<a
							className='sync_panel__info'
							href={SYNC_DOCUMENTATION_URL}
							target='_blank'
							rel='noopener'
							aria-label='How syncing works (opens the documentation in your browser)'
							onMouseEnter={showDocumentationHint}
							onMouseLeave={hideDocumentationHint}
							onFocus={showDocumentationHint}
							onBlur={hideDocumentationHint}
						>
							<Information size={20} />
						</a>
					</span>
					<span className='column_header__subtitle'>Share your Blender setup across other computers</span>
				</div>
				{isBusy && <InlineLoading className="column_header__loading" iconDescription="Working" />}
			</div>
			<div className='column_actions settings_panel__toolbar sync_panel__ways'>
				<span className='sync_panel__ways_label' id="sync-ways-label">Share by:</span>
				<div className='build_type_switch' role="tablist" aria-labelledby="sync-ways-label">
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
			<p className='sync_panel__way'>{section.when}</p>
			<div className='list_header settings_panel__list_header'>
				<span>Action</span>
				<span></span>
			</div>
			<div className='settings_panel__list'>
				{renderSection()}
			</div>
			<p className='sync_panel__note'>{AFTER_RECEIVING}</p>
		</div>
	)
}

export default SyncPanel

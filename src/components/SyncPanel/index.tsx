import { useEffect, useState } from 'react';
import { Button, InlineLoading, TextInput, Toggle } from '@carbon/react';
import { ArrowLeft, Copy, Laptop, TrashCan } from '@carbon/react/icons';
import { open } from '@tauri-apps/plugin-dialog';
import { useShallow } from 'zustand/react/shallow';
import { ILanPeer } from '../../models';
import { SETUP_FILE_FILTER, SYNC_DOCUMENTATION_URL } from '../../constants';
import { useBlenderManagerStore } from '../../store/blenderManagerStore';
import { useSetupRestoreStore } from '../../store/setupRestoreStore';
import { describeSyncFile, useSetupSyncStore } from '../../store/setupSyncStore';
import { formatLanSize, formatPin, platformLabel, useSetupLanStore } from '../../store/setupLanStore';
import { SyncSection, useUiControlsStore } from '../../store/uiControlsStore';
import { postStatus, postStatusError } from '../../store/statusStore';
import DocumentationLink from '../DocumentationLink';

/**
 * One way to share a setup: the tab label and the line under the tabs that says when it fits.
 * In order of preference: the first tab is the one that opens.
 */
const SECTIONS: { id: SyncSection, label: string, when: string }[] = [
	{ id: 'network', label: 'Local network', when: 'Use Local network if your computers are on the same network. No internet needed' },
	{ id: 'transfer', label: 'Transfer code', when: 'Use Transfer code if your computers are outside the local network. Requires an internet connection' },
	{ id: 'folder', label: 'Sync folder', when: 'Use Sync folder if your computers share a cloud drive such as Dropbox, OneDrive, iCloud or Google Drive' },
	{ id: 'file', label: 'Setup file', when: 'Use Setup file if you want to carry the setup yourself, on a USB stick, in an email or on any drive' },
];

const errorText = (e: unknown): string => (e instanceof Error ? e.message : String(e)).replace(/^cmd_\w+: /, "");


const plural = (count: number, one: string, many: string): string => `${count} ${count === 1 ? one : many}`;


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
	const { activeSection, setActiveSection, openShareSetup, setIsSyncOpen } = useUiControlsStore(
		useShallow((s) => ({ activeSection: s.syncSection, setActiveSection: s.setSyncSection, openShareSetup: s.openShareSetup, setIsSyncOpen: s.setIsSyncOpen }))
	)
	const installedBuilds = useBlenderManagerStore((s) => s.installedBuilds)
	const openSetup = useSetupRestoreStore((s) => s.open)
	const { syncStatus, isSyncBusy, loadSync, setSyncFolder, openSyncFile } = useSetupSyncStore(
		useShallow((s) => ({ syncStatus: s.status, isSyncBusy: s.isBusy, loadSync: s.load, setSyncFolder: s.setFolder, openSyncFile: s.openForApply }))
	)
	const { sent, isSending, isReceiving, isSavingFile, receiveTransfer } = useSetupSyncStore(
		useShallow((s) => ({ sent: s.sent, isSending: s.isSending, isReceiving: s.isReceiving, isSavingFile: s.isSavingFile, receiveTransfer: s.receive }))
	)
	const [codeDraft, setCodeDraft] = useState<string>("")
	const { lan, isStartingShare, isLanReceiving, refreshLan, browseLan, stopLanShare, receiveLan } = useSetupLanStore(
		useShallow((s) => ({ lan: s.status, isStartingShare: s.isStartingShare, isLanReceiving: s.isReceiving, refreshLan: s.refresh, browseLan: s.browse, stopLanShare: s.stopShare, receiveLan: s.receive }))
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

	const isBusy = isSyncBusy || isSavingFile || isSending || isReceiving || isStartingShare || isLanReceiving;

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
			<Row id="sync-save" label="Save this computer's setup" description={folderSet ? "Writes the setup to the folder; other computers are told it is newer" : "Choose a folder first"} inactive={!folderSet}>
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || !folderSet || installedBuilds.length === 0} onClick={() => openShareSetup('folder')}>
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
			<Row id="sync-transfer-send" label={sent ? "Your transfer code" : "Send to another computer"} description={sentDescription()}>
				{sent && (
					<>
						<code className='sync_panel__code'>{sent.code}</code>
						<Button kind="ghost" size="md" className='settings_location_row__delete' renderIcon={Copy} iconDescription="Copy the code" title="Copy the code" hasIconOnly onClick={() => void copyCode()} />
					</>
				)}
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || installedBuilds.length === 0} onClick={() => openShareSetup('transfer')}>
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
			<Row id="sync-file-save" label="Save setup to a file" description="Blender versions, preferences, theme, keymaps and the addon list of every series">
				<Button kind="tertiary" size="md" className='settings_row__button' disabled={isBusy || installedBuilds.length === 0} onClick={() => openShareSetup('file')}>
					{isSavingFile ? "Saving…" : "Save…"}
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
				<Row id="lan-share" label="Share this computer's setup" description={shareDescription()}>
					{lan?.share && <code className='sync_panel__code'>{formatPin(lan.share.pin)}</code>}
					<Toggle
						id="lan-share"
						size="sm"
						hideLabel
						aria-labelledby="lan-share-label"
						toggled={Boolean(lan?.share)}
						disabled={isBusy || (!lan?.share && installedBuilds.length === 0)}
						onToggle={(checked) => (checked ? openShareSetup('network') : void stopLanShare())}
					/>
				</Row>
				<div className='sync_panel__peers'>
					<div className='sync_panel__peers_header'>
						<span>Computers on this network</span>
						{peers.length === 0 && <InlineLoading className="sync_panel__looking" iconDescription="Looking" description="Looking…" />}
					</div>
					{peers.map((peer) => (
					<div key={peer.id} className='settings_row sync_panel__peer'>
						<div className='settings_row__main sync_panel__peer_main'>
							<Laptop size={20} className='sync_panel__peer_icon' aria-hidden="true" />
							<div className='sync_panel__peer_text'>
								<span className='settings_row__label' id={`lan-peer-${peer.id}-label`}>{peer.device}</span>
								<span className='settings_row__description' title={peerDescription(peer)}>{peerDescription(peer)}</span>
							</div>
						</div>
						<div className='settings_row__control'>
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
						</div>
					</div>
					))}
					{peers.length === 0 && (
						<div className='sync_panel__peers_empty'>
							No other computer found yet. Open Sync › Local network there too, or turn on sharing there. The first time, Windows may ask to allow Blenderbase on the network.
						</div>
					)}
				</div>
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

	return (
		<div className='settings_panel sync_panel'>
			<div className='column_header'>
				<div className='column_header__titles'>
					<span className='column_header__title column_header__title_row'>
						Sync
						<DocumentationLink href={SYNC_DOCUMENTATION_URL} hint="How syncing works · opens the documentation in your browser" />
					</span>
					<span className='column_header__subtitle'>Share your Blender setup across other computers</span>
				</div>
				{isBusy && <InlineLoading className="column_header__loading" iconDescription="Working" />}
				<Button kind="ghost" size="lg" className='column_header__back' title="Back to addons" onClick={() => setIsSyncOpen(false)}>
					<ArrowLeft /> Back to Addons
				</Button>
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
			{/* When the selected way fits, in the list header row: the row lines stay on the grid every column shares. */}
			<div className='list_header settings_panel__list_header'>
				<span className='sync_panel__way' title={section.when}>{section.when}</span>
				<span></span>
			</div>
			<div className='settings_panel__list'>
				{renderSection()}
			</div>
		</div>
	)
}

export default SyncPanel

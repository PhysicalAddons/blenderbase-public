import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useBlenderManagerStore } from '../store/blenderManagerStore';
import { useAddonStore } from '../store/addonStore';
import { useUiControlsStore } from '../store/uiControlsStore';
import { useStatusStore } from '../store/statusStore';
import { resolveSelectedBlenderVersion } from '.';

/** Installed versions are rescanned at most this often on refocus; alt-tabbing back and forth stays free. */
const VERSIONS_RESCAN_INTERVAL_MS = 15_000;
const RESCAN_MESSAGE = "Rescanning installed Blender versions…";

/**
 * Keeps the lists current without a refresh button: when the window regains focus,
 * installed Blender versions are rescanned (cheap: folders and metadata files), and the
 * selected version's addons are re-read if that version was launched from Blenderbase
 * since the last read (that is when Preferences changes happen; the read runs Blender
 * headlessly, so it is not done on every focus).
 */
export const useRescanOnFocus = (): void => {
    useEffect(() => {
        // The startup scan has just run; a refocus within the interval is skipped.
        let lastVersionsScan = Date.now();
        let running = false;

        const rescan = async () => {
            if (running) {
                return;
            }
            running = true;
            try {
                const status = useStatusStore.getState();
                // Something else is in flight (download, install, dialog): leave it alone.
                if (!status.isBusy && Date.now() - lastVersionsScan >= VERSIONS_RESCAN_INTERVAL_MS) {
                    lastVersionsScan = Date.now();
                    const before = { message: status.message, isError: status.isError };
                    status.setStatus(RESCAN_MESSAGE, true);
                    try {
                        await useBlenderManagerStore.getState().refreshInstalledBuilds();
                    } catch (e) {
                        console.error(e);
                    }
                    // Restore what was shown before, unless something else has posted meanwhile.
                    if (useStatusStore.getState().message === RESCAN_MESSAGE) {
                        useStatusStore.getState().setStatus(before.message, false, before.isError);
                    }
                }
                const { installedBuilds } = useBlenderManagerStore.getState();
                const { selectedBlenderVersionId } = useUiControlsStore.getState();
                const selected = resolveSelectedBlenderVersion(installedBuilds, selectedBlenderVersionId);
                const addons = useAddonStore.getState();
                if (selected && addons.launchedSinceReadIds.includes(selected.id) && !addons.isBusy) {
                    await addons.refreshAddons(selected.id);
                }
            } finally {
                running = false;
            }
        };

        const unlisten = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
            if (focused) {
                rescan().catch((e) => console.error(e));
            }
        });
        return () => {
            unlisten.then((off) => off()).catch((e) => console.error(e));
        };
    }, []);
};

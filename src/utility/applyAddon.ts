import { ask } from '@tauri-apps/plugin-dialog';
import { IAddon, IBlenderVersion } from '../models';
import { useAddonStore } from '../store/addonStore';
import { useBlenderManagerStore } from '../store/blenderManagerStore';
import { postStatus, useStatusStore } from '../store/statusStore';
import { addonLabel, applyTargetOf, blenderVersionLabel } from '.';

/** The drag payload type of an addon row. Only its presence matters: the addon store carries the addon. */
export const ADDON_DRAG_TYPE = 'application/x-blenderbase-addon';

/**
 * Applies an addon to the series of `target`: its files are copied there (a symlinked addon
 * is linked to the same source folder) and it is enabled by that build. When the series has
 * the addon already, the user is asked before it is replaced.
 */
export const applyAddonToVersion = async (addon: IAddon, target: IBlenderVersion): Promise<void> => {
    const outcome = await useAddonStore.getState().applyAddon(addon, target, false);
    if (outcome !== 'exists') {
        return;
    }
    const { installedBuilds } = useBlenderManagerStore.getState();
    const group = applyTargetOf(installedBuilds, target);
    const source = installedBuilds.find((x) => x.id === addon.parent_blender_version_id);
    const from = source ? ` from Blender ${blenderVersionLabel(source)}` : '';
    const confirmed = await ask(
        `${group.label} already has ${addonLabel(addon)}. Replace it with the copy${from}?`,
        { title: 'Replace addon', kind: 'warning', okLabel: 'Replace', cancelLabel: 'Cancel' }
    );
    if (confirmed) {
        await useAddonStore.getState().applyAddon(addon, target, true);
    } else {
        postStatus(`Kept the ${addonLabel(addon)} already in ${group.label}`);
    }
};

// Status-line hints while an addon is dragged. The line shown before the drag comes back
// when the drag ends without a drop; a drop posts its own status, which stays.
let statusBeforeDrag: { message: string, isBusy: boolean, isError: boolean } | null = null;
let lastDragHint: string | null = null;

/** Shows a hint for the drag in progress; repeating the current hint is free. */
export const postDragHint = (text: string): void => {
    const s = useStatusStore.getState();
    if (statusBeforeDrag === null) {
        statusBeforeDrag = { message: s.message, isBusy: s.isBusy, isError: s.isError };
    }
    if (lastDragHint !== text) {
        lastDragHint = text;
        postStatus(text);
    }
};

/** Ends the drag's hints: unless something else has posted meanwhile, the previous line returns. */
export const clearDragHints = (): void => {
    const before = statusBeforeDrag;
    const hint = lastDragHint;
    statusBeforeDrag = null;
    lastDragHint = null;
    if (before && hint !== null && useStatusStore.getState().message === hint) {
        useStatusStore.getState().setStatus(before.message, before.isBusy, before.isError);
    }
};

/** The hint at the start of a drag, and while the pointer is over nothing that takes the drop. */
export const dragStartHint = (addon: IAddon, hasTargets: boolean): string =>
    hasTargets
        ? `Drop ${addonLabel(addon)} on a Blender version to apply it there`
        : `No other Blender series is installed to apply ${addonLabel(addon)} to`;

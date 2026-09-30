import type { MouseEvent } from 'react';
import { Menu, type MenuOptions } from '@tauri-apps/api/menu';

export interface IContextMenuItem {
    text: string,
    action: () => void,
    /** False greys the entry out, for a choice with nothing to act on right now. */
    enabled?: boolean,
}

export interface IContextMenuSubmenu {
    text: string,
    items: IContextMenuEntry[],
    enabled?: boolean,
}

export interface IContextMenuSeparator {
    separator: true,
}

export type IContextMenuEntry = IContextMenuItem | IContextMenuSubmenu | IContextMenuSeparator;

type MenuItems = NonNullable<MenuOptions['items']>;

const toMenuItems = (entries: IContextMenuEntry[], prefix: string): MenuItems =>
    entries.map((entry, index): MenuItems[number] => {
        const id = `${prefix}-${index}`;
        if ('separator' in entry) {
            return { item: 'Separator' };
        }
        if ('items' in entry) {
            return { id, text: entry.text, enabled: entry.enabled ?? true, items: toMenuItems(entry.items, id) };
        }
        return { id, text: entry.text, enabled: entry.enabled ?? true, action: entry.action };
    });

/**
 * Shows a native right-click menu at the cursor. Call from an `onContextMenu`
 * handler; the browser's own menu is suppressed. An entry is an item, a
 * submenu (`items`) or a separator.
 */
export const showContextMenu = async (e: MouseEvent, entries: IContextMenuEntry[]): Promise<void> => {
    e.preventDefault();
    try {
        const menu = await Menu.new({ items: toMenuItems(entries, 'ctx') });
        await menu.popup();
    } catch (err) {
        console.error(err);
    }
};

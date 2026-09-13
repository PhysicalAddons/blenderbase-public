import type { MouseEvent } from 'react';
import { Menu } from '@tauri-apps/api/menu';

export interface IContextMenuItem {
    text: string,
    action: () => void,
}

/**
 * Shows a native right-click menu at the cursor. Call from an `onContextMenu`
 * handler; the browser's own menu is suppressed.
 */
export const showContextMenu = async (e: MouseEvent, items: IContextMenuItem[]): Promise<void> => {
    e.preventDefault();
    try {
        const menu = await Menu.new({
            items: items.map((item, index) => ({
                id: `ctx-${index}-${item.text}`,
                text: item.text,
                action: item.action,
            })),
        });
        await menu.popup();
    } catch (err) {
        console.error(err);
    }
};

import { RefObject, useEffect } from "react";

/**
 * Makes a scrollable list page by a fraction of its own visible height per mouse-wheel notch
 * instead of the browser's small default step, so scrolling feels column-based.
 *
 * @param ref       the scrolling element
 * @param fraction  share of the visible height to move per notch (0.5 = half a column)
 */
export const usePagedScroll = (ref: RefObject<HTMLElement | null>, fraction: number = 0.5) => {
    useEffect(() => {
        const el = ref.current;
        if (!el) {
            return;
        }
        const onWheel = (e: WheelEvent) => {
            if (e.deltaY === 0 || e.ctrlKey) {
                return;
            }
            const canScroll = el.scrollHeight > el.clientHeight;
            if (!canScroll) {
                return;
            }
            e.preventDefault();
            const step = Math.max(1, Math.round(el.clientHeight * fraction));
            el.scrollBy({ top: Math.sign(e.deltaY) * step, behavior: "smooth" });
        };
        // Non-passive so the default step can be suppressed.
        el.addEventListener("wheel", onWheel, { passive: false });
        return () => el.removeEventListener("wheel", onWheel);
    }, [ref, fraction]);
}

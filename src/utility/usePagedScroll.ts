import { RefObject, useEffect } from "react";

export interface PagedScrollOptions {
    /** Selector, inside the scrolling element, of the rows whose top edges form the grid. */
    rowSelector: string,
    /** Pixels one mouse-wheel notch moves before snapping; one row of the three-column grid. */
    stepPx?: number,
}

/**
 * Smooth scrolling that always comes to rest on a row boundary.
 *
 * A mouse-wheel notch moves one grid step and snaps to the nearest row edge; trackpad
 * deltas accumulate until they amount to a step, so a flick still moves whole rows.
 * Scrolling by other means (scrollbar drag, keyboard, touch) is left alone while it
 * happens and snapped to the nearest row once it ends, so the lists of every column
 * stay on the same 56px rhythm.
 */
export const usePagedScroll = (
    ref: RefObject<HTMLElement | null>,
    { rowSelector, stepPx = 56 }: PagedScrollOptions,
) => {
    useEffect(() => {
        const el = ref.current;
        if (!el) {
            return;
        }
        let pending: number | null = null; // target of the animation in flight
        let accumulated = 0;               // trackpad delta not yet turned into a step
        let ours = false;                  // the scroll now animating was started here

        const boundaries = (): number[] => {
            const origin = el.getBoundingClientRect().top - el.scrollTop;
            return [...el.querySelectorAll<HTMLElement>(rowSelector)]
                .map((row) => Math.round(row.getBoundingClientRect().top - origin));
        };
        const maxScroll = () => el.scrollHeight - el.clientHeight;
        const clamp = (y: number) => Math.max(0, Math.min(maxScroll(), y));
        const nearest = (y: number): number => {
            const edges = boundaries();
            if (edges.length === 0) {
                return y;
            }
            return edges.reduce((best, edge) => (Math.abs(edge - y) < Math.abs(best - y) ? edge : best));
        };
        const scrollTo = (y: number) => {
            pending = y;
            ours = true;
            el.scrollTo({ top: y, behavior: "smooth" });
        };

        const onWheel = (e: WheelEvent) => {
            if (e.deltaY === 0 || e.ctrlKey) {
                return;
            }
            if (el.scrollHeight <= el.clientHeight) {
                return;
            }
            e.preventDefault();
            // A mouse notch is about 100px; trackpads send many small deltas that add up.
            accumulated += e.deltaY;
            if (Math.abs(accumulated) < 40) {
                return;
            }
            const direction = Math.sign(accumulated);
            accumulated = 0;
            const base = pending ?? el.scrollTop;
            let target = clamp(nearest(base + direction * stepPx));
            // Never stall on the row we are already on: move one step and snap from there.
            if (Math.abs(target - base) < 1) {
                target = clamp(nearest(base + direction * stepPx * 1.5));
            }
            if (Math.abs(target - base) >= 1) {
                scrollTo(target);
            }
        };

        const onScrollEnd = () => {
            if (ours) {
                ours = false;
                pending = null;
                return;
            }
            // Scrollbar, keyboard or touch: settle on the nearest row unless the list end was reached.
            const y = el.scrollTop;
            const snapped = clamp(nearest(y));
            if (Math.abs(snapped - y) > 1 && y < maxScroll() - 1) {
                scrollTo(snapped);
            }
        };

        // Non-passive so the default wheel step can be suppressed.
        el.addEventListener("wheel", onWheel, { passive: false });
        el.addEventListener("scrollend", onScrollEnd);
        return () => {
            el.removeEventListener("wheel", onWheel);
            el.removeEventListener("scrollend", onScrollEnd);
        };
    }, [ref, rowSelector, stepPx]);
}

import { useEffect, type RefObject } from 'react';

/** While `open`, a press anywhere outside `ref` calls `close`, as a menu closes. */
export function useOutsideClose(ref: RefObject<HTMLElement | null>, open: boolean, close: () => void): void {
  useEffect(() => {
    if (!open) return;
    const onDown = (event: MouseEvent) => {
      if (!ref.current?.contains(event.target as Node)) close();
    };
    document.addEventListener('mousedown', onDown);
    return () => document.removeEventListener('mousedown', onDown);
  }, [ref, open, close]);
}

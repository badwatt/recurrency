import { useEffect, useRef } from "react";

interface SwipeOptions {
  onOpen: () => void;
  onClose: () => void;
  isOpen: boolean;
  edgeWidth?: number;
}

export function useSwipeSidebar({ onOpen, onClose, isOpen, edgeWidth = 30 }: SwipeOptions) {
  const startX = useRef(0);
  const startY = useRef(0);
  const tracking = useRef(false);

  useEffect(() => {
    const handleTouchStart = (e: TouchEvent) => {
      const touch = e.touches[0];
      startX.current = touch.clientX;
      startY.current = touch.clientY;
      tracking.current = touch.clientX < edgeWidth || isOpen;
    };

    const handleTouchMove = (e: TouchEvent) => {
      if (!tracking.current) return;

      const touch = e.touches[0];
      const diffX = touch.clientX - startX.current;
      const diffY = Math.abs(touch.clientY - startY.current);

      // Only intercept if horizontal swipe dominates
      if (Math.abs(diffX) > diffY && Math.abs(diffX) > 10) {
        e.preventDefault();
      }
    };

    const handleTouchEnd = (e: TouchEvent) => {
      if (!tracking.current) return;

      const touch = e.changedTouches[0];
      const diffX = touch.clientX - startX.current;
      const diffY = Math.abs(touch.clientY - startY.current);

      // Only trigger if horizontal movement was significant
      if (Math.abs(diffX) > diffY && Math.abs(diffX) > 50) {
        if (isOpen && diffX < -50) {
          onClose();
        } else if (!isOpen && diffX > 50) {
          onOpen();
        }
      }

      tracking.current = false;
    };

    document.addEventListener("touchstart", handleTouchStart, { passive: true });
    document.addEventListener("touchmove", handleTouchMove, { passive: false });
    document.addEventListener("touchend", handleTouchEnd, { passive: true });

    return () => {
      document.removeEventListener("touchstart", handleTouchStart);
      document.removeEventListener("touchmove", handleTouchMove);
      document.removeEventListener("touchend", handleTouchEnd);
    };
  }, [isOpen, onOpen, onClose, edgeWidth]);

  return null;
}
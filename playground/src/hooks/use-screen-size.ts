import { useLayoutEffect, useState } from "react";
import type { ScreenSizeType } from "@/types";

function getScreenSize(width: number): ScreenSizeType {
  if (width >= 960) return "wide";
  return "thin";
}

export function useScreenSize(): ScreenSizeType {
  // Initialize with current window size (SSR-safe)
  const [screenSize, setScreenSize] = useState<ScreenSizeType>(() => {
    // Check if we're in browser environment
    if (typeof window === "undefined") return "wide"; // Default for SSR
    return getScreenSize(window.innerWidth);
  });

  useLayoutEffect(() => {
    // Early return if not in browser
    if (typeof window === "undefined") return;

    const updateScreenSize = () => {
      const newSize = getScreenSize(window.innerWidth);
      setScreenSize((prevSize) => (prevSize === newSize ? prevSize : newSize));
    };

    // Set initial size
    updateScreenSize();

    // Add resize listener
    window.addEventListener("resize", updateScreenSize);

    // Cleanup
    return () => window.removeEventListener("resize", updateScreenSize);
  }, []);

  return screenSize;
}

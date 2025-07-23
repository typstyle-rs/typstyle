import { useCallback, useEffect, useState } from "react";
import type { ThemeType } from "@/types";
import { ThemeContext } from "./theme-context";

interface ThemeProviderProps {
  children: React.ReactNode;
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const THEME_STORE_KEY = "playground-theme";

  // Helper function to get system theme preference
  const getSystemTheme = (): ThemeType => {
    if (typeof window !== "undefined" && window.matchMedia) {
      return window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
    }
    return "light"; // fallback for SSR or older browsers
  };

  // Initialize theme with saved preference, defaulting to system preference
  const [theme, setTheme] = useState<ThemeType>(() => {
    if (typeof window === "undefined") {
      return "light"; // SSR fallback
    }

    const savedTheme = localStorage.getItem(
      THEME_STORE_KEY,
    ) as ThemeType | null;

    if (savedTheme && (savedTheme === "light" || savedTheme === "dark")) {
      // If we have a saved theme preference, use it
      return savedTheme;
    }

    // Otherwise, use system preference
    return getSystemTheme();
  });

  const toggleTheme = useCallback(() => {
    setTheme((prev) => (prev === "light" ? "dark" : "light"));
  }, []);

  // Apply theme to document root and save to localStorage
  useEffect(() => {
    const root = document.documentElement;
    root.setAttribute("data-theme", theme);
    localStorage.setItem(THEME_STORE_KEY, theme);
  }, [theme]);

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}

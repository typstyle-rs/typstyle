import { useTheme } from "@/hooks";
import { SampleDocumentSelector } from "./forms/SampleDocumentSelector";
import { DarkModeIcon, GitHubIcon, LightModeIcon, ShareIcon } from "./ui/Icons";

interface HeaderProps {
  onSampleSelect: (content: string) => void;
  onShareClick: () => void;
  isGeneratingShare?: boolean;
}

export function Header({
  onSampleSelect,
  onShareClick,
  isGeneratingShare = false,
}: HeaderProps) {
  const { theme, toggleTheme } = useTheme();
  return (
    <div className="navbar min-h-12 bg-base-200 shadow">
      <div className="flex-none ml-2">
        <h1 className="text-xl font-bold text-neutral m-1 drop-shadow">
          Typstyle Playground
        </h1>
      </div>

      <div className="flex-1 ml-4">
        <SampleDocumentSelector onSampleSelect={onSampleSelect} />
      </div>

      <div className="flex-none mr-2">
        {/* Share Link */}
        <button
          type="button"
          onClick={onShareClick}
          className="btn btn-ghost btn-circle"
          title="Share this playground"
          disabled={isGeneratingShare}
        >
          {isGeneratingShare ? (
            <span className="loading loading-spinner loading-sm"></span>
          ) : (
            <ShareIcon />
          )}
        </button>

        {/* GitHub Repo Link */}
        <a
          href="https://github.com/typstyle-rs/typstyle"
          target="_blank"
          rel="noopener noreferrer"
          className="btn btn-ghost btn-circle"
          title="View Typstyle on GitHub"
        >
          <GitHubIcon />
        </a>

        {/* Theme Toggle */}
        <button
          type="button"
          onClick={toggleTheme}
          className="btn btn-ghost btn-circle"
          title={`Switch to ${theme === "light" ? "dark" : "light"} mode`}
        >
          {theme === "light" ? <LightModeIcon /> : <DarkModeIcon />}
        </button>
      </div>
    </div>
  );
}

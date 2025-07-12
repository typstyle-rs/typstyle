import { useCallback, useState } from "react";
import type { PlaygroundState } from "@/utils";
import { copyToClipboard, generateShareUrl } from "@/utils";
import { useToast } from ".";

export interface ShareState {
  isOpen: boolean;
  isGenerating: boolean;
  url: string;
  usedPastebin: boolean;
  copied: boolean;
  error: string | null;
}

const initialState: ShareState = {
  isOpen: false,
  isGenerating: false,
  url: "",
  usedPastebin: false,
  copied: false,
  error: null,
};

export function useShareManager() {
  const [shareState, setShareState] = useState<ShareState>(initialState);
  const toast = useToast();

  const generateShare = useCallback(
    async (playgroundState: PlaygroundState) => {
      setShareState((prev) => ({
        ...prev,
        isGenerating: true,
        error: null,
        copied: false,
      }));

      try {
        const result = await generateShareUrl(playgroundState);
        setShareState((prev) => ({
          ...prev,
          isGenerating: false,
          isOpen: true,
          url: result.url,
          usedPastebin: result.usedPastebin,
        }));

        if (result.usedPastebin) {
          toast.success(
            "Share link generated using pastebin for large content",
          );
        } else {
          toast.success("Share link generated successfully");
        }
      } catch (error) {
        console.error("Error generating share URL:", error);
        const errorMessage =
          "Failed to generate share link. Using current URL as fallback.";
        setShareState((prev) => ({
          ...prev,
          isGenerating: false,
          isOpen: true,
          url: window.location.href,
          usedPastebin: false,
          error: errorMessage,
        }));
        toast.error(errorMessage);
      }
    },
    [toast],
  );

  const copyShareUrl = useCallback(async () => {
    if (!shareState.url) return false;

    const success = await copyToClipboard(shareState.url);
    if (success) {
      setShareState((prev) => ({ ...prev, copied: true }));
      toast.success("Share link copied to clipboard");
      // Reset copied state after 2 seconds
      setTimeout(() => {
        setShareState((prev) => ({ ...prev, copied: false }));
      }, 2000);
    } else {
      toast.error("Failed to copy link to clipboard");
    }
    return success;
  }, [shareState.url, toast]);

  const closeModal = useCallback(() => {
    setShareState((prev) => ({
      ...prev,
      isOpen: false,
      copied: false,
      error: null,
    }));
  }, []);

  const resetError = useCallback(() => {
    setShareState((prev) => ({ ...prev, error: null }));
  }, []);

  return {
    shareState,
    toasts: toast.toasts,
    dismissToast: toast.dismissToast,
    generateShare,
    copyShareUrl,
    closeModal,
    resetError,
  };
}

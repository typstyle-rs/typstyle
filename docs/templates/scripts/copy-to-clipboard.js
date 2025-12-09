/**
 * Copy button functionality for hypraw code blocks
 */

(() => {
  /**
   * Copy text to clipboard using the modern Clipboard API with fallback
   * @param {string} text - Text to copy
   * @returns {Promise<boolean>} - Success status
   */
  async function copyToClipboard(text) {
    if (!(!navigator.clipboard || !window.isSecureContext)) {
      try {
        await navigator.clipboard.writeText(text);
        return true;
      } catch (err) {
        console.warn("Clipboard API failed", err);
      }
    }
  }

  /**
   * Handle copy button click
   * @param {Event} event - Click event
   */
  async function handleCopyClick(event) {
    const button = event.target;
    const textToCopy = button.getAttribute("data-copy");

    if (!textToCopy) {
      console.warn("No text to copy found in data-copy attribute");
      return;
    }

    const success = await copyToClipboard(textToCopy);

    if (success) {
      // Visual feedback
      button.classList.add("copied");
      button.setAttribute("aria-label", "Copied!");

      // Reset after 2 seconds
      setTimeout(() => {
        button.classList.remove("copied");
        button.setAttribute("aria-label", "Copy code");
      }, 2000);
    } else {
      // Error feedback
      button.setAttribute("aria-label", "Copy failed");
      setTimeout(() => {
        button.setAttribute("aria-label", "Copy code");
      }, 2000);
    }
  }

  /**
   * Initialize copy buttons when DOM is ready
   */
  function initializeCopyButtons() {
    const copyButtons = document.querySelectorAll(".hypraw-copy-btn");

    copyButtons.forEach((button) => {
      button.addEventListener("click", handleCopyClick);
    });
  }

  // Initialize when DOM is ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initializeCopyButtons);
  } else {
    initializeCopyButtons();
  }
})();

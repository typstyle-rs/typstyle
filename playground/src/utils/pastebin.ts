// shz.al API base URL
const PASTEBIN_API = "https://shz.al";

/**
 * Upload content to shz.al pastebin
 */
export async function uploadToPastebin(
  content: string,
): Promise<string | null> {
  try {
    const formData = new FormData();
    formData.append("c", content);

    const response = await fetch(PASTEBIN_API, {
      method: "POST",
      body: formData,
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const result = await response.json();
    if (result.url) {
      // Extract the paste ID from the URL (e.g., "https://shz.al/abcd" -> "abcd")
      return (result.url as string).split("/").pop() ?? null;
    }

    return null;
  } catch (error) {
    console.error("Error uploading to pastebin:", error);
    return null;
  }
}

/**
 * Fetch content from shz.al pastebin
 */
export async function fetchFromPastebin(
  pasteId: string,
): Promise<string | null> {
  try {
    const response = await fetch(`${PASTEBIN_API}/${pasteId}`);

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return await response.text();
  } catch (error) {
    console.error("Error fetching from pastebin:", error);
    return null;
  }
}

import sampleManifest from "@/assets/sample-manifest.json";

export interface SampleManifest {
  samples: Sample[];
  default: string;
}

export interface Sample {
  id: string;
  name: string;
}

export const SAMPLE_MANIFEST: SampleManifest = sampleManifest;

export const SAMPLES: { [key: string]: Sample } = Object.fromEntries(
  sampleManifest.samples.map((it) => [it.name, it]),
);

interface LoadSampleOptions {
  onSuccess: (content: string) => void;
  onError?: (error: string) => void;
}

const fetchSampleDocument = async (id: string): Promise<string> => {
  const fetchUrl = `${import.meta.env.BASE_URL}samples/${id}.typ`;
  const response = await fetch(fetchUrl);
  if (!response.ok) {
    throw new Error(
      `Failed to load sample: ${response.status} ${response.statusText} from ${id}.typ`,
    );
  }
  return response.text();
};

const getSampleFileContent = async (
  sampleKey: keyof typeof SAMPLES,
): Promise<string> => {
  const sample = SAMPLES[sampleKey];
  if (!sample) {
    throw new Error(`Sample with key "${sampleKey}" not found.`);
  }
  return fetchSampleDocument(sample.id);
};

const getFallbackContent = (
  sampleKey: string,
  error: Error | string,
): string => {
  const errorMessage = error instanceof Error ? error.message : error;
  const sampleName = SAMPLES[sampleKey]?.name ?? "the requested sample";

  return `= Sample Document Loading Error

Failed to load: ${sampleName}
Error: ${errorMessage}

== Available Sample Documents
You can try selecting a different sample document from the dropdown.

${Object.values(SAMPLES)
  .map((doc) => `- ${doc.name}: ${doc.id}.typ`)
  .join("\n")}

Try refreshing the page or check your internet connection.`;
};

export const loadSample = async (
  sampleKey: string,
  { onSuccess, onError }: LoadSampleOptions,
): Promise<void> => {
  try {
    if (!(sampleKey in SAMPLES)) {
      sampleKey = sampleManifest.default;
    }
    const content = await getSampleFileContent(sampleKey);
    onSuccess(content);
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : "Unknown error";
    console.error(`Failed to load ${sampleKey} sample:`, error);

    onError?.(errorMessage);

    // Always provide fallback content
    const fallback = getFallbackContent(sampleKey, error as Error);
    onSuccess(fallback);
  }
};

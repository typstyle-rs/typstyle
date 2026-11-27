export * from "../generated/types";

import type { OptionKey, Options, PartialOptions } from "../generated/types";
import defaults from "../generated/defaults.json";
import schema from "../generated/schema.json";
import uiSchema from "../generated/ui-schema.json";

/** Resolved built-in defaults, generated from the Rust `Options::default`. */
export const DEFAULT_OPTIONS = defaults as Options;

/** Backwards-compatible alias for [`DEFAULT_OPTIONS`]. */
export const DEFAULT_CONFIG = DEFAULT_OPTIONS;

/** JSON Schema for the resolved formatter configuration. */
export interface OptionsSchemaProperty {
  type: "boolean" | "integer" | "number" | "string";
  description: string;
  default?: unknown;
  minimum?: number;
  maximum?: number;
  enum?: string[];
}

/** JSON Schema for the resolved formatter configuration. */
export interface OptionsSchema {
  $schema: string;
  title: string;
  type: "object";
  additionalProperties: false;
  properties: Record<OptionKey, OptionsSchemaProperty>;
}

/** Rendering metadata kept separate from the configuration schema. */
export interface OptionsUiOption {
  label: string;
  description: string;
  group: string;
  widget: "checkbox" | "number" | "slider" | "select" | "text" | string;
  order: number;
  default?: unknown;
}

/** Rendering metadata kept separate from the configuration schema. */
export interface OptionsUiSchema {
  groups: { name: string; options: OptionKey[] }[];
  options: Record<OptionKey, OptionsUiOption>;
}

/** Schema describing the resolved formatter configuration. */
export const OPTIONS_SCHEMA = schema as OptionsSchema;

/** Rendering hints, grouped and ordered for generic editors. */
export const OPTIONS_UI_SCHEMA = uiSchema as OptionsUiSchema;

/** Every canonical option key, in declaration order. */
export const OPTION_KEYS = OPTIONS_UI_SCHEMA.groups.flatMap(
  (group) => group.options,
);

/** Resolves explicit overrides against the built-in defaults. */
export function resolveOptions(overrides: PartialOptions): Options {
  return { ...DEFAULT_OPTIONS, ...overrides };
}

/** Backwards-compatible alias for [`resolveOptions`]. */
export const resolveConfig = resolveOptions;

/** Narrows an arbitrary string to a canonical option key. */
export function isOptionKey(key: string): key is OptionKey {
  return key in DEFAULT_OPTIONS;
}

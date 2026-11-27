import {
  DEFAULT_OPTIONS,
  OPTIONS_SCHEMA,
  OPTIONS_UI_SCHEMA,
  type OptionKey,
  type Options,
  type OptionsSchema,
  type OptionsUiOption,
  type OptionsUiSchema,
  type PartialOptions,
  resolveOptions,
} from "@typstyle/options";
import { type ComponentType, useId, useMemo } from "react";

/** Props passed to a custom option control. */
export interface OptionControlProps {
  option: OptionsUiOption;
  value: unknown;
  overridden: boolean;
  onChange: (value: unknown) => void;
  onReset: () => void;
}

export interface OptionsEditorProps {
  value: PartialOptions;
  onChange: (value: PartialOptions) => void;
  schema?: OptionsSchema;
  uiSchema?: OptionsUiSchema;
  defaults?: Options;
  renderers?: Partial<Record<OptionKey, ComponentType<OptionControlProps>>>;
}

/** Edits explicit overrides; unset keys inherit from the defaults layer. */
export function OptionsEditor({
  value,
  onChange,
  schema = OPTIONS_SCHEMA,
  uiSchema = OPTIONS_UI_SCHEMA,
  defaults = DEFAULT_OPTIONS,
  renderers = {},
}: OptionsEditorProps) {
  const idPrefix = useId();
  const overrides = value as Record<string, unknown>;

  const setOption = (key: OptionKey, next: unknown) => {
    onChange({ ...value, [key]: next });
  };

  const resetOption = (key: OptionKey) => {
    const next = { ...value };
    delete next[key];
    onChange(next);
  };

  const resetAll = () => onChange({});

  const effective = useMemo(() => resolveOptions(value), [value]);

  return (
    <div className="p-2 overflow-y-auto flex flex-col gap-4 text-sm">
      <div className="flex items-center justify-between">
        <span className="font-semibold">Formatter options</span>
        <button
          type="button"
          className="btn btn-xs"
          onClick={resetAll}
          disabled={Object.keys(value).length === 0}
        >
          Reset all
        </button>
      </div>

      {uiSchema.groups.map((group) => (
        <section key={group.name} className="flex flex-col gap-2">
          <h3 className="text-xs uppercase tracking-wide opacity-60">
            {group.name}
          </h3>
          {group.options.map((key) => {
            const option = uiSchema.options[key];
            const property = schema.properties[key];
            const Custom = renderers[key];
            const props: OptionControlProps = {
              option,
              value: overrides[key] ?? defaults[key],
              overridden: key in overrides,
              onChange: (next) => setOption(key, next),
              onReset: () => resetOption(key),
            };
            return (
              <div
                key={key}
                className="flex flex-col gap-1 rounded border border-base-300 p-2"
              >
                <div className="flex items-center justify-between gap-2">
                  <label htmlFor={`${idPrefix}-${key}`}>{option.label}</label>
                  <div className="flex items-center gap-1">
                    <span className="text-xs opacity-60">
                      {props.overridden ? "Overridden" : "Default"}
                    </span>
                    {props.overridden && (
                      <button
                        type="button"
                        className="btn btn-ghost btn-xs"
                        onClick={props.onReset}
                        aria-label={`Reset ${option.label}`}
                      >
                        Reset
                      </button>
                    )}
                  </div>
                </div>
                {option.description && (
                  <Description text={option.description} />
                )}
                {Custom ? (
                  <Custom {...props} />
                ) : (
                  <DefaultControl
                    id={`${idPrefix}-${key}`}
                    option={option}
                    property={property}
                    value={props.value}
                    onChange={props.onChange}
                  />
                )}
              </div>
            );
          })}
        </section>
      ))}

      <details className="text-xs">
        <summary className="cursor-pointer opacity-60">
          Explicit overrides and effective options
        </summary>
        <pre className="mt-1 overflow-auto">
          {JSON.stringify({ overrides: value, effective }, null, 2)}
        </pre>
      </details>
    </div>
  );
}

function Description({ text }: { text: string }) {
  return (
    <p className="text-xs opacity-60">
      {text
        .split("`")
        .map((part, index) =>
          index % 2 === 1 ? <code key={part}>{part}</code> : part,
        )}
    </p>
  );
}

function DefaultControl({
  id,
  option,
  property,
  value,
  onChange,
}: {
  id: string;
  option: OptionsUiOption;
  property: OptionsSchema["properties"][OptionKey];
  value: unknown;
  onChange: (value: unknown) => void;
}) {
  if (property.type === "boolean") {
    return (
      <input
        id={id}
        type="checkbox"
        className="checkbox"
        checked={Boolean(value)}
        onChange={(event) => onChange(event.target.checked)}
      />
    );
  }

  if (property.enum) {
    return (
      <select
        id={id}
        className="select"
        value={String(value)}
        onChange={(event) => onChange(event.target.value)}
      >
        {property.enum.map((choice) => (
          <option key={choice} value={choice}>
            {choice}
          </option>
        ))}
      </select>
    );
  }

  if (property.type === "integer" || property.type === "number") {
    const number = Number(value);
    const updateNumber = (raw: string) => {
      const parsed = Number(raw);
      onChange(Number.isFinite(parsed) ? parsed : 0);
    };
    if (option.widget === "slider") {
      return (
        <div className="flex items-center gap-2">
          <input
            id={id}
            type="range"
            className="range flex-1"
            min={property.minimum}
            max={property.maximum}
            value={number}
            onChange={(event) => updateNumber(event.target.value)}
          />
          <input
            type="number"
            className="input w-20"
            aria-label={`${option.label} value`}
            min={property.minimum}
            max={property.maximum}
            value={number}
            onChange={(event) => updateNumber(event.target.value)}
          />
        </div>
      );
    }
    return (
      <input
        id={id}
        type="number"
        className="input w-28"
        min={property.minimum}
        max={property.maximum}
        value={number}
        onChange={(event) => updateNumber(event.target.value)}
      />
    );
  }

  return (
    <input
      id={id}
      type="text"
      className="input"
      value={String(value)}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}

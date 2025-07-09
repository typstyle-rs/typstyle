declare module 'typstyle-wasm' {
  export interface Config {
    max_width?: number;
    tab_spaces?: number;
    blank_lines_upper_bound?: number;
    collapse_markup_spaces?: boolean;
    reorder_import_items?: boolean;
    wrap_text?: boolean;
  }

  export function format(code: string, config?: Config): string;
  export function parse(code: string): string;
  export function format_ir(code: string, config?: Config): string;
}
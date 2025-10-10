// Generic TextMate integration utilities for Monaco Editor
// Based on: https://github.com/janpoem/react-monaco/blob/main/packages/plugin-textmate/src/monaco-editor-textmate
// Logic remain unchanged.

import { INITIAL, type Registry, type StateStack } from "vscode-textmate";
import type { monaco } from "./types";

class TokenizerState implements monaco.languages.IState {
  constructor(private _ruleStack: StateStack) {}

  public get ruleStack(): StateStack {
    return this._ruleStack;
  }

  public clone(): TokenizerState {
    return new TokenizerState(this._ruleStack);
  }

  public equals(other: monaco.languages.IState): boolean {
    if (
      !other ||
      !(other instanceof TokenizerState) ||
      other._ruleStack !== this._ruleStack
    ) {
      return false;
    }
    return true;
  }
}

/**
 * Wires up monaco-editor with monaco-textmate
 *
 * @param _monaco monaco namespace this operation should apply to (usually the `monaco` global unless you have some other setup)
 * @param registry TmGrammar `Registry` this wiring should rely on to provide the grammars
 */
export async function wireTmGrammar(
  _monaco: typeof monaco,
  registry: Registry,
  languageId: string,
  scopeName: string,
  editor?: monaco.editor.ICodeEditor,
) {
  if (scopeName == null) return;
  const grammar = await registry.loadGrammar(scopeName);
  if (grammar == null) return;

  _monaco.languages.setTokensProvider(languageId, {
    getInitialState: () => new TokenizerState(INITIAL),
    tokenize: (line: string, state: TokenizerState) => {
      const res = grammar.tokenizeLine(line, state.ruleStack);
      return {
        endState: new TokenizerState(res.ruleStack),
        tokens: res.tokens.map((token) => ({
          ...token,
          // TODO: At the moment, monaco-editor doesn't seem to accept array of scopes
          scopes: editor
            ? TMToMonacoToken(editor, token.scopes)
            : token.scopes[token.scopes.length - 1],
        })),
      };
    },
  });
}

// as described in issue: https://github.com/NeekSandhu/monaco-textmate/issues/5
const TMToMonacoToken = (
  editor: monaco.editor.ICodeEditor,
  scopes: string[],
) => {
  let scopeName = "";
  if (!scopes[0]) return "";

  // get the scope name. Example: cpp , java, haskell
  for (let i = scopes[0].length - 1; i >= 0; i -= 1) {
    const char = scopes[0][i];
    if (char === ".") {
      break;
    }
    scopeName = char + scopeName;
  }

  // iterate through all scopes from last to first
  for (let i = scopes.length - 1; i >= 0; i -= 1) {
    const scope = scopes[i];
    if (scope == null) continue;
    /**
     * Try all possible tokens from high specific token to low specific token
     *
     * Example:
     * 0 meta.function.definition.parameters.cpp
     * 1 meta.function.definition.parameters
     *
     * 2 meta.function.definition.cpp
     * 3 meta.function.definition
     *
     * 4 meta.function.cpp
     * 5 meta.function
     *
     * 6 meta.cpp
     * 7 meta
     */
    for (let i = scope.length - 1; i >= 0; i -= 1) {
      const char = scope[i];
      if (char === ".") {
        const token = scope.slice(0, i);
        if (
          // biome-ignore lint/suspicious/noExplicitAny: property injection
          (editor as any)._themeService._theme._tokenTheme._match(
            `${token}.${scopeName}`,
          )._foreground > 1
        ) {
          return `${token}.${scopeName}`;
        }
        if (
          // biome-ignore lint/suspicious/noExplicitAny: property injection
          (editor as any)._themeService._theme._tokenTheme._match(token)
            ._foreground > 1
        ) {
          return token;
        }
      }
    }
  }

  return "";
};

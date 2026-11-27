import { beforeEach, describe, expect, it } from "vitest";
import { getStateFromUrl } from "@/utils/url";

describe("URL option parsing", () => {
  beforeEach(() => {
    window.history.replaceState({}, "", "/");
  });

  it("drops malformed and out-of-range values", async () => {
    window.history.replaceState(
      {},
      "",
      "/?max_width=wide&wrap_mode=bogus&tab_spaces=4&reorder_import_items=false",
    );

    const state = await getStateFromUrl();

    expect(state.formatOptions).toEqual({
      tab_spaces: 4,
      reorder_import_items: false,
    });
  });

  it("maps legacy camel-case keys to canonical option keys", async () => {
    window.history.replaceState({}, "", "/?lineWidth=100&wrapMode=fill");

    const state = await getStateFromUrl();

    expect(state.formatOptions).toEqual({
      max_width: 100,
      wrap_mode: "fill",
    });
  });
});

import { describe, expect, it } from "vitest";
import { layoutSunburst, layoutTreemap } from "./treemap";
import type { SpaceNode } from "../types/ipc";
function leaf(name: string, bytes: number): SpaceNode {
  return {
    name,
    path: `/test/${name}`,
    bytes,
    allocatedBytes: bytes,
    ageSeconds: 0,
    isDirectory: false,
    children: [],
  };
}
function root(bytes: number, children: SpaceNode[]): SpaceNode {
  return { ...leaf("root", bytes), path: "/test", isDirectory: true, children };
}
describe("Space Lens layouts", () => {
  it("preserves bytes omitted from a large folder and never mutates scan data", () => {
    const input = root(100, [leaf("visible", 30)]);
    const before = JSON.stringify(input);
    const tiles = layoutTreemap(input, 900, 380, "type");
    expect(tiles.map((t) => t.node.bytes).reduce((a, b) => a + b, 0)).toBe(100);
    expect(tiles.find((t) => t.node.name === "Other items")?.node.bytes).toBe(
      70,
    );
    expect(JSON.stringify(input)).toBe(before);
    expect(
      tiles.every((t) => t.width >= 0 && t.height >= 0 && t.x >= 0 && t.y >= 0),
    ).toBe(true);
  });
  it("does not double-count child data in treemap or sunburst", () => {
    const input = root(100, [root(40, [leaf("a", 40)]), leaf("b", 60)]);
    const tiles = layoutTreemap(input, 900, 380, "age");
    expect(tiles.map((t) => t.node.bytes).reduce((a, b) => a + b, 0)).toBe(100);
    expect(
      layoutSunburst(input, 175, "age").every((a) => !a.path.includes("NaN")),
    ).toBe(true);
    expect(layoutTreemap(root(0, []), 900, 380, "type")).toEqual([]);
  });
});

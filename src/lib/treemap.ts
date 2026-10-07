import { hierarchy, partition, treemap } from "d3-hierarchy";
import type { SpaceNode } from "../types/ipc";
export type HeatMode = "type" | "age";
export interface Tile {
  node: SpaceNode;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
}
const colors = [
  "#527c64",
  "#88a98b",
  "#aac0a0",
  "#c9d4b2",
  "#dce2c7",
  "#879d94",
  "#b3c3c1",
  "#d9c9a8",
];
function normalized(node: SpaceNode): SpaceNode {
  if (!node.isDirectory) return node;
  const children = node.children.map(normalized);
  const visible = children.reduce((sum, child) => sum + child.bytes, 0);
  const missing = Math.max(0, node.bytes - visible);
  if (missing > 0)
    children.push({
      name: "Other items",
      path: `${node.path}/[other]`,
      bytes: missing,
      allocatedBytes: 0,
      ageSeconds: 0,
      isDirectory: false,
      children: [],
    });
  return { ...node, children };
}
function tree(root: SpaceNode) {
  return hierarchy(normalized(root))
    .sum((n) => (n.isDirectory ? 0 : Math.max(0, n.bytes)))
    .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));
}
export function nodeColor(
  node: SpaceNode,
  index: number,
  mode: HeatMode,
): string {
  if (mode === "type") return colors[index % colors.length];
  const days = node.ageSeconds / 86400;
  return days > 180
    ? "#9c6952"
    : days > 90
      ? "#bc9568"
      : days > 30
        ? "#cbbb87"
        : days > 7
          ? "#9ab499"
          : "#557d68";
}
export function layoutTreemap(
  root: SpaceNode,
  width: number,
  height: number,
  mode: HeatMode,
): Tile[] {
  if (root.bytes === 0) return [];
  const layout = treemap<SpaceNode>()
    .size([width, height])
    .padding(5)
    .round(true)(tree(root));
  return (layout.children ?? [])
    .filter((node) => (node.value ?? 0) > 0)
    .map((node, index) => ({
      node: node.data,
      x: node.x0,
      y: node.y0,
      width: node.x1 - node.x0,
      height: node.y1 - node.y0,
      color: nodeColor(node.data, index, mode),
    }));
}
export interface Arc {
  node: SpaceNode;
  path: string;
  color: string;
}
export function layoutSunburst(
  root: SpaceNode,
  radius: number,
  mode: HeatMode,
): Arc[] {
  if (root.bytes === 0) return [];
  const layout = partition<SpaceNode>().size([Math.PI * 2, radius])(tree(root));
  return layout
    .descendants()
    .filter((n) => n.depth > 0 && n.depth <= 3 && n.x1 > n.x0)
    .map((n, index) => ({
      node: n.data,
      path: ringPath(n.x0, n.x1, n.y0, n.y1),
      color: nodeColor(n.data, index, mode),
    }));
}
function ringPath(
  start: number,
  end: number,
  inner: number,
  outer: number,
): string {
  // SVG arcs with equal endpoints cannot draw a full circle; leave an imperceptible gap.
  end = Math.min(end, start + Math.PI * 2 - 0.00001);
  const point = (angle: number, r: number) => [
    Math.sin(angle) * r,
    -Math.cos(angle) * r,
  ];
  const a = point(start, outer),
    b = point(end, outer),
    c = point(end, inner),
    d = point(start, inner);
  const large = end - start > Math.PI ? 1 : 0;
  return `M${a} A${outer},${outer} 0 ${large} 1 ${b} L${c} A${inner},${inner} 0 ${large} 0 ${d} Z`;
}

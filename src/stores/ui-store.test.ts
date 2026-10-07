import { expect, it } from "vitest";
import { useUi } from "./ui-store";
it("path selection uses new arrays and prevents duplicate selections", () => {
  useUi.getState().selectPaths(["/a", "/a"]);
  const before = useUi.getState().selectedPaths;
  useUi.getState().togglePath("/b");
  expect(before).toEqual(["/a"]);
  expect(useUi.getState().selectedPaths).toEqual(["/a", "/b"]);
  useUi.getState().togglePath("/a");
  expect(useUi.getState().selectedPaths).toEqual(["/b"]);
});

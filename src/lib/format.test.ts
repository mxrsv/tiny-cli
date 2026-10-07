import { expect, it } from "vitest";
import { formatBytes, percent, recoverable } from "./format";
it("reports unsupported cleanup semantics conservatively", () => {
  expect(["docker", "time-machine-local", "trash"].map(recoverable)).toEqual([
    false,
    false,
    false,
  ]);
  expect(recoverable("user-logs")).toBe(true);
  expect(percent(12, 0)).toBe(0);
  expect(percent(120, 100)).toBe(100);
  expect(formatBytes(1024 ** 3)).toBe("1 GB");
  expect(formatBytes(NaN)).toBe("0 B");
});

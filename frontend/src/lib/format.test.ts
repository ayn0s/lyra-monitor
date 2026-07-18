import { describe, expect, it } from "vitest";
import { formatBytes, formatUptime } from "./format";

describe("formatBytes", () => {
  it("converts bytes to GB with two decimals", () => {
    expect(formatBytes(1024 ** 3)).toBe("1.00 GB");
  });

  it("handles zero", () => {
    expect(formatBytes(0)).toBe("0.00 GB");
  });

  it("rounds to the nearest hundredth", () => {
    expect(formatBytes(1.5 * 1024 ** 3)).toBe("1.50 GB");
  });
});

describe("formatUptime", () => {
  it("formats zero seconds", () => {
    expect(formatUptime(0)).toBe("0d 0h 0m");
  });

  it("formats minutes only", () => {
    expect(formatUptime(125)).toBe("0d 0h 2m");
  });

  it("formats hours and minutes", () => {
    expect(formatUptime(3661)).toBe("0d 1h 1m");
  });

  it("formats days, hours and minutes", () => {
    expect(formatUptime(90000)).toBe("1d 1h 0m");
  });

  it("truncates rather than rounds", () => {
    expect(formatUptime(3659)).toBe("0d 1h 0m");
  });
});

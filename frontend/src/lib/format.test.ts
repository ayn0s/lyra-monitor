import { describe, expect, it } from "vitest";
import { formatBytes, formatBytesPerSecond, formatUptime } from "./format";

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

describe("formatBytesPerSecond", () => {
  it("formats sub-kilobyte rates in B/s", () => {
    expect(formatBytesPerSecond(512)).toBe("512 B/s");
  });

  it("formats kilobyte rates in KB/s", () => {
    expect(formatBytesPerSecond(2048)).toBe("2.0 KB/s");
  });

  it("formats megabyte rates in MB/s", () => {
    expect(formatBytesPerSecond(5 * 1024 ** 2)).toBe("5.0 MB/s");
  });

  it("handles zero", () => {
    expect(formatBytesPerSecond(0)).toBe("0 B/s");
  });

  it("switches units at the 1024 boundary", () => {
    expect(formatBytesPerSecond(1023)).toBe("1023 B/s");
    expect(formatBytesPerSecond(1024)).toBe("1.0 KB/s");
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

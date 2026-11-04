import { describe, expect, it } from "vitest";
import { ago, bytes, files } from "./format";

describe("format", () => {
  it("sizes", () => {
    expect(bytes(0)).toBe("0 B");
    expect(bytes(1536)).toBe("1.5 KB");
    expect(bytes(10 * 1024 * 1024)).toBe("10 MB");
  });

  it("file counts", () => {
    expect(files(1)).toBe("1 file");
    expect(files(0)).toBe("0 files");
  });

  it("time ago", () => {
    expect(ago(1000, 1010)).toBe("just now");
    expect(ago(1000, 1000 + 125)).toBe("2 min ago");
    expect(ago(1000, 1000 + 7200)).toBe("2 h ago");
    expect(ago(1000, 1000 + 3 * 86400)).toBe("3 days ago");
  });
});

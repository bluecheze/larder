import { describe, expect, it } from "vitest";
import { bytes, files } from "./format";

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
});

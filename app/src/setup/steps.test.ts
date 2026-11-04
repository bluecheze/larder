import { describe, expect, it } from "vitest";
import { initialDraft, problem, stepsFor, toHostConfig } from "./steps";

const defaults = { device_name: "Studio Mac", library: "/Users/me/Larder", port: 5273 };

describe("setup steps", () => {
  it("starts as a host", () => {
    expect(initialDraft(defaults).role).toBe("host");
    expect(stepsFor("host")).toContain("library");
    expect(stepsFor("client")).toContain("invite");
    expect(stepsFor("client")).not.toContain("library");
  });

  it("checks the network step", () => {
    const draft = initialDraft(defaults);
    expect(problem("network", draft)).toBeNull();
    expect(problem("network", { ...draft, port: "80" })).toMatch(/port/);
    expect(problem("network", { ...draft, useDomain: true, domain: "nope", email: "a@b.c" })).toMatch(/domain/);
    expect(problem("network", { ...draft, useDomain: true, domain: "files.example.com", email: "" })).toMatch(/email/);
    expect(problem("network", { ...draft, useDomain: true, domain: "files.example.com", email: "a@b.c" })).toBeNull();
  });

  it("wants a real invite", () => {
    const draft = { ...initialDraft(defaults), role: "client" as const };
    expect(problem("invite", draft)).not.toBeNull();
    expect(problem("invite", { ...draft, invite: " larder://pc:5273?token=x " })).toBeNull();
  });

  it("builds a host config and drops the domain when it's off", () => {
    const config = toHostConfig({ ...initialDraft(defaults), domain: "left.over", email: "x@y.z" });
    expect(config.host).toEqual({ library: "/Users/me/Larder", port: 5273, domain: null, email: null });
    expect(config.client).toBeNull();
  });

  it("keeps the domain when it's on", () => {
    const config = toHostConfig({ ...initialDraft(defaults), useDomain: true, domain: "Files.Example.com ", email: "a@b.c" });
    expect(config.host?.domain).toBe("files.example.com");
  });
});

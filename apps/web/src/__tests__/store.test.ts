import { beforeEach, describe, expect, it, vi } from "vitest";
import { addActivity, addStreamId, getActivity, getStreamIds, type ActivityItem } from "../lib/store";

const activity = (n: number): ActivityItem => ({
  hash: `hash-${n}`,
  kind: "withdraw",
  streamId: String(n),
  at: n,
  status: "SUCCESS",
});

describe("browser persistence", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
  });

  it("adds a stream ID only once", () => {
    addStreamId("wallet-a", "stream-1");
    addStreamId("wallet-a", "stream-1");
    expect(getStreamIds("wallet-a")).toEqual(["stream-1"]);
  });

  it("keeps stream IDs isolated by address", () => {
    addStreamId("wallet-a", "stream-a");
    addStreamId("wallet-b", "stream-b");
    expect(getStreamIds("wallet-a")).toEqual(["stream-a"]);
    expect(getStreamIds("wallet-b")).toEqual(["stream-b"]);
    expect(localStorage.getItem("streamora:streams:wallet-a")).toBe('["stream-a"]');
  });

  it("puts newest activity first", () => {
    addActivity("wallet-a", activity(1));
    addActivity("wallet-a", activity(2));
    expect(getActivity("wallet-a").map((item) => item.hash)).toEqual(["hash-2", "hash-1"]);
  });

  it("caps activity history at 100 items", () => {
    for (let i = 0; i < 101; i += 1) addActivity("wallet-a", activity(i));
    const items = getActivity("wallet-a");
    expect(items).toHaveLength(100);
    expect(items[0].hash).toBe("hash-100");
    expect(items.at(-1)?.hash).toBe("hash-1");
  });

  it("returns an empty list when stored JSON is corrupt", () => {
    localStorage.setItem("streamora:streams:wallet-a", "{");
    localStorage.setItem("streamora:activity:wallet-a", "not-json");
    expect(getStreamIds("wallet-a")).toEqual([]);
    expect(getActivity("wallet-a")).toEqual([]);
  });

  it("falls back to empty lists when storage reads throw", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("storage blocked");
    });
    expect(getStreamIds("wallet-a")).toEqual([]);
    expect(getActivity("wallet-a")).toEqual([]);
  });

  it("silently ignores writes when storage throws", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("storage blocked");
    });
    expect(() => addStreamId("wallet-a", "stream-1")).not.toThrow();
    expect(() => addActivity("wallet-a", activity(1))).not.toThrow();
  });
});

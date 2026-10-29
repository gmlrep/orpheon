import { describe, expect, it } from "vitest";
import { enqueueArt } from "./artQueue";
import { deferred } from "../test/helpers";

describe("enqueueArt", () => {
  it("runs at most two jobs concurrently", async () => {
    const started: number[] = [];
    const gates = [deferred(), deferred(), deferred(), deferred(), deferred()];

    const jobs = gates.map((gate, i) =>
      enqueueArt(async () => {
        started.push(i);
        await gate.promise;
        return i;
      }),
    );

    await Promise.resolve();
    expect(started).toEqual([0, 1]);

    gates[0].resolve();
    await jobs[0];
    await Promise.resolve();
    expect(started).toEqual([0, 1, 2]);

    gates[1].resolve();
    gates[2].resolve();
    await Promise.all([jobs[1], jobs[2]]);
    await Promise.resolve();
    expect(started).toEqual([0, 1, 2, 3, 4]);

    gates[3].resolve();
    gates[4].resolve();
    await expect(Promise.all(jobs)).resolves.toEqual([0, 1, 2, 3, 4]);
  });

  it("rejects without blocking the queue", async () => {
    const fail = deferred<never>();
    const ok = deferred<string>();

    const failed = enqueueArt(() => fail.promise);
    const passed = enqueueArt(() => ok.promise);
    const third = enqueueArt(async () => "third");

    fail.reject(new Error("boom"));
    await expect(failed).rejects.toThrow("boom");

    ok.resolve("ok");
    await expect(passed).resolves.toBe("ok");
    await expect(third).resolves.toBe("third");
  });
});

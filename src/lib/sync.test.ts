import { describe, it, expect } from "vitest";
import { writable, get } from "svelte/store";
import { createSync, type Bus } from "./sync";

function fakeBus() {
  const listeners = new Map<string, ((p: unknown) => void)[]>();
  const bus: Bus = {
    emit: (c, p) => { (listeners.get(c) ?? []).forEach((cb) => cb(p)); },
    listen: (c, cb) => { listeners.set(c, [...(listeners.get(c) ?? []), cb]); },
  };
  return bus;
}

describe("createSync", () => {
  it("does not broadcast the initial store value (no clobber on open)", () => {
    const bus = fakeBus();
    let emits = 0;
    const spy: Bus = { emit: (c, p) => { emits++; bus.emit(c, p); }, listen: bus.listen };
    const s = createSync(spy, "a");
    s.register("sync:sw", writable({ v: 1 }));
    expect(emits).toBe(0); // registering must not emit
  });

  it("propagates a change from window A to window B, and B does not echo it back", () => {
    const bus = fakeBus();
    const a = createSync(bus, "a");
    const b = createSync(bus, "b");
    const sa = writable({ v: 0 });
    const sb = writable({ v: 0 });
    a.register("sync:sw", sa);
    b.register("sync:sw", sb);

    let bEmits = 0;
    const origEmit = bus.emit;
    (bus as any).emit = (c: string, p: unknown) => { if ((p as any).origin === "b") bEmits++; origEmit(c, p); };

    sa.set({ v: 42 }); // user action in A
    expect(get(sb)).toEqual({ v: 42 }); // B mirrored it
    expect(bEmits).toBe(0); // B applied without re-broadcasting (echo guard)
  });

  it("answers a request by broadcasting current state to the requester", () => {
    const bus = fakeBus();
    const a = createSync(bus, "a");
    const b = createSync(bus, "b");
    const sa = writable({ v: 7 });
    const sb = writable({ v: 0 });
    a.register("sync:sw", sa);
    b.register("sync:sw", sb);
    b.request("sync:sw");       // B opens and asks for current state
    expect(get(sb)).toEqual({ v: 7 }); // A answered, B adopted it
    expect(get(sa)).toEqual({ v: 7 }); // A unchanged
  });
});

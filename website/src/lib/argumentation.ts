// website/src/lib/argumentation.ts
//
// Type-safe wrapper around the wasm-pack output. Components import
// from here, never from /wasm/... directly. Lets us refactor the
// binding layer without touching components.

let modPromise: Promise<any> | null = null;

async function loadModule(): Promise<any> {
  if (!modPromise) {
    modPromise = (async () => {
      // @ts-ignore — generated, no typings
      const mod = await import('/wasm/argumentation/argumentation_wasm.js');
      await mod.default();
      return mod;
    })();
  }
  return modPromise;
}

export type Extension = string[];

export class Framework {
  private inner: any;
  private constructor(inner: any) {
    this.inner = inner;
  }

  static async create(): Promise<Framework> {
    const mod = await loadModule();
    return new Framework(new mod.WasmFramework());
  }

  addArgument(id: string): this {
    this.inner.add_argument(id);
    return this;
  }
  addAttack(from: string, to: string): this {
    this.inner.add_attack(from, to);
    return this;
  }

  groundedExtension(): Extension {
    return Array.from(this.inner.grounded_extension()) as string[];
  }

  preferredExtensions(): Extension[] {
    const raw = this.inner.preferred_extensions() as unknown[];
    return raw.map((ext) => Array.from(ext as Iterable<string>));
  }

  isCredulouslyAccepted(arg: string): boolean {
    return this.inner.is_credulously_accepted(arg);
  }
  isSkepticallyAccepted(arg: string): boolean {
    return this.inner.is_skeptically_accepted(arg);
  }
}

export type LiveEdgeKey = string; // "from->to"

export class WeightedFramework {
  private inner: any;
  private constructor(inner: any) {
    this.inner = inner;
  }

  static async create(): Promise<WeightedFramework> {
    const mod = await loadModule();
    return new WeightedFramework(new mod.WasmWeightedFramework());
  }

  addArgument(id: string): this {
    this.inner.add_argument(id);
    return this;
  }
  addWeightedAttack(from: string, to: string, weight: number): this {
    this.inner.add_weighted_attack(from, to, weight);
    return this;
  }

  setIntensity(beta: number): this {
    this.inner.set_intensity(beta);
    return this;
  }
  intensity(): number {
    return this.inner.current_intensity();
  }

  liveAttackKeys(): Set<LiveEdgeKey> {
    return new Set(Array.from(this.inner.live_attacks_at_current_beta()) as string[]);
  }

  isCredulouslyAccepted(arg: string): boolean {
    return this.inner.is_credulous(arg);
  }
}

export function edgeKey(from: string, to: string): LiveEdgeKey {
  return `${from}->${to}`;
}

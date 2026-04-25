import React, {useEffect, useState} from 'react';
import Layout from '@theme/Layout';
import BrowserOnly from '@docusaurus/BrowserOnly';

function SmokeInner() {
  const [result, setResult] = useState<string[]>(['loading...']);
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        // @ts-ignore — generated, no typings yet (Task 8 adds them)
        const mod = await import('/wasm/argumentation/argumentation_wasm.js');
        await mod.default();

        // Dung
        const fw = new mod.WasmFramework();
        fw.add_argument('A'); fw.add_argument('B'); fw.add_argument('C');
        fw.add_attack('A', 'B'); fw.add_attack('B', 'C');
        const grounded = fw.grounded_extension();

        // Weighted
        const w = new mod.WasmWeightedFramework();
        w.add_argument('A'); w.add_argument('B');
        w.add_weighted_attack('B', 'A', 0.4);
        w.set_intensity(0.0);
        const liveAt0 = w.live_attacks_at_current_beta();
        const credAt0 = w.is_credulous('A');
        w.set_intensity(0.4);
        const liveAt4 = w.live_attacks_at_current_beta();
        const credAt4 = w.is_credulous('A');

        if (!cancelled) setResult([
          `grounded(A→B→C) = [${[...grounded].join(', ')}]`,
          `weighted: live attacks at β=0.0 = [${[...liveAt0].join(', ')}]`,
          `weighted: A credulous at β=0.0 = ${credAt0}`,
          `weighted: live attacks at β=0.4 = [${[...liveAt4].join(', ')}]`,
          `weighted: A credulous at β=0.4 = ${credAt4}`,
        ]);
      } catch (e) {
        if (!cancelled) setResult([`ERROR: ${(e as Error).message}`]);
      }
    })();
    return () => { cancelled = true; };
  }, []);
  return (
    <div style={{padding: '4rem', fontFamily: 'monospace', fontSize: 16, lineHeight: 1.7}}>
      {result.map((line, i) => <div key={i}>{line}</div>)}
    </div>
  );
}

export default function WasmSmoke() {
  return (
    <Layout title="WASM smoke test">
      <BrowserOnly fallback={<div>Loading…</div>}>
        {() => <SmokeInner />}
      </BrowserOnly>
    </Layout>
  );
}

import React, {useEffect, useState} from 'react';
import Layout from '@theme/Layout';
import BrowserOnly from '@docusaurus/BrowserOnly';

function SmokeInner() {
  const [result, setResult] = useState<string>('loading...');
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        // @ts-ignore — generated, no typings yet (Task 8 adds them)
        const mod = await import('/wasm/argumentation/argumentation_wasm.js');
        await mod.default(); // initialize wasm
        const fw = new mod.WasmFramework();
        fw.add_argument('A');
        fw.add_argument('B');
        fw.add_argument('C');
        fw.add_attack('A', 'B');
        fw.add_attack('B', 'C');
        const ext = fw.grounded_extension();
        if (!cancelled) setResult(`grounded(A→B→C) = [${ext.join(', ')}]`);
      } catch (e) {
        if (!cancelled) setResult(`ERROR: ${(e as Error).message}`);
      }
    })();
    return () => { cancelled = true; };
  }, []);
  return <div style={{padding: '4rem', fontFamily: 'monospace', fontSize: 18}}>{result}</div>;
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

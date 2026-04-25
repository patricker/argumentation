import React, {useEffect, useState} from 'react';
import Layout from '@theme/Layout';
import BrowserOnly from '@docusaurus/BrowserOnly';
import {Framework, WeightedFramework} from '@site/src/lib/argumentation';
import BetaPlayground from '@site/src/components/BetaPlayground';

function SmokeInner() {
  const [result, setResult] = useState<string[]>(['loading...']);
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const fw = await Framework.create();
        fw.addArgument('A').addArgument('B').addArgument('C');
        fw.addAttack('A', 'B').addAttack('B', 'C');

        const w = await WeightedFramework.create();
        w.addArgument('A').addArgument('B').addWeightedAttack('B', 'A', 0.4);
        w.setIntensity(0.0);
        const liveAt0 = [...w.liveAttackKeys()];
        const credAt0 = w.isCredulouslyAccepted('A');
        w.setIntensity(0.4);
        const liveAt4 = [...w.liveAttackKeys()];
        const credAt4 = w.isCredulouslyAccepted('A');

        if (!cancelled) setResult([
          `grounded(A→B→C) = [${fw.groundedExtension().join(', ')}]`,
          `weighted: live attacks at β=0.0 = [${liveAt0.join(', ')}]`,
          `weighted: A credulous at β=0.0 = ${credAt0}`,
          `weighted: live attacks at β=0.4 = [${liveAt4.join(', ')}]`,
          `weighted: A credulous at β=0.4 = ${credAt4}`,
        ]);
      } catch (e) {
        if (!cancelled) setResult([`ERROR: ${(e as Error).message}`]);
      }
    })();
    return () => { cancelled = true; };
  }, []);
  return (
    <>
      <div style={{padding: '4rem', fontFamily: 'monospace', fontSize: 16, lineHeight: 1.7}}>
        {result.map((line, i) => <div key={i}>{line}</div>)}
      </div>
      <div style={{maxWidth: 700, margin: '3rem 0', padding: '0 4rem'}}>
        <BetaPlayground
          title="BetaPlayground smoke (Alice vs Bob)"
          args={[
            {id: 'A', label: 'Alice: warmer is healthier'},
            {id: 'B', label: 'Bob: cooler saves energy'},
          ]}
          attacks={[{from: 'B', to: 'A', weight: 0.4}]}
        />
      </div>
    </>
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

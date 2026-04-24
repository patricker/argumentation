import React, {useEffect, useRef, useState} from 'react';
import styles from './styles.module.css';
import SceneTrace, {type Trace} from '../SceneTrace';

type Props = {
  tracePaths: {beta: number; path: string}[];
  title?: string;
};

export default function BetaSlider({tracePaths, title}: Props) {
  const [idx, setIdx] = useState(0);
  const [traces, setTraces] = useState<Record<number, Trace>>({});
  const inFlight = useRef<Set<number>>(new Set());

  useEffect(() => {
    tracePaths.forEach(tp => {
      if (traces[tp.beta] || inFlight.current.has(tp.beta)) return;
      inFlight.current.add(tp.beta);
      fetch(tp.path)
        .then(r => r.json())
        .then(json => {
          setTraces(prev => ({...prev, [tp.beta]: json}));
        })
        .finally(() => {
          inFlight.current.delete(tp.beta);
        });
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tracePaths.map(tp => tp.path).join('|')]);

  const current = tracePaths[idx];
  const trace = traces[current?.beta];

  return (
    <div className={styles.wrapper}>
      {title && <div className={styles.title}>{title}</div>}
      <div className={styles.sliderRow}>
        <span>β:</span>
        <input
          type="range"
          min={0}
          max={tracePaths.length - 1}
          step={1}
          value={idx}
          onChange={e => setIdx(Number(e.target.value))}
        />
        <span className={styles.value}>{current?.beta.toFixed(2)}</span>
      </div>
      {trace ? <SceneTrace trace={trace} /> : <div>Loading…</div>}
    </div>
  );
}

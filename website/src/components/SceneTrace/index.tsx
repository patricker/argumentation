import React, {useEffect, useState} from 'react';
import styles from './styles.module.css';
import AttackGraph, {type ArgNode, type ArgEdge} from '../AttackGraph';

export type Trace = {
  scene_name: string;
  beta: number;
  participants: string[];
  seeded_arguments: {actor: string; affordance_name: string; conclusion: string}[];
  attacks: {attacker: string; target: string; weight: number}[];
  beats: {actor: string; action: string; accepted: boolean}[];
  errors: string[];
};

type Props = {trace: Trace};

export default function SceneTrace({trace}: Props) {
  const [beatIdx, setBeatIdx] = useState(0);
  const [playing, setPlaying] = useState(false);
  const totalBeats = trace.beats.length;

  useEffect(() => {
    if (!playing) return;
    if (beatIdx >= totalBeats) {setPlaying(false); return;}
    const t = setTimeout(() => setBeatIdx(i => i + 1), 900);
    return () => clearTimeout(t);
  }, [playing, beatIdx, totalBeats]);

  const visibleBeats = trace.beats.slice(0, beatIdx);

  const argNodes: ArgNode[] = trace.seeded_arguments.map(a => ({
    id: a.conclusion,
    label: `${a.conclusion}\n(${a.actor})`,
  }));
  const argEdges: ArgEdge[] = trace.attacks.map(e => ({
    from: e.attacker, to: e.target, weight: e.weight,
  }));

  return (
    <div className={styles.wrapper}>
      <div className={styles.header}>
        <strong>Scene: {trace.scene_name}</strong>
        <span className={styles.beta}>β = {trace.beta.toFixed(2)}</span>
      </div>
      <AttackGraph arguments={argNodes} attacks={argEdges} height={240} />
      <div className={styles.controls}>
        <button onClick={() => setBeatIdx(0)} disabled={beatIdx === 0}>⟲ Reset</button>
        <button onClick={() => setBeatIdx(i => Math.max(0, i - 1))} disabled={beatIdx === 0}>← Back</button>
        <button onClick={() => setBeatIdx(i => Math.min(totalBeats, i + 1))} disabled={beatIdx >= totalBeats}>Next →</button>
        <button onClick={() => {setBeatIdx(0); setPlaying(true);}}>▶ Play</button>
        <span className={styles.counter}>Beat {beatIdx} / {totalBeats}</span>
      </div>
      <ol className={styles.beatList}>
        {visibleBeats.map((b, i) => (
          <li key={i} className={b.accepted ? styles.accepted : styles.rejected}>
            <strong>{b.actor}</strong> proposed <code>{b.action}</code>
            {b.accepted ? ' — accepted' : ' — rejected'}
          </li>
        ))}
      </ol>
      {trace.errors.length > 0 && beatIdx === totalBeats && (
        <div className={styles.errors}>
          <strong>Latched errors:</strong>
          <ul>{trace.errors.map((e, i) => <li key={i}>{e}</li>)}</ul>
        </div>
      )}
    </div>
  );
}

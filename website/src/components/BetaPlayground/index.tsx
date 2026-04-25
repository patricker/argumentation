import React, {useEffect, useMemo, useState} from 'react';
import BrowserOnly from '@docusaurus/BrowserOnly';
import AttackGraph, {type ArgNode, type ArgEdge} from '../AttackGraph';
import {WeightedFramework} from '@site/src/lib/argumentation';
import styles from './styles.module.css';

export type BetaPlaygroundArg = {id: string; label?: string};
export type BetaPlaygroundAttack = {from: string; to: string; weight: number};

type Props = {
  title?: string;
  args: BetaPlaygroundArg[];
  attacks: BetaPlaygroundAttack[];
  initialBeta?: number;
};

function Inner({title, args, attacks, initialBeta = 0}: Props) {
  const [fw, setFw] = useState<WeightedFramework | null>(null);
  const [beta, setBeta] = useState<number>(initialBeta);

  // Build the framework once per (args, attacks) shape change. Subagents
  // working with React must keep wasm objects out of state-shape deps to
  // avoid infinite rebuild loops; we key on a stable string instead.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      const w = await WeightedFramework.create();
      args.forEach((a) => w.addArgument(a.id));
      attacks.forEach((e) => w.addWeightedAttack(e.from, e.to, e.weight));
      w.setIntensity(initialBeta);
      if (!cancelled) setFw(w);
    })();
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    args.map((a) => a.id).join('|'),
    attacks.map((e) => `${e.from}->${e.to}@${e.weight}`).join('|'),
  ]);

  // Recompute live edges + per-arg credulous state whenever β changes or
  // the framework is (re)constructed.
  const {liveEdges, acceptedById} = useMemo(() => {
    if (!fw) {
      return {
        liveEdges: new Set<string>(),
        acceptedById: new Map<string, boolean>(),
      };
    }
    fw.setIntensity(beta);
    const live = fw.liveAttackKeys();
    const acc = new Map<string, boolean>();
    args.forEach((a) => acc.set(a.id, fw.isCredulouslyAccepted(a.id)));
    return {liveEdges: live, acceptedById: acc};
  }, [fw, beta, args]);

  const argNodes: ArgNode[] = args.map((a) => {
    const isAccepted = acceptedById.get(a.id);
    return {
      id: a.id,
      label: a.label,
      accepted:
        isAccepted === undefined ? undefined : isAccepted ? 'credulous' : false,
    };
  });
  const argEdges: ArgEdge[] = attacks.map((e) => ({
    from: e.from,
    to: e.to,
    weight: e.weight,
  }));

  return (
    <div className={styles.wrapper}>
      {title && <div className={styles.title}>{title}</div>}
      <div className={styles.controls}>
        <span>β:</span>
        <input
          className={styles.slider}
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={beta}
          onChange={(e) => setBeta(Number(e.target.value))}
        />
        <span className={styles.beta}>{beta.toFixed(2)}</span>
      </div>
      <AttackGraph
        arguments={argNodes}
        attacks={argEdges}
        liveEdges={liveEdges}
        height={300}
        legend
      />
      <div className={styles.status}>
        {args.map((a) => {
          const acc = acceptedById.get(a.id);
          return (
            <span key={a.id} className={styles.statusItem}>
              <code>{a.id}</code>{' '}
              ·{' '}
              <span className={acc ? styles.accepted : styles.rejected}>
                {acc ? 'credulously accepted' : 'not accepted'}
              </span>
            </span>
          );
        })}
      </div>
    </div>
  );
}

export default function BetaPlayground(props: Props) {
  return (
    <BrowserOnly
      fallback={
        <div className={styles.wrapper}>
          <div className={styles.title}>Loading…</div>
        </div>
      }
    >
      {() => <Inner {...props} />}
    </BrowserOnly>
  );
}

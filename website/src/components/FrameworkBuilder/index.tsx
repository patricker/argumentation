import React, {useCallback, useEffect, useMemo, useState} from 'react';
import BrowserOnly from '@docusaurus/BrowserOnly';
import ReactFlow, {
  Background,
  Controls,
  MarkerType,
  type Edge,
  type Node,
  type NodeMouseHandler,
  ReactFlowProvider,
} from 'reactflow';
import 'reactflow/dist/style.css';
import {Framework} from '@site/src/lib/argumentation';
import styles from './styles.module.css';

type Mode = 'add-node' | 'add-attack' | 'select';
type Preset = 'blank' | 'nixon' | 'tweety' | 'hal-carla';

type FBState = {
  args: {id: string; label: string; x: number; y: number}[];
  attacks: {from: string; to: string}[];
};

const PRESETS: Record<Preset, FBState> = {
  blank: {args: [], attacks: []},
  nixon: {
    args: [
      {id: 'A', label: 'Republican → not pacifist', x: -160, y: 0},
      {id: 'B', label: 'Quaker → pacifist', x: 160, y: 0},
    ],
    attacks: [
      {from: 'A', to: 'B'},
      {from: 'B', to: 'A'},
    ],
  },
  tweety: {
    args: [
      {id: 'A1', label: 'Tweety flies (R1)', x: -160, y: 0},
      {id: 'A2', label: "Tweety doesn't fly (R2)", x: 160, y: 0},
    ],
    attacks: [
      {from: 'A1', to: 'A2'},
      {from: 'A2', to: 'A1'},
    ],
  },
  'hal-carla': {
    args: [
      {id: 'H1', label: 'Hal: life > property', x: -200, y: -120},
      {id: 'C1', label: 'Carla: property rights', x: 200, y: -120},
      {id: 'H2', label: 'Hal: too poor to compensate', x: -200, y: 120},
      {id: 'C2', label: 'Carla: my only dose', x: 200, y: 120},
    ],
    attacks: [
      {from: 'C1', to: 'H1'},
      {from: 'H1', to: 'C1'},
      {from: 'C2', to: 'H2'},
      {from: 'H2', to: 'C1'},
    ],
  },
};

function Inner() {
  const [mode, setMode] = useState<Mode>('add-node');
  const [preset, setPreset] = useState<Preset>('blank');
  const [state, setState] = useState<FBState>(PRESETS.blank);
  const [pendingAttack, setPendingAttack] = useState<string | null>(null);
  const [grounded, setGrounded] = useState<string[]>([]);
  const [preferred, setPreferred] = useState<string[][]>([]);
  const [credulous, setCredulous] = useState<string[]>([]);
  const [skeptical, setSkeptical] = useState<string[]>([]);
  const [counter, setCounter] = useState<number>(1);

  // Recompute extensions whenever state changes.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const fw = await Framework.create();
        state.args.forEach((a) => fw.addArgument(a.id));
        state.attacks.forEach((e) => fw.addAttack(e.from, e.to));
        const g = fw.groundedExtension().slice().sort();
        const p = fw.preferredExtensions().map((e) => e.slice().sort());
        const c = state.args.filter((a) => fw.isCredulouslyAccepted(a.id)).map((a) => a.id);
        const s = state.args.filter((a) => fw.isSkepticallyAccepted(a.id)).map((a) => a.id);
        if (!cancelled) {
          setGrounded(g);
          setPreferred(p);
          setCredulous(c);
          setSkeptical(s);
        }
      } catch (err) {
        // Recompute should never throw for the small frameworks the
        // playground produces; if it does, leave previous state intact.
        if (!cancelled) {
          // eslint-disable-next-line no-console
          console.error('FrameworkBuilder recompute failed:', err);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [state]);

  const loadPreset = (p: Preset) => {
    setPreset(p);
    setState(PRESETS[p]);
    setPendingAttack(null);
    setCounter(PRESETS[p].args.length + 1);
  };

  const onPaneClick = useCallback(
    (event: React.MouseEvent) => {
      if (mode !== 'add-node') return;
      const target = event.currentTarget as HTMLElement;
      const rect = target.getBoundingClientRect();
      const x = event.clientX - rect.left - rect.width / 2;
      const y = event.clientY - rect.top - rect.height / 2;
      const id = String.fromCharCode(64 + counter); // A, B, C, ...
      setState((prev) => ({
        ...prev,
        args: [...prev.args, {id, label: id, x, y}],
      }));
      setCounter((c) => c + 1);
    },
    [mode, counter],
  );

  const onNodeClick: NodeMouseHandler = useCallback(
    (_event, node) => {
      if (mode !== 'add-attack') return;
      if (pendingAttack === null) {
        setPendingAttack(node.id);
      } else if (pendingAttack !== node.id) {
        const from = pendingAttack;
        const to = node.id;
        setState((prev) => {
          if (prev.attacks.some((e) => e.from === from && e.to === to)) return prev;
          return {...prev, attacks: [...prev.attacks, {from, to}]};
        });
        setPendingAttack(null);
      } else {
        setPendingAttack(null); // click same node to cancel
      }
    },
    [mode, pendingAttack],
  );

  const reactFlowNodes: Node[] = useMemo(
    () =>
      state.args.map((a) => ({
        id: a.id,
        position: {x: a.x, y: a.y},
        data: {label: a.label},
        style: {
          borderRadius: 10,
          padding: 8,
          minWidth: 90,
          border:
            pendingAttack === a.id
              ? '2px solid var(--ifm-color-warning)'
              : credulous.includes(a.id)
              ? '2px solid var(--ifm-color-success)'
              : '1.5px solid var(--ifm-color-emphasis-400)',
          background: 'var(--ifm-background-surface-color)',
        },
      })),
    [state.args, credulous, pendingAttack],
  );

  const reactFlowEdges: Edge[] = useMemo(
    () =>
      state.attacks.map((e, i) => ({
        id: `e-${i}`,
        source: e.from,
        target: e.to,
        type: 'smoothstep',
        style: {stroke: 'var(--ifm-color-danger)', strokeWidth: 2},
        markerEnd: {
          type: MarkerType.ArrowClosed,
          color: 'var(--ifm-color-danger)',
          width: 20,
          height: 20,
        },
      })),
    [state.attacks],
  );

  return (
    <div className={styles.wrapper}>
      <div>
        <div className={styles.toolbar}>
          <span>Mode:</span>
          <button className={mode === 'add-node' ? styles.active : ''} onClick={() => setMode('add-node')}>
            + Node
          </button>
          <button
            className={mode === 'add-attack' ? styles.active : ''}
            onClick={() => {
              setMode('add-attack');
              setPendingAttack(null);
            }}
          >
            → Attack
          </button>
          <button className={mode === 'select' ? styles.active : ''} onClick={() => setMode('select')}>
            Select
          </button>
          <span style={{marginLeft: 'auto'}}>Preset:</span>
          <button className={preset === 'blank' ? styles.active : ''} onClick={() => loadPreset('blank')}>
            Blank
          </button>
          <button className={preset === 'nixon' ? styles.active : ''} onClick={() => loadPreset('nixon')}>
            Nixon
          </button>
          <button className={preset === 'tweety' ? styles.active : ''} onClick={() => loadPreset('tweety')}>
            Tweety
          </button>
          <button className={preset === 'hal-carla' ? styles.active : ''} onClick={() => loadPreset('hal-carla')}>
            Hal &amp; Carla
          </button>
        </div>
        <div className={styles.canvas}>
          <ReactFlow
            nodes={reactFlowNodes}
            edges={reactFlowEdges}
            onPaneClick={onPaneClick}
            onNodeClick={onNodeClick}
            fitView
            fitViewOptions={{padding: 0.3}}
            proOptions={{hideAttribution: true}}
            nodesDraggable={mode === 'select'}
            nodesConnectable={false}
            elementsSelectable={mode === 'select'}
          >
            <Background gap={18} size={1} color="var(--ifm-color-emphasis-200)" />
            <Controls showInteractive={false} />
          </ReactFlow>
        </div>
      </div>
      <div className={styles.panel}>
        <h4>Grounded</h4>
        {grounded.length ? (
          <ul>
            {grounded.map((id) => (
              <li key={id}>
                <code>{id}</code>
              </li>
            ))}
          </ul>
        ) : (
          <p className={styles.empty}>∅</p>
        )}

        <h4>Preferred extensions ({preferred.length})</h4>
        {preferred.length ? (
          <ul>
            {preferred.map((ext, i) => (
              <li key={i}>{`{ ${ext.map((id) => `'${id}'`).join(', ')} }`}</li>
            ))}
          </ul>
        ) : (
          <p className={styles.empty}>∅</p>
        )}

        <h4>Credulously accepted</h4>
        {credulous.length ? (
          <ul>
            {credulous.map((id) => (
              <li key={id}>
                <code>{id}</code>
              </li>
            ))}
          </ul>
        ) : (
          <p className={styles.empty}>∅</p>
        )}

        <h4>Skeptically accepted</h4>
        {skeptical.length ? (
          <ul>
            {skeptical.map((id) => (
              <li key={id}>
                <code>{id}</code>
              </li>
            ))}
          </ul>
        ) : (
          <p className={styles.empty}>∅</p>
        )}
      </div>
    </div>
  );
}

export default function FrameworkBuilder() {
  return (
    <BrowserOnly fallback={<div className={styles.wrapper}>Loading playground…</div>}>
      {() => (
        <ReactFlowProvider>
          <Inner />
        </ReactFlowProvider>
      )}
    </BrowserOnly>
  );
}

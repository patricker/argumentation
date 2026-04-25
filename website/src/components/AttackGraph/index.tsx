import React, {useMemo} from 'react';
import ReactFlow, {
  Background,
  Controls,
  Handle,
  MarkerType,
  Position,
  type Edge,
  type Node,
  type NodeProps,
} from 'reactflow';
import 'reactflow/dist/style.css';
import styles from './styles.module.css';

export type AcceptedState = true | false | 'credulous' | 'undecided';

export type ArgNode = {
  id: string;
  label?: string;
  accepted?: AcceptedState;
};

export type ArgEdge = {
  from: string;
  to: string;
  weight?: number;
  kind?: 'attack' | 'support' | 'undercut';
};

type Layout = 'row' | 'circle' | 'auto';

type Props = {
  arguments: ArgNode[];
  attacks?: ArgEdge[];
  supports?: ArgEdge[];
  height?: number;
  title?: string;
  layout?: Layout;
  legend?: boolean;
  caption?: string;
};

function ArgumentNode({data}: NodeProps<{argId: string; label?: string; accepted?: AcceptedState}>) {
  const acceptedClass =
    data.accepted === true
      ? styles.accepted
      : data.accepted === false
      ? styles.rejected
      : data.accepted === 'credulous'
      ? styles.credulous
      : '';
  return (
    <div className={`${styles.node} ${acceptedClass}`}>
      <Handle type="target" position={Position.Top} className={styles.handle} />
      <Handle type="source" position={Position.Top} className={styles.handle} />
      <Handle type="target" position={Position.Right} className={styles.handle} />
      <Handle type="source" position={Position.Right} className={styles.handle} />
      <Handle type="target" position={Position.Bottom} className={styles.handle} />
      <Handle type="source" position={Position.Bottom} className={styles.handle} />
      <Handle type="target" position={Position.Left} className={styles.handle} />
      <Handle type="source" position={Position.Left} className={styles.handle} />
      <div className={styles.nodeId}>{data.argId}</div>
      {data.label && <div className={styles.nodeLabel}>{data.label}</div>}
    </div>
  );
}

const nodeTypes = {arg: ArgumentNode};

function computePositions(n: number, layout: 'row' | 'circle'): {x: number; y: number}[] {
  if (n === 0) return [];
  if (layout === 'row') {
    const gap = 240;
    const totalWidth = (n - 1) * gap;
    const startX = 80 - totalWidth / 2;
    return Array.from({length: n}, (_, i) => ({x: startX + i * gap, y: 0}));
  }
  const radius = Math.max(140, 55 * n);
  return Array.from({length: n}, (_, i) => {
    const angle = (i / n) * Math.PI * 2 - Math.PI / 2;
    return {x: radius * Math.cos(angle), y: radius * Math.sin(angle)};
  });
}

export default function AttackGraph({
  arguments: args,
  attacks = [],
  supports = [],
  height = 360,
  title,
  layout = 'auto',
  legend = true,
  caption,
}: Props) {
  const chosenLayout: 'row' | 'circle' =
    layout === 'auto' ? (args.length <= 2 ? 'row' : 'circle') : layout;

  const nodes: Node[] = useMemo(() => {
    const positions = computePositions(args.length, chosenLayout);
    return args.map((a, i) => ({
      id: a.id,
      type: 'arg',
      data: {argId: a.id, label: a.label, accepted: a.accepted},
      position: positions[i],
      draggable: true,
    }));
  }, [args, chosenLayout]);

  const edges: Edge[] = useMemo(() => {
    const attackEdges: Edge[] = attacks.map((e, i) => {
      const isUndercut = e.kind === 'undercut';
      const selfLoop = e.from === e.to;
      return {
        id: `attack-${i}`,
        source: e.from,
        target: e.to,
        type: selfLoop ? 'default' : 'smoothstep',
        label: e.weight !== undefined ? e.weight.toFixed(2) : undefined,
        style: {
          stroke: 'var(--ag-attack)',
          strokeWidth: 2,
          strokeDasharray: isUndercut ? '6 4' : undefined,
        },
        labelStyle: {fontSize: 11, fontWeight: 700, fill: 'var(--ag-attack)'},
        labelBgStyle: {fill: 'var(--ag-surface)', fillOpacity: 0.95},
        labelBgPadding: [4, 2],
        labelBgBorderRadius: 4,
        markerEnd: {
          type: MarkerType.ArrowClosed,
          color: 'var(--ag-attack)',
          width: 22,
          height: 22,
        },
      };
    });
    const supportEdges: Edge[] = supports.map((e, i) => ({
      id: `support-${i}`,
      source: e.from,
      target: e.to,
      type: 'smoothstep',
      label: e.weight !== undefined ? e.weight.toFixed(2) : undefined,
      style: {stroke: 'var(--ag-support)', strokeWidth: 2},
      labelStyle: {fontSize: 11, fontWeight: 700, fill: 'var(--ag-support)'},
      labelBgStyle: {fill: 'var(--ag-surface)', fillOpacity: 0.95},
      labelBgPadding: [4, 2],
      labelBgBorderRadius: 4,
      markerEnd: {
        type: MarkerType.ArrowClosed,
        color: 'var(--ag-support)',
        width: 22,
        height: 22,
      },
    }));
    return [...attackEdges, ...supportEdges];
  }, [attacks, supports]);

  const hasUndercut = attacks.some((e) => e.kind === 'undercut');

  return (
    <figure className={styles.wrapper}>
      {title && <div className={styles.title}>{title}</div>}
      <div className={styles.canvas} style={{height}}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          fitView
          fitViewOptions={{padding: 0.25, maxZoom: 1.1}}
          nodesConnectable={false}
          elementsSelectable={false}
          panOnDrag
          zoomOnScroll={false}
          zoomOnPinch
          zoomOnDoubleClick={false}
          proOptions={{hideAttribution: true}}
        >
          <Background gap={18} size={1} color="var(--ag-grid)" />
          <Controls showInteractive={false} />
        </ReactFlow>
      </div>
      {legend && (
        <div className={styles.legend}>
          {attacks.length > 0 && (
            <span className={styles.legendItem}>
              <svg width="28" height="10" aria-hidden="true">
                <line x1="0" y1="5" x2="22" y2="5" stroke="var(--ag-attack)" strokeWidth="2" />
                <polygon points="22,1 28,5 22,9" fill="var(--ag-attack)" />
              </svg>
              attacks
            </span>
          )}
          {hasUndercut && (
            <span className={styles.legendItem}>
              <svg width="28" height="10" aria-hidden="true">
                <line
                  x1="0"
                  y1="5"
                  x2="22"
                  y2="5"
                  stroke="var(--ag-attack)"
                  strokeWidth="2"
                  strokeDasharray="5 3"
                />
                <polygon points="22,1 28,5 22,9" fill="var(--ag-attack)" />
              </svg>
              undercuts
            </span>
          )}
          {supports.length > 0 && (
            <span className={styles.legendItem}>
              <svg width="28" height="10" aria-hidden="true">
                <line x1="0" y1="5" x2="22" y2="5" stroke="var(--ag-support)" strokeWidth="2" />
                <polygon points="22,1 28,5 22,9" fill="var(--ag-support)" />
              </svg>
              supports
            </span>
          )}
          <span className={styles.legendHint}>drag nodes · scroll to pan</span>
        </div>
      )}
      {caption && <figcaption className={styles.caption}>{caption}</figcaption>}
    </figure>
  );
}

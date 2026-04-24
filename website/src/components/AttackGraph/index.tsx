import React from 'react';
import ReactFlow, {Background, Controls, MarkerType, type Edge, type Node} from 'reactflow';
import 'reactflow/dist/style.css';
import styles from './styles.module.css';

export type ArgNode = {id: string; label?: string; accepted?: boolean};
export type ArgEdge = {from: string; to: string; weight?: number; kind?: 'attack' | 'support'};

type Props = {
  arguments: ArgNode[];
  attacks?: ArgEdge[];
  supports?: ArgEdge[];
  height?: number;
  title?: string;
};

export default function AttackGraph({arguments: args, attacks = [], supports = [], height = 300, title}: Props) {
  const count = args.length;
  const radius = Math.max(80, 40 * count);
  const nodes: Node[] = args.map((a, i) => {
    const angle = (i / Math.max(count, 1)) * Math.PI * 2 - Math.PI / 2;
    return {
      id: a.id,
      data: {label: a.label ?? a.id},
      position: {x: 200 + radius * Math.cos(angle), y: 150 + radius * Math.sin(angle)},
      style: {
        background: a.accepted === true ? 'var(--ifm-color-success-contrast-background)'
                  : a.accepted === false ? 'var(--ifm-color-danger-contrast-background)'
                  : undefined,
        border: '1px solid var(--ifm-color-emphasis-400)',
        borderRadius: 8,
        padding: 8,
        fontSize: 13,
      },
    };
  });

  const attackEdges: Edge[] = attacks.map((e, i) => ({
    id: `a-${i}`,
    source: e.from,
    target: e.to,
    label: e.weight !== undefined ? e.weight.toFixed(2) : undefined,
    style: {stroke: 'var(--ifm-color-danger)'},
    markerEnd: {type: MarkerType.ArrowClosed, color: 'var(--ifm-color-danger)'},
  }));
  const supportEdges: Edge[] = supports.map((e, i) => ({
    id: `s-${i}`,
    source: e.from,
    target: e.to,
    label: e.weight !== undefined ? e.weight.toFixed(2) : undefined,
    style: {stroke: 'var(--ifm-color-success)'},
    markerEnd: {type: MarkerType.ArrowClosed, color: 'var(--ifm-color-success)'},
  }));

  return (
    <div className={styles.wrapper} style={{height}}>
      {title && <div className={styles.title}>{title}</div>}
      <ReactFlow
        nodes={nodes}
        edges={[...attackEdges, ...supportEdges]}
        fitView
        nodesDraggable
        proOptions={{hideAttribution: true}}
      >
        <Background />
        <Controls showInteractive={false} />
      </ReactFlow>
    </div>
  );
}

import React, { useMemo } from 'react';
import ForceGraph2D from 'react-force-graph-2d';
import { Share2, Loader2, Info } from 'lucide-react';
import { PageHeader } from './Headers';

export const TopologyPage = ({ topology, loading }) => {
    const graphData = useMemo(() => {
        if (!topology?.nodes) return { nodes: [], links: [] };

        const nodes = [
            { id: 'hub', label: 'HUB', type: 'central', color: '#5E6AD2', size: 8 }
        ];
        const links = [];

        topology.nodes.forEach(n => {
            nodes.push({
                id: `node-${n.node_id}`,
                label: `Node ${n.node_id}`,
                type: 'node',
                color: '#4ADE80',
                size: 5,
                eventCount: n.event_count
            });
            links.push({ source: 'hub', target: `node-${n.node_id}`, val: 1 });

            n.streams.forEach(s => {
                const streamId = `stream-${n.node_id}-${s}`;
                nodes.push({
                    id: streamId, label: `S:${s}`, type: 'stream',
                    color: '#FBBF24', size: 3
                });
                links.push({ source: `node-${n.node_id}`, target: streamId, val: 0.5 });
            });
        });

        return { nodes, links };
    }, [topology]);

    return (
        <div className="animate-fade-in h-[calc(100vh-100px)] flex flex-col">
            <PageHeader title="Topology" subtitle="Causal network and stream distribution" />

            <div className="flex-1 min-h-0 bg-bg-elevated border border-border rounded-lg relative overflow-hidden">
                {loading ? (
                    <div className="absolute inset-0 z-10 flex items-center justify-center bg-bg/60">
                        <Loader2 className="animate-spin text-fg-subtle" size={18} />
                    </div>
                ) : (
                    <ForceGraph2D
                        graphData={graphData}
                        nodeLabel={n => `${n.label}${n.eventCount ? ` (${n.eventCount} events)` : ''}`}
                        nodeColor={n => n.color}
                        nodeRelSize={n => n.size || 4}
                        linkColor={() => 'rgba(255,255,255,0.06)'}
                        backgroundColor="transparent"
                        showNavInfo={false}
                        height={600}
                        d3AlphaDecay={0.05}
                        d3VelocityDecay={0.1}
                    />
                )}

                {/* Legend */}
                <div className="absolute top-3 left-3 flex flex-col gap-1">
                    <Legend color="#5E6AD2" label="Hub" />
                    <Legend color="#4ADE80" label="Node" />
                    <Legend color="#FBBF24" label="Stream" />
                </div>

                <div className="absolute bottom-3 right-3 flex items-center gap-1.5 px-2.5 py-1.5 bg-bg-overlay border border-border rounded-md text-[11px] text-fg-subtle">
                    <Info size={11} /> Drag to explore
                </div>
            </div>
        </div>
    );
};

const Legend = ({ color, label }) => (
    <div className="flex items-center gap-1.5 px-2 py-1 bg-bg-overlay border border-border rounded-md">
        <div className="w-2 h-2 rounded-full" style={{ backgroundColor: color }} />
        <span className="text-[10px] font-medium text-fg-subtle">{label}</span>
    </div>
);

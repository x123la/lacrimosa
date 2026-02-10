import React, { useState, useCallback, useEffect } from 'react';
import ReactFlow, {
    addEdge,
    MiniMap,
    Controls,
    Background,
    useNodesState,
    useEdgesState,
    Handle,
    Position
} from 'reactflow';
import 'reactflow/dist/style.css';
import { Settings, Play, Square, Trash2, Plus, ArrowRight, Save } from 'lucide-react';

const NodeWrapper = ({ data, children, color = 'border-border' }) => (
    <div className={`bg-bg-elevated border-2 ${color} rounded-md min-w-[150px] shadow-lg`}>
        <div className="px-3 py-1.5 border-b border-border bg-white/5 text-xs font-medium flex justify-between items-center">
            <span>{data.label}</span>
            <Settings size={10} className="text-fg-muted cursor-pointer hover:text-fg" />
        </div>
        <div className="p-3 text-xs text-fg-muted">{children}</div>
    </div>
);

const SourceNode = ({ data }) => (
    <NodeWrapper data={data} color="border-blue-500/50">
        <div className="flex items-center gap-2">
            <ArrowRight size={12} />
            {data.connectorName || 'Select Connector'}
        </div>
        <Handle type="source" position={Position.Right} className="w-2 h-2 bg-blue-500" />
    </NodeWrapper>
);

const FilterNode = ({ data }) => (
    <NodeWrapper data={data} color="border-yellow-500/50">
        <Handle type="target" position={Position.Left} className="w-2 h-2 bg-yellow-500" />
        <div className="font-mono">{data.field || 'field'} {data.op || '=='} {data.value || '?'}</div>
        <Handle type="source" position={Position.Right} className="w-2 h-2 bg-yellow-500" />
    </NodeWrapper>
);

const SinkNode = ({ data }) => (
    <NodeWrapper data={data} color="border-green-500/50">
        <Handle type="target" position={Position.Left} className="w-2 h-2 bg-green-500" />
        <div className="flex items-center gap-2">
            <ArrowRight size={12} />
            {data.output || 'Output Stream'}
        </div>
    </NodeWrapper>
);

const nodeTypes = {
    source: SourceNode,
    filter: FilterNode,
    sink: SinkNode,
};

const defaultFlow = () => ({
    nodes: [
        { id: '1', type: 'source', position: { x: 50, y: 100 }, data: { label: 'Source', connectorName: 'journal' } },
        { id: '2', type: 'filter', position: { x: 260, y: 100 }, data: { label: 'Filter', field: 'stream_id', op: '==', value: '1' } },
        { id: '3', type: 'sink', position: { x: 470, y: 100 }, data: { label: 'Sink', output: 'clean-stream' } },
    ],
    edges: [{ id: 'e1-2', source: '1', target: '2' }, { id: 'e2-3', source: '2', target: '3' }],
});

const toFlow = (pipeline) => {
    if (!pipeline || !pipeline.nodes || pipeline.nodes.length === 0) {
        return defaultFlow();
    }

    const nodes = pipeline.nodes.map((node) => ({
        id: node.id,
        type: node.node_type,
        position: node.position || { x: 100, y: 100 },
        data: {
            label: node.node_type?.replace('_', ' ')?.replace(/^\w/, (c) => c.toUpperCase()) || 'Node',
            ...(node.config || {}),
        },
    }));

    const edges = (pipeline.edges || []).map((edge, idx) => ({
        id: `e-${edge.from_node}-${edge.to_node}-${idx}`,
        source: edge.from_node,
        target: edge.to_node,
    }));

    return { nodes, edges };
};

const fromFlow = (nodes, edges) => ({
    nodes: nodes.map((node) => {
        const { label, ...config } = node.data || {};
        return {
            id: node.id,
            node_type: node.type || 'filter',
            config,
            position: node.position,
        };
    }),
    edges: edges.map((edge) => ({
        from_node: edge.source,
        to_node: edge.target,
    })),
});

export const PipelinesPage = () => {
    const [pipelines, setPipelines] = useState([]);
    const [selectedPipeline, setSelectedPipeline] = useState(null);
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState([]);

    useEffect(() => {
        fetchPipelines();
    }, []);

    const fetchPipelines = async () => {
        const res = await fetch('/api/pipelines');
        if (!res.ok) return;
        const data = await res.json();
        setPipelines(data);
    };

    const loadPipeline = (pipeline) => {
        setSelectedPipeline(pipeline);
        const flow = toFlow(pipeline);
        setNodes(flow.nodes);
        setEdges(flow.edges);
    };

    const onConnect = useCallback((params) => setEdges((eds) => addEdge(params, eds)), [setEdges]);

    const createPipeline = async () => {
        const name = prompt('Pipeline Name:');
        if (!name) return;
        const flow = defaultFlow();
        const payload = fromFlow(flow.nodes, flow.edges);
        const res = await fetch('/api/pipelines', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name, nodes: payload.nodes, edges: payload.edges })
        });
        if (!res.ok) return;
        const newPipe = await res.json();
        setPipelines((prev) => [...prev, newPipe]);
        loadPipeline(newPipe);
    };

    const savePipeline = async () => {
        if (!selectedPipeline) return;
        const payload = fromFlow(nodes, edges);
        const res = await fetch(`/api/pipelines/${selectedPipeline.id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
        });
        if (!res.ok) return;
        const updated = await res.json();
        setPipelines((prev) => prev.map((pipeline) => pipeline.id === updated.id ? updated : pipeline));
        setSelectedPipeline(updated);
    };

    const deletePipeline = async (id) => {
        if (!confirm('Are you sure?')) return;
        await fetch(`/api/pipelines/${id}`, { method: 'DELETE' });
        setPipelines((prev) => prev.filter((pipeline) => pipeline.id !== id));
        if (selectedPipeline?.id === id) {
            setSelectedPipeline(null);
            setNodes([]);
            setEdges([]);
        }
    };

    const toggleStatus = async (id, currentStatus) => {
        const action = currentStatus === 'running' ? 'stop' : 'run';
        await fetch(`/api/pipelines/${id}/${action}`, { method: 'POST' });
        await fetchPipelines();
    };

    return (
        <div className="flex h-full">
            <div className="w-64 border-r border-border bg-bg-surface flex flex-col">
                <div className="p-3 border-b border-border flex justify-between items-center">
                    <span className="text-xs font-bold text-fg-muted uppercase">Pipelines</span>
                    <button onClick={createPipeline} className="p-1 hover:bg-white/10 rounded">
                        <Plus size={14} />
                    </button>
                </div>
                <div className="flex-1 overflow-y-auto">
                    {pipelines.map((pipeline) => (
                        <div
                            key={pipeline.id}
                            onClick={() => loadPipeline(pipeline)}
                            className={`p-3 border-b border-border cursor-pointer hover:bg-white/5 ${selectedPipeline?.id === pipeline.id ? 'bg-accent/10 border-l-2 border-l-accent' : 'border-l-2 border-l-transparent'}`}
                        >
                            <div className="font-medium text-sm truncate">{pipeline.name}</div>
                            <div className="flex justify-between items-center mt-2">
                                <span className={`text-[10px] uppercase font-bold px-1.5 py-0.5 rounded ${pipeline.status === 'running' ? 'bg-green-500/20 text-green-400' : 'bg-zinc-700 text-zinc-400'}`}>
                                    {pipeline.status}
                                </span>
                                <div className="flex gap-2">
                                    <button onClick={(event) => { event.stopPropagation(); toggleStatus(pipeline.id, pipeline.status); }}>
                                        {pipeline.status === 'running' ? <Square size={12} fill="currentColor" /> : <Play size={12} fill="currentColor" />}
                                    </button>
                                    <button onClick={(event) => { event.stopPropagation(); deletePipeline(pipeline.id); }}>
                                        <Trash2 size={12} />
                                    </button>
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            <div className="flex-1 h-full bg-bg relative">
                {selectedPipeline ? (
                    <div className="h-full w-full">
                        <div className="absolute top-4 right-4 z-10 flex gap-2">
                            <button onClick={savePipeline} className="bg-accent text-white px-3 py-1.5 rounded-md text-xs font-medium flex items-center gap-1.5 hover:bg-accent/90">
                                <Save size={14} /> Save Layout
                            </button>
                        </div>
                        <ReactFlow
                            nodes={nodes}
                            edges={edges}
                            onNodesChange={onNodesChange}
                            onEdgesChange={onEdgesChange}
                            onConnect={onConnect}
                            nodeTypes={nodeTypes}
                            fitView
                            className="bg-bg-elevated"
                        >
                            <Background color="#333" gap={16} />
                            <Controls className="bg-bg-surface border-border fill-fg" />
                            <MiniMap className="bg-bg-surface border-border" />
                        </ReactFlow>
                    </div>
                ) : (
                    <div className="flex items-center justify-center h-full text-fg-muted">
                        Select or create a pipeline
                    </div>
                )}
            </div>
        </div>
    );
};


import React, { useState, useEffect, useMemo } from 'react';
import { Search, AlertCircle, Clock, GitCommit, ChevronRight, ChevronDown, Activity, LayoutList } from 'lucide-react';

export const TracesPage = () => {
    const [traces, setTraces] = useState([]);
    const [selectedTraceId, setSelectedTraceId] = useState(null);
    const [loading, setLoading] = useState(false);
    const [filter, setFilter] = useState('');

    useEffect(() => {
        fetchTraces();
        const interval = setInterval(fetchTraces, 5000);
        return () => clearInterval(interval);
    }, []);

    const fetchTraces = async () => {
        try {
            const res = await fetch('/api/traces?limit=50');
            const data = await res.json();
            setTraces(data);
        } catch (e) {
            console.error(e);
        }
    };

    const selectedTrace = useMemo(() =>
        traces.find(t => t.trace_id === selectedTraceId),
        [traces, selectedTraceId]);

    const filteredTraces = useMemo(() =>
        traces.filter(t => t.trace_id.includes(filter) || t.root_span?.name.includes(filter)),
        [traces, filter]);

    return (
        <div className="flex h-full animate-fade-in">
            {/* Sidebar List */}
            <div className="w-80 border-r border-border flex flex-col bg-bg-surface">
                <div className="p-3 border-b border-border">
                    <div className="relative">
                        <Search size={14} className="absolute left-2.5 top-2.5 text-fg-muted" />
                        <input
                            type="text"
                            placeholder="Filter traces..."
                            value={filter}
                            onChange={e => setFilter(e.target.value)}
                            className="w-full bg-bg-elevated border border-border rounded pl-8 pr-3 py-1.5 text-xs focus:outline-none focus:border-accent transition-colors"
                        />
                    </div>
                </div>
                <div className="flex-1 overflow-y-auto">
                    {filteredTraces.map(trace => (
                        <div
                            key={trace.trace_id}
                            onClick={() => setSelectedTraceId(trace.trace_id)}
                            className={`p-3 border-b border-white/5 cursor-pointer hover:bg-white/5 transition-colors ${selectedTraceId === trace.trace_id ? 'bg-accent/10 border-l-2 border-l-accent' : 'border-l-2 border-l-transparent'}`}
                        >
                            <div className="flex justify-between items-start mb-1">
                                <span className="font-mono text-[11px] text-accent truncate w-24" title={trace.trace_id}>
                                    {trace.trace_id.slice(0, 8)}...
                                </span>
                                <span className="text-[10px] text-fg-muted">
                                    {new Date(trace.start_time).toLocaleTimeString()}
                                </span>
                            </div>
                            <div className="font-medium text-xs truncate mb-1 text-fg">
                                {trace.root_span?.name || '(root missing)'}
                            </div>
                            <div className="flex items-center gap-3 text-[10px] text-fg-muted">
                                <span className="flex items-center gap-1">
                                    <Clock size={10} /> {trace.duration_ms}ms
                                </span>
                                <span className="flex items-center gap-1">
                                    <GitCommit size={10} /> {trace.spans.length}
                                </span>
                                {trace.error_count > 0 && (
                                    <span className="flex items-center gap-1 text-red-400 font-bold">
                                        <AlertCircle size={10} /> {trace.error_count}
                                    </span>
                                )}
                            </div>
                        </div>
                    ))}
                    {filteredTraces.length === 0 && (
                        <div className="p-8 text-center text-fg-muted text-xs">
                            No traces found
                        </div>
                    )}
                </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex flex-col bg-bg overflow-hidden">
                {selectedTrace ? (
                    <TraceDetailView trace={selectedTrace} />
                ) : (
                    <div className="flex-1 flex items-center justify-center text-fg-muted">
                        <div className="text-center">
                            <Activity size={48} className="mx-auto mb-4 opacity-20" />
                            <p className="text-sm">Select a trace to view details</p>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};

const TraceDetailView = ({ trace }) => {
    // Basic waterfall calculation
    const minStart = Math.min(...trace.spans.map(s => s.start_time_unix_nano));
    const maxEnd = Math.max(...trace.spans.map(s => s.end_time_unix_nano));
    const totalDuration = maxEnd - minStart;

    const getLeft = (start) => ((start - minStart) / totalDuration) * 100;
    const getWidth = (start, end) => Math.max(((end - start) / totalDuration) * 100, 0.5); // min width 0.5%

    return (
        <div className="flex flex-col h-full">
            <div className="h-14 border-b border-border flex items-center justify-between px-6 bg-bg-surface">
                <div>
                    <h2 className="text-sm font-bold text-fg">{trace.root_span?.name || 'Trace Trace'}</h2>
                    <div className="flex items-center gap-2 text-xs text-fg-muted font-mono mt-0.5">
                        <span>{trace.trace_id}</span>
                        <span>•</span>
                        <span>{trace.duration_ms}ms</span>
                        <span>•</span>
                        <span>{trace.services.length} services</span>
                    </div>
                </div>
            </div>

            <div className="flex-1 overflow-y-auto p-6">
                <div className="relative">
                    {/* Time Grid (simplified) */}
                    <div className="absolute inset-0 flex pointer-events-none">
                        {[0, 25, 50, 75, 100].map(p => (
                            <div key={p} className="flex-1 border-r border-white/5 first:border-l relative h-full">
                                <span className="absolute -top-5 right-0 text-[9px] text-fg-faint translate-x-1/2">
                                    {(trace.duration_ms * p / 100).toFixed(0)}ms
                                </span>
                            </div>
                        ))}
                    </div>

                    <div className="space-y-1 relative pt-2">
                        {trace.spans.sort((a, b) => a.start_time_unix_nano - b.start_time_unix_nano).map((span, i) => {
                            const left = getLeft(span.start_time_unix_nano);
                            const width = getWidth(span.start_time_unix_nano, span.end_time_unix_nano);
                            const isError = typeof span.status === 'object' && span.status.Error;

                            return (
                                <div key={span.span_id} className="group relative flex items-center h-7 hover:bg-white/5 rounded">
                                    {/* Label Section - Fixed width or sticky? Simplified here */}
                                    <div className="w-48 shrink-0 px-2 text-xs truncate flex items-center gap-2" style={{ paddingLeft: `${(span.parent_span_id ? 4 : 0)}px` }}> {/* Primitive nesting indentation logic */}
                                        <span className={`w-2 h-2 rounded-full ${getColorForService(span.service_name)}`}></span>
                                        <span className={isError ? 'text-red-400' : 'text-fg-muted group-hover:text-fg'}>{span.name}</span>
                                    </div>

                                    {/* Bar Section */}
                                    <div className="flex-1 relative h-full ml-4">
                                        <div
                                            className={`absolute top-1.5 h-4 rounded-sm text-[9px] flex items-center px-1.5 text-white/90 overflow-hidden whitespace-nowrap transition-all
                                              ${isError ? 'bg-red-500/80 border border-red-400' : 'bg-accent/40 border border-accent/50 group-hover:bg-accent/60'}`}
                                            style={{
                                                left: `${left}%`,
                                                width: `${width}%`,
                                                minWidth: '4px'
                                            }}
                                            title={JSON.stringify(span.attributes, null, 2)}
                                        >
                                            <span className="opacity-0 group-hover:opacity-100 transition-opacity drop-shadow-md">
                                                {((span.end_time_unix_nano - span.start_time_unix_nano) / 1_000_000).toFixed(2)}ms
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                </div>
            </div>
        </div>
    );
};

const getColorForService = (svc) => {
    // Deterministic color from string hash
    const colors = ['bg-blue-500', 'bg-green-500', 'bg-purple-500', 'bg-orange-500', 'bg-cyan-500', 'bg-pink-500'];
    let hash = 0;
    for (let i = 0; i < svc.length; i++) hash = svc.charCodeAt(i) + ((hash << 5) - hash);
    return colors[Math.abs(hash) % colors.length];
};


import React, { useState, useEffect } from 'react';
import { PageHeader } from './Headers';
import { Play, Search, Save, Clock, Download, ChevronRight, ChevronDown, List, Code as CodeIcon, Database } from 'lucide-react';

export const QueryConsolePage = () => {
    const [query, setQuery] = useState('SELECT * FROM journal LIMIT 50');
    const [results, setResults] = useState(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState(null);
    const [history, setHistory] = useState([]);

    const runQuery = async () => {
        setLoading(true);
        setError(null);
        try {
            const res = await fetch('/api/query', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ query })
            });
            const data = await res.json();
            if (!res.ok) throw new Error(data.error || 'Query failed');

            setResults(data);
            setHistory(h => [query, ...h.slice(0, 9)]);
        } catch (err) {
            setError(err.message);
        } finally {
            setLoading(false);
        }
    };

    const handleKeyDown = (e) => {
        if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
            e.preventDefault();
            runQuery();
        }
    };

    const handleHistoryClick = (h) => {
        setQuery(h);
    };

    return (
        <div className="animate-fade-in max-w-7xl mx-auto h-[calc(100vh-8rem)] flex flex-col">
            <PageHeader
                title="Query Console"
                subtitle="Cross-stream SQL-like analysis engine"
                actions={
                    <button
                        onClick={runQuery}
                        disabled={loading}
                        className="flex items-center gap-2 px-4 py-1.5 bg-emerald-500 text-black font-semibold rounded-md
                                 hover:bg-emerald-400 transition-colors text-sm disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        {loading ? <div className="w-4 h-4 border-2 border-black/30 border-t-black rounded-full animate-spin" /> : <Play size={16} />}
                        Run Query
                    </button>
                }
            />

            <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 mt-4 flex-1 min-h-0">
                {/* Main Editor & Results */}
                <div className="lg:col-span-3 flex flex-col gap-4 min-h-0 h-full">
                    <div className="relative flex-none">
                        <textarea
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                            onKeyDown={handleKeyDown}
                            className="w-full h-32 bg-[#0A0A0A] border border-white/10 rounded-lg p-4 font-mono text-sm text-white/90 
                                     focus:border-emerald-500/50 focus:outline-none resize-none"
                            placeholder="SELECT * FROM connectors WHERE..."
                        />
                        <div className="absolute bottom-3 right-3 text-xs text-white/30 pointer-events-none">
                            Cmd + Enter to run
                        </div>
                    </div>

                    {error && (
                        <div className="flex-none bg-red-500/10 border border-red-500/20 text-red-400 px-4 py-3 rounded-lg text-sm font-mono">
                            ERROR: {error}
                        </div>
                    )}

                    {results && (
                        <div className="flex-1 bg-[#0A0A0A] border border-white/10 rounded-lg flex flex-col min-h-0 overflow-hidden">
                            <div className="flex-none flex items-center justify-between px-4 py-2 border-b border-white/5 bg-white/[0.02]">
                                <div className="text-xs text-white/50 flex gap-4 font-mono">
                                    <span>{results.total} events</span>
                                    <span>{results.query_time_ms}ms</span>
                                    <span>Sources: {(results.streams_searched || []).join(', ') || 'all'}</span>
                                </div>
                                <button className="text-white/40 hover:text-white transition-colors" title="Export JSON">
                                    <Download size={14} />
                                </button>
                            </div>

                            <div className="flex-1 overflow-auto">
                                <table className="w-full text-left border-collapse">
                                    <thead className="bg-[#111] sticky top-0 z-10 text-xs font-semibold text-white/50 uppercase tracking-wider">
                                        <tr>
                                            <th className="px-4 py-3 border-b border-white/5 w-12 text-right">#</th>
                                            <th className="px-4 py-3 border-b border-white/5 w-48">Timestamp</th>
                                            <th className="px-4 py-3 border-b border-white/5 w-32">Stream</th>
                                            <th className="px-4 py-3 border-b border-white/5">Payload</th>
                                        </tr>
                                    </thead>
                                    <tbody className="divide-y divide-white/5 text-sm font-mono text-white/80">
                                        {results.events.map((event, i) => (
                                            <EventRow key={event.id || i} event={event} index={i + 1} />
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    )}
                </div>

                {/* Sidebar: History & Schema */}
                <div className="flex flex-col gap-4 h-full min-h-0 overflow-hidden">
                    <div className="flex-none bg-white/5 border border-white/10 rounded-lg p-4 max-h-[40%] overflow-y-auto">
                        <div className="flex items-center gap-2 mb-3 text-white/50 text-xs font-medium uppercase tracking-wider sticky top-0 bg-[#161616] p-1 -m-1 z-10">
                            <Clock size={12} />
                            Recent Queries
                        </div>
                        <div className="space-y-2">
                            {history.map((h, i) => (
                                <button
                                    key={i}
                                    onClick={() => handleHistoryClick(h)}
                                    className="block w-full text-left text-xs font-mono text-white/70 hover:text-white hover:bg-white/5 p-2 rounded truncate transition-colors border border-transparent hover:border-white/10"
                                    title={h}
                                >
                                    {h}
                                </button>
                            ))}
                            {history.length === 0 && (
                                <div className="text-xs text-white/30 italic px-2">No history yet</div>
                            )}
                        </div>
                    </div>

                    <div className="flex-1 bg-white/5 border border-white/10 rounded-lg p-4 overflow-y-auto">
                        <div className="flex items-center gap-2 mb-3 text-white/50 text-xs font-medium uppercase tracking-wider sticky top-0 bg-[#161616] p-1 -m-1 z-10">
                            <Database size={12} />
                            Schema Hint
                        </div>
                        <div className="text-xs text-white/40 leading-relaxed font-mono">
                            <p className="mb-2 text-white/60">Fields:</p>
                            <ul className="list-disc pl-4 space-y-1 mb-4">
                                <li>id (string)</li>
                                <li>stream (string)</li>
                                <li>timestamp (iso8601)</li>
                                <li>sequence (u64)</li>
                                <li>payload (json)</li>
                                <li>connector_id (string)</li>
                            </ul>
                            <p className="mb-2 text-white/60">Operators:</p>
                            <ul className="list-disc pl-4 space-y-1">
                                <li>=, !=</li>
                                <li>&gt;, &gt;=, &lt;, &lt;=</li>
                                <li>CONTAINS</li>
                                <li>STARTSWITH</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

const EventRow = ({ event, index }) => {
    const [expanded, setExpanded] = useState(false);

    return (
        <>
            <tr
                onClick={() => setExpanded(!expanded)}
                className={`cursor-pointer hover:bg-white/[0.04] transition-colors ${expanded ? 'bg-white/[0.06]' : ''}`}
            >
                <td className="px-4 py-2 text-white/30 text-right align-top">{index}</td>
                <td className="px-4 py-2 text-white/60 whitespace-nowrap align-top">{event.timestamp}</td>
                <td className="px-4 py-2 text-emerald-400/80 align-top">{event.stream}</td>
                <td className="px-4 py-2 text-white/70 truncate max-w-xl align-top block">
                    {JSON.stringify(event.payload)}
                </td>
            </tr>
            {expanded && (
                <tr className="bg-white/[0.04]">
                    <td colSpan={4} className="px-4 pb-4 pt-1">
                        <div className="bg-black/50 rounded p-3 overflow-x-auto border border-white/5">
                            <pre className="text-xs text-blue-300 font-mono">
                                {JSON.stringify(event, null, 2)}
                            </pre>
                        </div>
                    </td>
                </tr>
            )}
        </>
    );
};

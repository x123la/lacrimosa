
import React, { useState, useEffect } from 'react';
import {
    AlertTriangle,
    CheckCircle,
    Clock,
    MessageSquare,
    Shield,
    ShieldAlert,
    ShieldCheck,
    Search,
    Filter
} from 'lucide-react';

export const IncidentsPage = () => {
    const [incidents, setIncidents] = useState([]);
    const [filter, setFilter] = useState('open'); // open, all, resolved
    const [selectedIncident, setSelectedIncident] = useState(null);

    useEffect(() => {
        fetchIncidents();
        const interval = setInterval(fetchIncidents, 5000);
        return () => clearInterval(interval);
    }, []);

    const fetchIncidents = async () => {
        try {
            const res = await fetch('/api/alerts/incidents');
            const data = await res.json();
            setIncidents(data);
        } catch (e) {
            console.error(e);
        }
    };

    const handleAction = async (id, action) => {
        try {
            await fetch(`/api/alerts/incidents/${id}/${action}`, { method: 'POST' });
            fetchIncidents();
            if (selectedIncident?.id === id) {
                // Refresh selection details
                const res = await fetch('/api/alerts/incidents');
                const data = await res.json();
                setSelectedIncident(data.find(i => i.id === id));
            }
        } catch (e) {
            console.error(e);
        }
    };

    const filteredIncidents = incidents.filter(i => {
        if (filter === 'open') return i.status !== 'resolved';
        if (filter === 'resolved') return i.status === 'resolved';
        return true;
    }).sort((a, b) => new Date(b.updated_at) - new Date(a.updated_at));

    return (
        <div className="flex h-full animate-fade-in">
            {/* List Sidebar */}
            <div className="w-96 border-r border-border flex flex-col bg-bg-surface">
                <div className="p-3 border-b border-border flex gap-2">
                    <button
                        onClick={() => setFilter('open')}
                        className={`px-3 py-1 text-xs rounded-full border ${filter === 'open' ? 'bg-red-500/20 border-red-500/50 text-red-200' : 'border-border text-fg-muted hover:text-fg'}`}
                    >
                        Open
                    </button>
                    <button
                        onClick={() => setFilter('resolved')}
                        className={`px-3 py-1 text-xs rounded-full border ${filter === 'resolved' ? 'bg-green-500/20 border-green-500/50 text-green-200' : 'border-border text-fg-muted hover:text-fg'}`}
                    >
                        Resolved
                    </button>
                    <button
                        onClick={() => setFilter('all')}
                        className={`px-3 py-1 text-xs rounded-full border ${filter === 'all' ? 'bg-accent/20 border-accent/50 text-accent' : 'border-border text-fg-muted hover:text-fg'}`}
                    >
                        All
                    </button>
                </div>

                <div className="flex-1 overflow-y-auto">
                    {filteredIncidents.map(incident => (
                        <div
                            key={incident.id}
                            onClick={() => setSelectedIncident(incident)}
                            className={`p-4 border-b border-border cursor-pointer hover:bg-white/5 transition-colors ${selectedIncident?.id === incident.id ? 'bg-accent/10 border-l-2 border-l-accent' : 'border-l-2 border-l-transparent'}`}
                        >
                            <div className="flex justify-between items-start mb-1">
                                <div className="flex items-center gap-2">
                                    {incident.severity === 'critical' && <ShieldAlert size={14} className="text-red-500" />}
                                    {incident.severity === 'warning' && <AlertTriangle size={14} className="text-yellow-500" />}
                                    {incident.severity === 'info' && <Shield size={14} className="text-blue-500" />}
                                    <span className="font-medium text-sm text-fg">{incident.rule_name}</span>
                                </div>
                                <span className="text-[10px] text-fg-muted">
                                    {new Date(incident.updated_at).toLocaleTimeString()}
                                </span>
                            </div>
                            <div className="text-xs text-fg-muted truncate mb-2">
                                {incident.message}
                            </div>
                            <div className="flex items-center gap-2">
                                <span className={`text-[10px] px-1.5 rounded uppercase font-bold
                                    ${incident.status === 'open' ? 'bg-red-500/20 text-red-400' :
                                        incident.status === 'acknowledged' ? 'bg-yellow-500/20 text-yellow-400' :
                                            'bg-green-500/20 text-green-400'}`}>
                                    {incident.status}
                                </span>
                                <span className="text-[10px] text-fg-faint font-mono">
                                    {incident.id.slice(0, 8)}
                                </span>
                            </div>
                        </div>
                    ))}
                    {filteredIncidents.length === 0 && (
                        <div className="p-8 text-center text-fg-muted text-xs">
                            No {filter} incidents found.
                        </div>
                    )}
                </div>
            </div>

            {/* Detail View */}
            <div className="flex-1 flex flex-col bg-bg text-fg">
                {selectedIncident ? (
                    <>
                        <div className="p-6 border-b border-border bg-bg-surface">
                            <div className="flex justify-between items-start mb-4">
                                <div>
                                    <h1 className="text-xl font-bold flex items-center gap-2">
                                        {selectedIncident.rule_name}
                                        <span className={`text-xs px-2 py-0.5 rounded-full border ${selectedIncident.severity === 'critical' ? 'border-red-500/30 text-red-400 bg-red-500/10' :
                                                selectedIncident.severity === 'warning' ? 'border-yellow-500/30 text-yellow-400 bg-yellow-500/10' :
                                                    'border-blue-500/30 text-blue-400 bg-blue-500/10'
                                            }`}>
                                            {selectedIncident.severity.toUpperCase()}
                                        </span>
                                    </h1>
                                    <p className="text-fg-muted mt-1 text-sm">{selectedIncident.message}</p>
                                </div>
                                <div className="flex gap-2">
                                    {selectedIncident.status === 'open' && (
                                        <button
                                            onClick={() => handleAction(selectedIncident.id, 'acknowledge')}
                                            className="px-4 py-2 bg-yellow-600/20 text-yellow-200 border border-yellow-600/50 rounded hover:bg-yellow-600/30 flex items-center gap-2 text-sm"
                                        >
                                            <ShieldCheck size={14} /> Acknowledge
                                        </button>
                                    )}
                                    {selectedIncident.status !== 'resolved' && (
                                        <button
                                            onClick={() => handleAction(selectedIncident.id, 'resolve')}
                                            className="px-4 py-2 bg-green-600/20 text-green-200 border border-green-600/50 rounded hover:bg-green-600/30 flex items-center gap-2 text-sm"
                                        >
                                            <CheckCircle size={14} /> Resolve
                                        </button>
                                    )}
                                </div>
                            </div>

                            <div className="grid grid-cols-4 gap-4 text-xs text-fg-muted mt-4 p-4 bg-bg rounded-lg border border-border">
                                <div>
                                    <div className="uppercase text-[10px] font-bold text-fg-faint mb-1">Created</div>
                                    <div className="font-mono">{new Date(selectedIncident.created_at).toLocaleString()}</div>
                                </div>
                                <div>
                                    <div className="uppercase text-[10px] font-bold text-fg-faint mb-1">Status</div>
                                    <div className="font-mono">{selectedIncident.status}</div>
                                </div>
                                <div>
                                    <div className="uppercase text-[10px] font-bold text-fg-faint mb-1">Rule ID</div>
                                    <div className="font-mono">{selectedIncident.rule_id}</div>
                                </div>
                                <div>
                                    <div className="uppercase text-[10px] font-bold text-fg-faint mb-1">ID</div>
                                    <div className="font-mono">{selectedIncident.id}</div>
                                </div>
                            </div>
                        </div>

                        <div className="flex-1 overflow-y-auto p-6">
                            <h3 className="text-sm font-bold text-fg mb-4 flex items-center gap-2">
                                <Clock size={14} /> Timeline
                            </h3>
                            <div className="border-l-2 border-border ml-2 space-y-6 pl-6 relative">
                                {selectedIncident.timeline.map((entry, i) => (
                                    <div key={i} className="relative">
                                        <div className={`absolute -left-[31px] w-3 h-3 rounded-full border-2 border-bg 
                                            ${entry.action === 'opened' ? 'bg-red-500' :
                                                entry.action === 'resolved' ? 'bg-green-500' :
                                                    'bg-bg-elevated/50'}`}></div>
                                        <div className="text-xs text-fg-muted mb-0.5">
                                            {new Date(entry.timestamp).toLocaleString()}
                                        </div>
                                        <div className="text-sm font-medium text-fg">
                                            {entry.detail}
                                        </div>
                                        {entry.actor && (
                                            <div className="text-xs text-fg-faint mt-1 flex items-center gap-1">
                                                <Shield size={10} /> {entry.actor}
                                            </div>
                                        )}
                                    </div>
                                ))}
                            </div>
                        </div>
                    </>
                ) : (
                    <div className="flex-1 flex items-center justify-center text-fg-muted">
                        <div className="text-center">
                            <ShieldAlert size={48} className="mx-auto mb-4 opacity-20" />
                            <p className="text-sm">Select an incident to view details</p>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};

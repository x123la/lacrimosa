
import React, { useState, useEffect } from 'react';
import { PageHeader } from './Headers';
import { Plus, Trash, Play, CircleSlash, Activity, Globe, Database, Server, RefreshCw } from 'lucide-react';

const CONNECTOR_ICONS = {
    journal: Database,
    webhook: Globe,
    kafka: Server,
    nats: Activity, // Using Activity as a placeholder for streaming/messaging
};

const CONNECTOR_LABELS = {
    journal: "Internal Journal",
    webhook: "HTTP Webhook",
    kafka: "Apache Kafka",
    nats: "NATS / JetStream",
};

export const ConnectorsPage = () => {
    const [connectors, setConnectors] = useState([]);
    const [loading, setLoading] = useState(true);
    const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);

    useEffect(() => {
        fetchConnectors();
        const interval = setInterval(fetchConnectors, 5000);
        return () => clearInterval(interval);
    }, []);

    const fetchConnectors = async () => {
        try {
            const res = await fetch('/api/connectors');
            if (res.ok) {
                const data = await res.json();
                setConnectors(data);
            }
        } catch (error) {
            console.error("Failed to fetch connectors", error);
        } finally {
            setLoading(false);
        }
    };

    const handleDelete = async (id) => {
        if (!confirm("Are you sure you want to remove this connector?")) return;
        try {
            await fetch(`/api/connectors/${id}`, { method: 'DELETE' });
            fetchConnectors();
        } catch (error) {
            console.error("Failed to delete connector", error);
        }
    };

    return (
        <div className="animate-fade-in max-w-6xl mx-auto">
            <PageHeader
                title="Data Connectors"
                subtitle="Manage unified stream sources (Kafka, NATS, Http)"
                actions={
                    <button
                        onClick={() => setIsCreateModalOpen(true)}
                        className="flex items-center gap-2 px-3 py-1.5 bg-blue-500/10 text-blue-400 rounded-md
                                 hover:bg-blue-500/20 transition-colors text-sm font-medium border border-blue-500/20"
                    >
                        <Plus size={16} />
                        Add Connector
                    </button>
                }
            />

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mt-6">
                {connectors.map(c => (
                    <ConnectorCard key={c.id} connector={c} onDelete={handleDelete} />
                ))}
            </div>

            {connectors.length === 0 && !loading && (
                <div className="mt-12 text-center p-12 border border-white/5 rounded-lg bg-white/5 mx-auto max-w-lg">
                    <div className="w-12 h-12 rounded-full bg-white/5 flex items-center justify-center mx-auto mb-4">
                        <Server className="text-white/40" />
                    </div>
                    <h3 className="text-lg font-medium text-white mb-2">No connectors configured</h3>
                    <p className="text-white/50 text-sm mb-6">
                        Add a connector to start ingesting events from external sources.
                    </p>
                    <button
                        onClick={() => setIsCreateModalOpen(true)}
                        className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-md text-sm font-medium transition-colors"
                    >
                        Create your first connector
                    </button>
                </div>
            )}

            {isCreateModalOpen && (
                <CreateConnectorModal
                    onClose={() => setIsCreateModalOpen(false)}
                    onCreated={() => {
                        setIsCreateModalOpen(false);
                        fetchConnectors();
                    }}
                />
            )}
        </div>
    );
};

const ConnectorCard = ({ connector, onDelete }) => {
    const Icon = CONNECTOR_ICONS[connector.kind] || Server;
    const isHealthy = connector.status === 'connected';

    return (
        <div className="p-5 rounded-lg border border-white/10 bg-white/5 hover:bg-white/[0.07] transition-colors group relative">
            <div className="flex justify-between items-start mb-4">
                <div className="flex items-center gap-3">
                    <div className={`w-10 h-10 rounded-lg flex items-center justify-center
                        ${isHealthy ? 'bg-emerald-500/10 text-emerald-400' : 'bg-amber-500/10 text-amber-400'}`}>
                        <Icon size={20} />
                    </div>
                    <div>
                        <h4 className="font-medium text-white">{connector.name}</h4>
                        <div className="flex items-center gap-2 mt-0.5">
                            <span className="text-xs text-white/40 font-mono uppercase tracking-wider">
                                {CONNECTOR_LABELS[connector.kind] || connector.kind}
                            </span>
                            <span className={`w-1.5 h-1.5 rounded-full ${isHealthy ? 'bg-emerald-500' : 'bg-amber-500'}`} />
                        </div>
                    </div>
                </div>

                {connector.kind !== 'journal' && (
                    <button
                        onClick={() => onDelete(connector.id)}
                        className="p-1.5 text-white/20 hover:text-red-400 hover:bg-red-500/10 rounded-md transition-colors opacity-0 group-hover:opacity-100"
                        title="Delete connector"
                    >
                        <Trash size={16} />
                    </button>
                )}
            </div>

            {/* Metrics */}
            <div className="grid grid-cols-2 gap-4 pt-4 border-t border-white/5">
                <div>
                    <div className="text-xs text-white/40 mb-1">Events</div>
                    <div className="text-sm font-mono text-white/80">
                        {connector.metrics.events_total.toLocaleString()}
                    </div>
                </div>
                <div>
                    <div className="text-xs text-white/40 mb-1">Rate</div>
                    <div className="text-sm font-mono text-white/80">
                        {connector.metrics.events_per_sec.toFixed(1)}/s
                    </div>
                </div>
                <div>
                    <div className="text-xs text-white/40 mb-1">Bytes</div>
                    <div className="text-sm font-mono text-white/80">
                        {(connector.metrics.bytes_total / 1024 / 1024).toFixed(1)} MB
                    </div>
                </div>
                {connector.kind === 'webhook' && (
                    <div className="col-span-2 mt-2 pt-2 border-t border-white/5">
                        <div className="text-xs text-white/40 mb-1">Endpoint ID</div>
                        <div className="text-xs font-mono text-blue-400/80 break-all select-all">
                            {connector.id}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};

const CreateConnectorModal = ({ onClose, onCreated }) => {
    const [kind, setKind] = useState('webhook');
    const [name, setName] = useState('');
    const [config, setConfig] = useState({});
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [error, setError] = useState(null);

    const handleSubmit = async (e) => {
        e.preventDefault();
        setIsSubmitting(true);
        setError(null);

        // Prepare params based on kind
        const params = {};
        if (kind === 'webhook') {
            params.provider = config.provider || 'generic';
        } else if (kind === 'kafka') {
            params.brokers = config.brokers || 'localhost:9092';
            params.topic = config.topic || 'events';
        }

        try {
            const res = await fetch('/api/connectors', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name,
                    kind,
                    params
                })
            });

            if (!res.ok) throw new Error(await res.text());
            onCreated();
        } catch (err) {
            setError(err.message);
            setIsSubmitting(false);
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
            <div className="bg-[#111111] border border-white/10 rounded-xl w-full max-w-md p-6 shadow-2xl">
                <h2 className="text-lg font-medium text-white mb-1">New Connector</h2>
                <p className="text-white/40 text-sm mb-6">Add a new data source to the control center.</p>

                {error && (
                    <div className="bg-red-500/10 border border-red-500/20 text-red-400 px-3 py-2 rounded-md text-sm mb-4">
                        {error}
                    </div>
                )}

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <label className="block text-xs font-medium text-white/50 mb-1.5">Capabilities</label>
                        <div className="grid grid-cols-2 gap-2">
                            {['webhook', 'kafka', 'nats'].map(k => (
                                <button
                                    type="button"
                                    key={k}
                                    onClick={() => setKind(k)}
                                    className={`px-3 py-2 rounded-md text-sm text-center border transition-colors
                                        ${kind === k
                                            ? 'bg-blue-600 text-white border-blue-500'
                                            : 'bg-white/5 text-white/60 border-transparent hover:bg-white/10'}`}
                                >
                                    {CONNECTOR_LABELS[k]}
                                </button>
                            ))}
                        </div>
                    </div>

                    <div>
                        <label className="block text-xs font-medium text-white/50 mb-1.5">Name</label>
                        <input
                            type="text"
                            value={name}
                            onChange={e => setName(e.target.value)}
                            placeholder="e.g. Production Kafka"
                            className="w-full bg-black/40 border border-white/10 rounded-md px-3 py-2 text-white text-sm focus:border-blue-500 focus:outline-none"
                            required
                        />
                    </div>

                    {kind === 'webhook' && (
                        <div>
                            <label className="block text-xs font-medium text-white/50 mb-1.5">Provider</label>
                            <select
                                value={config.provider || 'generic'}
                                onChange={e => setConfig({ ...config, provider: e.target.value })}
                                className="w-full bg-black/40 border border-white/10 rounded-md px-3 py-2 text-white text-sm focus:border-blue-500 focus:outline-none"
                            >
                                <option value="generic">Generic JSON</option>
                                <option value="github">GitHub</option>
                                <option value="stripe">Stripe</option>
                                <option value="pagerduty">PagerDuty</option>
                            </select>
                        </div>
                    )}

                    {kind === 'kafka' && (
                        <>
                            <div>
                                <label className="block text-xs font-medium text-white/50 mb-1.5">Brokers</label>
                                <input
                                    type="text"
                                    value={config.brokers || ''}
                                    onChange={e => setConfig({ ...config, brokers: e.target.value })}
                                    placeholder="localhost:9092"
                                    className="w-full bg-black/40 border border-white/10 rounded-md px-3 py-2 text-white text-sm focus:border-blue-500 focus:outline-none"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-medium text-white/50 mb-1.5">Topic</label>
                                <input
                                    type="text"
                                    value={config.topic || ''}
                                    onChange={e => setConfig({ ...config, topic: e.target.value })}
                                    placeholder="events"
                                    className="w-full bg-black/40 border border-white/10 rounded-md px-3 py-2 text-white text-sm focus:border-blue-500 focus:outline-none"
                                />
                            </div>
                        </>
                    )}

                    <div className="flex justify-end gap-3 mt-6">
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-4 py-2 text-white/60 hover:text-white text-sm font-medium transition-colors"
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            disabled={isSubmitting}
                            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-md text-sm font-medium transition-colors disabled:opacity-50"
                        >
                            {isSubmitting ? 'Creating...' : 'Create Connector'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
};

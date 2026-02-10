import React from 'react';
import { BarChart3, TrendingUp } from 'lucide-react';
import { PageHeader } from './Headers';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Cell } from 'recharts';

export const AnalyticsPage = ({ data, loading }) => {
    if (loading || !data?.streams) return (
        <div className="p-20 text-center text-fg-subtle text-[13px] animate-pulse">Aggregating stream metrics…</div>
    );

    const chartData = data.streams.map(s => ({
        name: `S:${s.stream_id}`,
        events: s.event_count,
        nodes: s.nodes.length
    })).sort((a, b) => b.events - a.events);

    return (
        <div className="animate-fade-in flex flex-col gap-5">
            <PageHeader title="Stream Analytics" subtitle="Event distribution across logical data streams" />

            <div className="grid grid-cols-3 gap-3 h-[500px]">
                {/* Chart */}
                <div className="col-span-2 bg-bg-elevated border border-border rounded-lg p-5 flex flex-col">
                    <div className="flex items-center gap-2 mb-5">
                        <TrendingUp size={14} strokeWidth={1.6} className="text-fg-subtle" />
                        <h4 className="text-[13px] font-semibold text-fg">Events Per Stream</h4>
                    </div>

                    <div className="flex-1 min-h-0">
                        <ResponsiveContainer width="100%" height="100%">
                            <BarChart data={chartData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
                                <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="rgba(255,255,255,0.025)" />
                                <XAxis dataKey="name" axisLine={false} tickLine={false}
                                    tick={{ fontSize: 10, fill: 'var(--color-fg-faint)' }} />
                                <YAxis axisLine={false} tickLine={false}
                                    tick={{ fontSize: 10, fill: 'var(--color-fg-faint)' }} />
                                <Tooltip
                                    cursor={{ fill: 'rgba(255,255,255,0.02)' }}
                                    contentStyle={{
                                        backgroundColor: 'var(--color-bg-overlay)',
                                        border: '1px solid var(--color-border)',
                                        borderRadius: '6px', fontSize: '11px',
                                        boxShadow: '0 8px 24px rgba(0,0,0,0.4)'
                                    }}
                                />
                                <Bar dataKey="events" radius={[3, 3, 0, 0]}>
                                    {chartData.map((_, index) => (
                                        <Cell key={index} fill="var(--color-accent)" fillOpacity={index % 2 === 0 ? 0.7 : 0.5} />
                                    ))}
                                </Bar>
                            </BarChart>
                        </ResponsiveContainer>
                    </div>
                </div>

                {/* Stream list */}
                <div className="flex flex-col gap-2 overflow-y-auto">
                    <h4 className="text-[11px] font-semibold text-fg-subtle uppercase tracking-wide px-1 mb-1">Streams</h4>
                    {chartData.map((s, i) => (
                        <div key={i} className="bg-bg-elevated border border-border rounded-lg p-3 flex items-center justify-between hover:bg-bg-surface transition-colors">
                            <div className="flex items-center gap-3">
                                <span className="text-[14px] font-mono text-fg-faint w-6">{(i + 1).toString().padStart(2, '0')}</span>
                                <div>
                                    <div className="text-[13px] font-semibold text-fg">{s.name}</div>
                                    <div className="text-[11px] text-fg-subtle">{s.nodes} nodes</div>
                                </div>
                            </div>
                            <div className="text-right">
                                <div className="text-[14px] font-mono font-semibold text-accent">{s.events.toLocaleString()}</div>
                                <div className="text-[10px] text-fg-faint">events</div>
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

import React from 'react';
import { Activity } from 'lucide-react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

const CustomTooltip = ({ active, payload, label }) => {
    if (active && payload && payload.length) {
        return (
            <div className="bg-bg-overlay border border-border rounded-md px-3 py-2 shadow-strong animate-fade-in backdrop-blur-md">
                <p className="text-[10px] font-bold text-fg-faint uppercase tracking-wider mb-1.5">{label}</p>
                {payload.map((item, i) => (
                    <div key={i} className="flex items-center justify-between gap-8 py-0.5">
                        <div className="flex items-center gap-2">
                            <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: item.color }} />
                            <span className="text-[12px] text-fg-muted font-medium">{item.name}</span>
                        </div>
                        <span className="text-[12px] font-mono font-bold text-fg">{item.value.toLocaleString()}</span>
                    </div>
                ))}
            </div>
        );
    }
    return null;
};

export const MainThroughputChart = ({ data }) => (
    <div className="bg-bg-elevated border border-border rounded-lg p-5 h-[380px] flex flex-col group/chart shadow-subtle hover:shadow-medium transition-all">
        <div className="flex justify-between items-center mb-6">
            <div className="flex items-center gap-2.5">
                <div className="p-1.5 bg-accent/10 rounded">
                    <Activity size={14} strokeWidth={2} className="text-accent" />
                </div>
                <div>
                    <h3 className="text-[13px] font-bold text-fg tracking-tight">System Throughput</h3>
                    <p className="text-[10px] text-fg-faint uppercase font-bold tracking-widest mt-0.5">Real-time IO activity</p>
                </div>
            </div>
            <div className="flex items-center gap-2 px-2 py-1 bg-bg-surface border border-border rounded-md">
                <div className="w-1.5 h-1.5 rounded-full bg-green animate-pulse-dot" />
                <span className="text-[11px] text-fg-muted font-mono font-bold">LIVE</span>
            </div>
        </div>

        <div className="flex-1 min-h-0">
            <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={data}>
                    <defs>
                        <linearGradient id="tpsGrad" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stopColor="var(--color-accent)" stopOpacity={0.2} />
                            <stop offset="100%" stopColor="var(--color-accent)" stopOpacity={0} />
                        </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="4 4" vertical={false} stroke="rgba(255,255,255,0.03)" />
                    <XAxis
                        dataKey="time"
                        tick={{ fontSize: 10, fill: 'var(--color-fg-faint)', fontWeight: 600 }}
                        axisLine={false} tickLine={false}
                        interval="preserveStartEnd" minTickGap={60}
                    />
                    <YAxis
                        tick={{ fontSize: 10, fill: 'var(--color-fg-faint)', fontWeight: 600 }}
                        axisLine={false} tickLine={false} width={40}
                        tickFormatter={val => val >= 1000 ? `${(val / 1000).toFixed(1)}k` : val}
                    />
                    <Tooltip
                        content={<CustomTooltip />}
                        cursor={{ stroke: 'rgba(255,255,255,0.1)', strokeWidth: 1 }}
                    />
                    <Area
                        type="monotone" dataKey="tps" name="TPS"
                        stroke="var(--color-accent)" strokeWidth={2}
                        fill="url(#tpsGrad)" isAnimationActive={false}
                        dot={false} activeDot={{ r: 4, fill: 'var(--color-accent)', stroke: 'var(--color-bg)', strokeWidth: 2 }}
                    />
                </AreaChart>
            </ResponsiveContainer>
        </div>
    </div>
);

import React from 'react';
import { AreaChart, Area, ResponsiveContainer } from 'recharts';

export const StatCard = ({ title, value, icon, color = 'accent', sparkline, sub, loading }) => {
    const lineColor = {
        accent: 'var(--color-accent)',
        green: 'var(--color-green)',
        amber: 'var(--color-amber)',
        red: 'var(--color-red)',
    }[color] || 'var(--color-accent)';

    return (
        <div className={`bg-bg-elevated border border-border rounded-lg p-4 hover:border-border-hover hover:shadow-[var(--shadow-medium)] hover:-translate-y-0.5 transition-all group overflow-hidden relative
      ${loading ? 'animate-shimmer' : ''}`}>
            <div className="flex items-center justify-between mb-3">
                <span className="text-[11px] font-bold uppercase tracking-wider text-fg-faint flex items-center gap-2">
                    {icon && React.cloneElement(icon, { size: 12, strokeWidth: 2, className: 'text-fg-subtle group-hover:text-accent' })}
                    {title}
                </span>
                {sparkline && sparkline.length > 1 && !loading && (
                    <div className="w-16 h-6 opacity-40 group-hover:opacity-100 transition-opacity">
                        <ResponsiveContainer width="100%" height="100%">
                            <AreaChart data={sparkline}>
                                <Area
                                    type="monotone" dataKey="val"
                                    stroke={lineColor} fill={lineColor}
                                    fillOpacity={0.1} strokeWidth={1.5}
                                    isAnimationActive={false}
                                />
                            </AreaChart>
                        </ResponsiveContainer>
                    </div>
                )}
            </div>

            {loading ? (
                <div className="h-6 w-24 bg-white/[0.03] rounded animate-pulse mb-1 mt-1" />
            ) : (
                <div className="text-[22px] font-semibold font-mono tracking-tighter text-fg leading-none">
                    {value || '0'}
                </div>
            )}

            {loading ? (
                <div className="h-3 w-16 bg-white/[0.02] rounded animate-pulse mt-1" />
            ) : (
                sub && <div className="text-[11px] text-fg-subtle mt-1.5 font-mono flex items-center gap-1.5">
                    <span className="w-1 h-1 rounded-full bg-fg-faint" />
                    {sub}
                </div>
            )}
        </div>
    );
};

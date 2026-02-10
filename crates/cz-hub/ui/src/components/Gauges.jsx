import React from 'react';

export const RingGaugeLarge = ({ value, head, tail, capacity }) => {
    const radius = 70;
    const circ = 2 * Math.PI * radius;
    const offset = circ * (1 - value / 100);
    const color = value > 90 ? 'var(--color-red)' : value > 70 ? 'var(--color-amber)' : 'var(--color-accent)';

    return (
        <div className="bg-bg-elevated border border-border rounded-lg p-5 flex flex-col items-center justify-center text-center h-full">
            <div className="relative w-36 h-36 mb-4">
                <svg className="w-full h-full -rotate-90" viewBox="0 0 180 180">
                    <circle cx="90" cy="90" r={radius} fill="none" stroke="rgba(255,255,255,0.04)" strokeWidth="8" />
                    <circle
                        cx="90" cy="90" r={radius} fill="none"
                        stroke={color} strokeWidth="8"
                        strokeDasharray={circ} strokeDashoffset={offset}
                        strokeLinecap="round"
                        style={{ transition: 'stroke-dashoffset 0.6s ease, stroke 0.4s ease' }}
                    />
                </svg>
                <div className="absolute inset-0 flex flex-col items-center justify-center">
                    <span className="text-2xl font-semibold font-mono" style={{ color }}>{value.toFixed(1)}%</span>
                    <span className="text-[10px] text-fg-subtle mt-0.5">utilization</span>
                </div>
            </div>

            <h3 className="text-[13px] font-semibold text-fg mb-1">Ring Buffer</h3>
            <p className="text-[11px] text-fg-subtle font-mono mb-4">
                {head.toLocaleString()} / {capacity?.toLocaleString()} slots
            </p>

            <div className="w-full space-y-1.5 text-[11px]">
                <div className="flex justify-between px-1">
                    <span className="text-fg-subtle">Head</span>
                    <span className="font-mono text-fg-muted">{head.toLocaleString()}</span>
                </div>
                <div className="flex justify-between px-1">
                    <span className="text-fg-subtle">Tail</span>
                    <span className="font-mono text-fg-muted">{tail.toLocaleString()}</span>
                </div>
                <div className="flex justify-between px-1">
                    <span className="text-fg-subtle">State</span>
                    <span className={`font-semibold ${value >= 90 ? 'text-red' : value >= 70 ? 'text-amber' : 'text-green'}`}>
                        {value >= 99 ? 'SATURATED' : value >= 90 ? 'CRITICAL' : 'NOMINAL'}
                    </span>
                </div>
            </div>
        </div>
    );
};

import React from 'react';
import { Activity, Cpu } from 'lucide-react';

export const StatusBar = ({ status, metrics, system }) => (
    <div className="h-7 px-4 flex items-center justify-between text-[11px] text-fg-subtle border-t border-border bg-bg-elevated">
        <div className="flex items-center gap-4">
            <div className="flex items-center gap-1.5">
                <div className={`w-1.5 h-1.5 rounded-full ${status === 'Connected' ? 'bg-green' : 'bg-red'}`} />
                <span>{status}</span>
            </div>
            <span className="text-fg-faint">|</span>
            <span className="font-mono text-fg-muted">{Math.round(metrics.tps).toLocaleString()} <span className="text-fg-faint">TPS</span></span>
            <span className="font-mono text-fg-muted">{Math.round(metrics.bps).toLocaleString()} <span className="text-fg-faint">BPS</span></span>
        </div>
        <div className="flex items-center gap-4">
            {system && (
                <>
                    <span className="font-mono text-fg-faint">PID {system.pid}</span>
                    <span className="font-mono text-fg-faint">{(system.memory_rss_kb / 1024).toFixed(1)} MB</span>
                </>
            )}
            <span className="text-fg-faint">LACRIMOSA v0.3.0</span>
        </div>
    </div>
);

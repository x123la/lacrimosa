import React from 'react';
import { Activity, Database, HardDrive, Clock, Zap } from 'lucide-react';
import { StatCard } from './StatCard';
import { MainThroughputChart } from './Charts';
import { RingGaugeLarge } from './Gauges';
import { PageHeader } from './Headers';

export const OverviewPage = ({ metrics, history, system, triggerSimulate }) => (
    <div className="animate-fade-in">
        <PageHeader
            title="Overview"
            subtitle={system ? `PID ${system.pid} · Uptime ${fmtUptime(system.uptime_seconds)} · Linux (io_uring)` : 'Connecting…'}
            actions={
                <button
                    onClick={triggerSimulate}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-accent hover:bg-accent-hover transition-colors text-[13px] font-medium text-white cursor-pointer"
                >
                    <Zap size={14} strokeWidth={1.6} /> Simulate
                </button>
            }
        />

        <div className="grid grid-cols-4 gap-3 mb-5">
            <StatCard
                title="Events" value={fmtNum(metrics.events)}
                icon={<Activity />} color="accent"
                sparkline={history.slice(-20).map(h => ({ val: h.tps }))}
                sub={metrics.tps > 0 ? `${fmtNum(Math.round(metrics.tps))}/s` : null}
            />
            <StatCard
                title="Throughput" value={fmtBytes(metrics.bytes)}
                icon={<Database />} color="green"
                sparkline={history.slice(-20).map(h => ({ val: h.bps }))}
                sub={metrics.bps > 0 ? `${fmtBytes(Math.round(metrics.bps))}/s` : null}
            />
            <StatCard
                title="Buffer" value={`${metrics.utilization_pct.toFixed(1)}%`}
                icon={<HardDrive />}
                color={metrics.utilization_pct > 80 ? 'red' : metrics.utilization_pct > 50 ? 'amber' : 'green'}
            />
            <StatCard
                title="Uptime" value={fmtUptime(metrics.uptime_seconds)}
                icon={<Clock />} color="accent"
            />
        </div>

        <div className="grid grid-cols-3 gap-3">
            <div className="col-span-2">
                <MainThroughputChart data={history} />
            </div>
            <RingGaugeLarge
                value={metrics.utilization_pct}
                head={metrics.head} tail={metrics.tail}
                capacity={Math.floor(1024 * 1024 * 1024 / 32)}
            />
        </div>
    </div>
);

const fmtNum = n => {
    if (n >= 1e9) return `${(n / 1e9).toFixed(2)}B`;
    if (n >= 1e6) return `${(n / 1e6).toFixed(2)}M`;
    if (n >= 1e3) return `${(n / 1e3).toFixed(1)}K`;
    return n.toLocaleString();
};

const fmtBytes = b => {
    if (b >= 1e12) return `${(b / 1e12).toFixed(2)} TB`;
    if (b >= 1e9) return `${(b / 1e9).toFixed(2)} GB`;
    if (b >= 1e6) return `${(b / 1e6).toFixed(1)} MB`;
    if (b >= 1e3) return `${(b / 1e3).toFixed(1)} KB`;
    return `${b} B`;
};

const fmtUptime = s => {
    if (!s) return '0s';
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = s % 60;
    if (h > 0) return `${h}h ${m}m`;
    if (m > 0) return `${m}m ${sec}s`;
    return `${sec}s`;
};

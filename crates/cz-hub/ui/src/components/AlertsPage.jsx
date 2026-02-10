import React from 'react';
import { Bell, Settings, AlertCircle, AlertTriangle, CheckCircle2 } from 'lucide-react';
import { PageHeader } from './Headers';

export const AlertsPage = ({ alerts, rules, onUpdateRules }) => (
    <div className="animate-fade-in grid grid-cols-12 gap-5">
        <div className="col-span-8 flex flex-col gap-4">
            <PageHeader title="Alerts" subtitle="System events and threshold violations" />

            <div className="flex flex-col gap-1.5">
                {alerts.length === 0 ? (
                    <div className="bg-bg-elevated border border-border rounded-lg p-16 flex flex-col items-center justify-center text-fg-faint">
                        <CheckCircle2 size={28} className="mb-2" />
                        <p className="text-[13px] font-medium">All systems nominal</p>
                    </div>
                ) : (
                    alerts.map(alert => (
                        <div key={alert.id}
                            className={`bg-bg-elevated border border-border rounded-md px-4 py-3 flex items-center gap-3 hover:bg-bg-surface transition-colors
                ${alert.severity === 'critical' ? 'border-l-2 border-l-red' :
                                    alert.severity === 'warn' ? 'border-l-2 border-l-amber' : 'border-l-2 border-l-accent'}`}
                        >
                            <span className={alert.severity === 'critical' ? 'text-red' : alert.severity === 'warn' ? 'text-amber' : 'text-accent'}>
                                {alert.severity === 'critical' ? <AlertCircle size={14} /> : <AlertTriangle size={14} />}
                            </span>
                            <div className="flex-1 min-w-0">
                                <span className="text-[13px] font-medium text-fg truncate block">{alert.message}</span>
                                <span className="text-[11px] text-fg-subtle">{alert.rule_name}</span>
                            </div>
                            <span className="text-[11px] font-mono text-fg-faint shrink-0">
                                {new Date(alert.timestamp).toLocaleTimeString()}
                            </span>
                        </div>
                    ))
                )}
            </div>
        </div>

        <div className="col-span-4 flex flex-col gap-4">
            <div className="flex items-center gap-2 pt-0.5">
                <Settings size={14} strokeWidth={1.6} className="text-fg-subtle" />
                <h3 className="text-[12px] font-semibold text-fg-subtle uppercase tracking-wide">Rules</h3>
            </div>

            <div className="flex flex-col gap-2">
                {rules.map((rule, i) => (
                    <div key={i} className="bg-bg-elevated border border-border rounded-lg p-4">
                        <div className="flex justify-between items-start mb-2">
                            <span className="text-[11px] font-mono text-fg-subtle">{rule.condition}</span>
                            <span className={`text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded
                ${rule.severity === 'critical' ? 'bg-red-muted text-red' : 'bg-amber-muted text-amber'}`}>
                                {rule.severity}
                            </span>
                        </div>
                        <h5 className="text-[13px] font-semibold text-fg mb-0.5">{rule.name}</h5>
                        <p className="text-[11px] text-fg-subtle mb-3">
                            Threshold: {rule.threshold}{rule.condition.includes('pct') || rule.condition.includes('utilization') ? '%' : ''}
                        </p>
                        <div className="flex items-center justify-between pt-2.5 border-t border-border">
                            <span className={`text-[11px] font-medium ${rule.enabled ? 'text-green' : 'text-fg-faint'}`}>
                                {rule.enabled ? 'Active' : 'Disabled'}
                            </span>
                            <button className="text-[11px] text-fg-subtle hover:text-fg transition-colors cursor-pointer">Edit</button>
                        </div>
                    </div>
                ))}
                <button className="w-full py-2 rounded-md border border-dashed border-border hover:border-border-hover text-fg-subtle hover:text-fg-muted text-[11px] font-medium transition-colors cursor-pointer">
                    + Add rule
                </button>
            </div>
        </div>
    </div>
);

import React from 'react';
import { ShieldCheck, Play, History, CheckCircle2, Loader2, FileCode } from 'lucide-react';
import { PageHeader } from './Headers';

export const VerifyPage = ({ onRunTests, results, running }) => (
    <div className="animate-fade-in grid grid-cols-12 gap-5 h-[calc(100vh-100px)] overflow-hidden">
        <div className="col-span-8 flex flex-col gap-4 overflow-hidden">
            <PageHeader
                title="Verification"
                subtitle="Kani proofs and workspace unit tests"
                actions={
                    <button
                        disabled={running}
                        onClick={onRunTests}
                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-green/90 hover:bg-green transition-colors text-[13px] font-medium text-bg disabled:opacity-40 cursor-pointer"
                    >
                        {running ? <Loader2 size={14} className="animate-spin" /> : <Play size={14} />}
                        {running ? 'Running…' : 'Run Tests'}
                    </button>
                }
            />

            <div className="flex-1 bg-bg-elevated border border-border rounded-lg flex flex-col overflow-hidden">
                <div className="px-4 py-2.5 border-b border-border flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <FileCode size={14} strokeWidth={1.6} className="text-fg-subtle" />
                        <h4 className="text-[13px] font-semibold text-fg">Output</h4>
                    </div>
                    {results && (
                        <span className={`text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded
              ${results.success ? 'bg-green-muted text-green' : 'bg-red-muted text-red'}`}>
                            {results.success ? 'PASSED' : 'FAILED'}
                        </span>
                    )}
                </div>

                <div className="flex-1 p-4 overflow-y-auto font-mono text-[11px] leading-relaxed bg-bg text-fg-muted">
                    {!results && !running ? (
                        <div className="h-full flex flex-col items-center justify-center text-fg-faint">
                            <ShieldCheck size={28} className="mb-2" />
                            <p className="text-[12px] font-medium">No verification logs</p>
                        </div>
                    ) : running ? (
                        <div className="space-y-1 text-fg-subtle animate-pulse">
                            <p className="text-green">$ cargo kani --package cz-verify</p>
                            <p>Checking CausalEvent ordering monotonicity…</p>
                            <p>Verifying Index Ring wrapping logic…</p>
                            <p>Validating zero-copy alignment…</p>
                        </div>
                    ) : (
                        <pre className="whitespace-pre-wrap">{results.output}</pre>
                    )}
                </div>
            </div>
        </div>

        <div className="col-span-4 flex flex-col gap-4 overflow-hidden">
            <div className="flex items-center gap-2 pt-0.5">
                <History size={14} strokeWidth={1.6} className="text-fg-subtle" />
                <h3 className="text-[12px] font-semibold text-fg-subtle uppercase tracking-wide">History</h3>
            </div>

            <div className="flex-1 overflow-y-auto space-y-2">
                {[1, 2, 3].map(i => (
                    <div key={i} className="bg-bg-elevated border border-border rounded-lg p-3 flex items-center gap-3 opacity-50 hover:opacity-100 transition-opacity">
                        <CheckCircle2 size={14} className="text-green shrink-0" />
                        <div className="flex-1 min-w-0">
                            <div className="text-[11px] font-medium text-fg">Test Group #A{i}2</div>
                            <div className="text-[11px] font-mono text-fg-subtle">PASSED · 42ms</div>
                        </div>
                    </div>
                ))}
                <p className="text-center py-3 text-[11px] text-fg-faint">End of session</p>
            </div>
        </div>
    </div>
);

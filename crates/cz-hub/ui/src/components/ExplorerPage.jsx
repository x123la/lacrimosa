import React from 'react';
import { Search, Box, Info, Loader2 } from 'lucide-react';
import { PageHeader } from './Headers';

const PayloadInspector = ({ detail }) => {
    if (!detail) {
        return (
            <div className="h-full flex items-center justify-center text-sm text-fg-muted">
                Select an event to inspect payload.
            </div>
        );
    }

    return (
        <div className="flex flex-col h-full gap-4">
            <div className="flex items-center gap-2">
                <div className="p-1.5 bg-accent/10 rounded">
                    <Box size={14} className="text-accent" />
                </div>
                <h4 className="text-[12px] font-bold text-fg uppercase tracking-wider">Payload Inspector</h4>
            </div>

            <div className="grid grid-cols-2 gap-3 text-[11px] font-mono">
                <div className="bg-bg border border-border rounded p-2">
                    <div className="text-fg-faint uppercase mb-1">Payload Size</div>
                    <div className="text-fg">{detail.payload_size} bytes</div>
                </div>
                <div className="bg-bg border border-border rounded p-2">
                    <div className="text-fg-faint uppercase mb-1">Payload Offset</div>
                    <div className="text-fg">{detail.payload_offset}</div>
                </div>
            </div>

            <div className="flex-1 min-h-0 grid grid-cols-1 gap-3">
                <div className="bg-bg border border-border rounded p-3 overflow-auto">
                    <div className="text-[10px] text-fg-faint uppercase mb-2">Hex</div>
                    <pre className="text-[11px] font-mono text-fg-muted whitespace-pre-wrap">{detail.payload_hex || '(empty)'}</pre>
                </div>
                <div className="bg-bg border border-border rounded p-3 overflow-auto">
                    <div className="text-[10px] text-fg-faint uppercase mb-2">ASCII</div>
                    <pre className="text-[11px] font-mono text-fg-muted whitespace-pre-wrap">{detail.payload_ascii || '(empty)'}</pre>
                </div>
            </div>
        </div>
    );
};

export const ExplorerPage = ({
    data = [],
    loading,
    offset,
    limit,
    setOffset,
    onSelectEvent,
    selectedSlot,
    detail,
    detailLoading,
}) => (
    <div className="flex-1 flex flex-col h-full bg-bg overflow-hidden animate-fade-in">
        <PageHeader
            title="Event Explorer"
            subtitle="Inspect sequenced events and payload bytes"
            actions={
                <div className="flex items-center gap-2">
                    <div className="relative group">
                        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 text-fg-faint" size={14} />
                        <input
                            type="text"
                            placeholder="Filtering coming soon"
                            disabled
                            className="bg-bg-elevated border border-border rounded-md pl-9 pr-3 py-1.5 text-[13px] w-64 opacity-70"
                        />
                    </div>
                </div>
            }
        />

        <div className="flex-1 flex min-h-0 border-t border-border">
            <div className="flex-1 flex flex-col min-w-0 bg-bg">
                <div className="grid grid-cols-[80px_1fr_90px_90px_140px] px-6 py-2 border-b border-border bg-bg-elevated/50">
                    {['SLOT', 'LAMPORT', 'NODE', 'STREAM', 'CHECKSUM'].map((h) => (
                        <span key={h} className="text-[10px] font-bold text-fg-faint uppercase tracking-[0.1em]">{h}</span>
                    ))}
                </div>

                <div className="flex-1 overflow-y-auto">
                    {loading ? (
                        <div className="flex flex-col gap-px p-4">
                            {Array.from({ length: 12 }).map((_, i) => (
                                <div key={i} className="h-10 bg-white/[0.02] rounded-md animate-pulse mb-1" />
                            ))}
                        </div>
                    ) : (
                        <div className="flex flex-col">
                            {data.map((item) => (
                                <button
                                    key={item.slot}
                                    onClick={() => onSelectEvent(item.slot)}
                                    className={`grid grid-cols-[80px_1fr_90px_90px_140px] px-6 py-2.5 items-center transition-all border-b border-border/40 group text-left
                    ${selectedSlot === item.slot
                                            ? 'bg-accent/5 text-accent shadow-[inset_4px_0_0_0_var(--color-accent)]'
                                            : 'text-fg-muted hover:bg-white/[0.03] hover:text-fg'}`}
                                >
                                    <span className="font-mono text-[12px] font-bold">#{item.slot}</span>
                                    <span className="font-mono text-[12px]">{item.lamport_ts}</span>
                                    <span className="font-mono text-[12px]">{item.node_id}</span>
                                    <span className="font-mono text-[12px]">{item.stream_id}</span>
                                    <span className="font-mono text-[11px] opacity-80">0x{Number(item.checksum).toString(16)}</span>
                                </button>
                            ))}
                            {!loading && data.length === 0 && (
                                <div className="p-8 text-center text-fg-faint">No events found for this range.</div>
                            )}
                        </div>
                    )}
                </div>

                <div className="p-4 border-t border-border bg-bg-elevated/30 flex justify-between items-center px-6">
                    <span className="text-[11px] text-fg-faint font-medium uppercase tracking-wider">Offset {offset}</span>
                    <div className="flex gap-2">
                        <button
                            onClick={() => setOffset(Math.max(0, offset - limit))}
                            disabled={offset === 0}
                            className="btn-secondary h-7 px-3 text-[11px] disabled:opacity-40"
                        >
                            Previous
                        </button>
                        <button
                            onClick={() => setOffset(offset + limit)}
                            className="btn-secondary h-7 px-3 text-[11px]"
                        >
                            Next
                        </button>
                    </div>
                </div>
            </div>

            <div className={`w-[480px] border-l border-border bg-bg-elevated overflow-hidden flex flex-col transition-all
        ${selectedSlot !== null ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0 absolute'}`}>
                {selectedSlot !== null ? (
                    <div className="flex-1 flex flex-col p-6 overflow-hidden">
                        <div className="mb-6">
                            <h3 className="text-[18px] font-bold text-fg tracking-tight mb-2">Event Detail</h3>
                            <p className="text-[12px] text-fg-muted leading-relaxed">
                                Slot <span className="text-accent font-mono">#{selectedSlot}</span>
                                {detail && (
                                    <span>
                                        {' '}· Node {detail.node_id} · Stream {detail.stream_id} · Lamport {detail.lamport_ts}
                                    </span>
                                )}
                            </p>
                        </div>

                        {detailLoading ? (
                            <div className="flex-1 flex items-center justify-center text-fg-muted">
                                <Loader2 className="animate-spin" size={18} />
                            </div>
                        ) : (
                            <PayloadInspector detail={detail} />
                        )}
                    </div>
                ) : (
                    <div className="flex-1 flex flex-col items-center justify-center p-12 text-center">
                        <div className="w-12 h-12 rounded-full bg-bg-surface flex items-center justify-center mb-4 border border-border">
                            <Info size={20} className="text-fg-faint" />
                        </div>
                        <h4 className="text-[14px] font-bold text-fg mb-1">No Event Selected</h4>
                        <p className="text-[12px] text-fg-faint">Select an event from the stream to inspect payload and metadata.</p>
                    </div>
                )}
            </div>
        </div>
    </div>
);

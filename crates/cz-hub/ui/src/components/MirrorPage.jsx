import React from 'react';
import { Eye, HardDrive, Hash, Layers, Info } from 'lucide-react';
import { PageHeader } from './Headers';

export const MirrorPage = ({ layout, loading }) => {
    if (loading || !layout) return (
        <div className="p-20 text-center text-fg-subtle text-[13px] animate-pulse">Scanning mmap layout…</div>
    );

    const pct = (layout.slots_used / layout.index_ring_slot_count) * 100;

    return (
        <div className="animate-fade-in flex flex-col gap-5">
            <PageHeader title="Journal Mirror" subtitle="Memory-mapped backing store diagnostic" />

            <div className="grid grid-cols-4 gap-3">
                <MiniCard icon={<HardDrive />} label="Total Size" value={fmtB(layout.total_size_bytes)} />
                <MiniCard icon={<Hash />} label="Index Slots" value={layout.index_ring_slot_count.toLocaleString()} />
                <MiniCard icon={<Layers />} label="Slot Size" value={`${layout.index_ring_slot_size} B`} />
                <MiniCard icon={<Info />} label="Blob Region" value={fmtB(layout.blob_storage_size_bytes)} />
            </div>

            <div className="bg-bg-elevated border border-border rounded-lg p-5 flex flex-col gap-6">
                {/* Storage map */}
                <div>
                    <div className="flex justify-between items-end mb-3">
                        <div>
                            <h4 className="text-[13px] font-semibold text-fg mb-0.5">Storage Map</h4>
                            <p className="text-[11px] text-fg-subtle">Linear address space of {layout.journal_path || 'journal.db'}</p>
                        </div>
                        <div className="flex gap-4 text-[10px] font-medium text-fg-subtle">
                            <div className="flex items-center gap-1.5">
                                <div className="w-2 h-2 rounded-sm bg-accent" /> Index Ring
                            </div>
                            <div className="flex items-center gap-1.5">
                                <div className="w-2 h-2 rounded-sm bg-green" /> Blob Storage
                            </div>
                        </div>
                    </div>

                    <div className="relative h-8 w-full bg-bg rounded-md border border-border overflow-hidden flex">
                        <div
                            className="h-full bg-accent/60 transition-all duration-700"
                            style={{ width: `${(layout.index_ring_size_bytes / layout.total_size_bytes) * 100}%` }}
                        />
                        <div className="h-full bg-green/30 flex-1" />
                    </div>

                    <div className="flex justify-between mt-2 text-[10px] font-mono text-fg-faint">
                        <span>0x00000000</span>
                        <span>0x{layout.total_size_bytes.toString(16).padStart(8, '0')}</span>
                    </div>
                </div>

                {/* Details */}
                <div className="grid grid-cols-2 gap-8">
                    <div className="space-y-3">
                        <h5 className="text-[11px] font-semibold text-fg-subtle uppercase">Index Saturation</h5>
                        <div>
                            <div className="flex justify-between text-[11px] font-mono mb-1.5">
                                <span className="text-fg-subtle">{layout.slots_used} used</span>
                                <span className="text-accent">{layout.slots_free} free</span>
                            </div>
                            <div className="h-1.5 w-full bg-bg rounded-full overflow-hidden">
                                <div
                                    className={`h-full bg-accent transition-all duration-500 ${pct > 90 ? 'animate-pulse' : ''}`}
                                    style={{ width: `${pct}%` }}
                                />
                            </div>
                        </div>
                        <p className="text-[11px] text-fg-subtle leading-relaxed">
                            When full, LRU eviction advances the tail cursor.
                        </p>
                    </div>

                    <div className="space-y-3">
                        <h5 className="text-[11px] font-semibold text-fg-subtle uppercase">Pointers</h5>
                        <div className="bg-bg rounded-md border border-border p-3 space-y-2">
                            <Row label="Index Start" value="0x00000000" />
                            <Row label="Blob Start" value={`0x${layout.index_ring_size_bytes.toString(16)}`} />
                            <Row label="Logical Head" value={`#${layout.slots_used % layout.index_ring_slot_count}`} />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

const MiniCard = ({ icon, label, value }) => (
    <div className="bg-bg-elevated border border-border rounded-lg px-4 py-3 flex items-center gap-3">
        <span className="text-fg-subtle">{React.cloneElement(icon, { size: 14, strokeWidth: 1.6 })}</span>
        <div>
            <span className="text-[10px] font-medium text-fg-subtle block">{label}</span>
            <span className="text-[13px] font-semibold font-mono text-fg">{value}</span>
        </div>
    </div>
);

const Row = ({ label, value }) => (
    <div className="flex justify-between items-center text-[11px]">
        <span className="text-fg-subtle font-medium">{label}</span>
        <span className="font-mono text-fg-muted">{value}</span>
    </div>
);

const fmtB = b => {
    if (b >= 1024 * 1024) return `${(b / (1024 * 1024)).toFixed(1)} MB`;
    if (b >= 1024) return `${(b / 1024).toFixed(1)} KB`;
    return `${b} B`;
};

import React, { useState, useEffect, useCallback } from 'react';
import { Search } from 'lucide-react';

export const CommandPalette = ({ isOpen, onClose, actions }) => {
    const [query, setQuery] = useState('');
    const [selectedIndex, setSelectedIndex] = useState(0);

    const filtered = actions.filter(a =>
        a.label.toLowerCase().includes(query.toLowerCase()) ||
        a.description?.toLowerCase().includes(query.toLowerCase())
    );

    const handleKeyDown = useCallback((e) => {
        if (e.key === 'Escape') onClose();
        if (e.key === 'ArrowDown') { e.preventDefault(); setSelectedIndex(s => (s + 1) % filtered.length); }
        if (e.key === 'ArrowUp') { e.preventDefault(); setSelectedIndex(s => (s - 1 + filtered.length) % filtered.length); }
        if (e.key === 'Enter' && filtered[selectedIndex]) {
            filtered[selectedIndex].onSelect();
            onClose();
        }
    }, [filtered, selectedIndex, onClose]);

    useEffect(() => {
        if (isOpen) {
            setQuery('');
            setSelectedIndex(0);
            window.addEventListener('keydown', handleKeyDown);
            return () => window.removeEventListener('keydown', handleKeyDown);
        }
    }, [isOpen, handleKeyDown]);

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-[200] flex items-start justify-center pt-[18vh] px-4 bg-black/50" onClick={onClose}>
            <div
                className="w-full max-w-[560px] bg-bg-overlay border border-border rounded-xl overflow-hidden shadow-[0_24px_48px_-12px_rgba(0,0,0,0.5)] animate-fade-in"
                onClick={e => e.stopPropagation()}
            >
                {/* Search input */}
                <div className="flex items-center gap-3 px-4 py-3 border-b border-border">
                    <Search size={15} strokeWidth={1.6} className="text-fg-subtle" />
                    <input
                        autoFocus
                        className="flex-1 bg-transparent text-[14px] text-fg placeholder:text-fg-faint outline-none"
                        placeholder="Type a command or search…"
                        value={query}
                        onChange={e => { setQuery(e.target.value); setSelectedIndex(0); }}
                    />
                    <kbd className="text-[10px] text-fg-faint font-medium px-1.5 py-0.5 rounded border border-border bg-bg">
                        ESC
                    </kbd>
                </div>

                {/* Results */}
                <div className="max-h-[320px] overflow-y-auto py-1">
                    {filtered.map((action, i) => (
                        <button
                            key={action.id}
                            className={`flex items-center gap-3 w-full px-4 py-2 text-left transition-colors cursor-pointer
                ${i === selectedIndex ? 'bg-accent-muted' : 'hover:bg-white/[0.02]'}`}
                            onMouseEnter={() => setSelectedIndex(i)}
                            onClick={() => { action.onSelect(); onClose(); }}
                        >
                            <span className={`${i === selectedIndex ? 'text-accent' : 'text-fg-subtle'}`}>
                                {React.cloneElement(action.icon, { size: 15, strokeWidth: 1.6 })}
                            </span>
                            <span className={`text-[13px] font-medium flex-1 ${i === selectedIndex ? 'text-fg' : 'text-fg-muted'}`}>
                                {action.label}
                            </span>
                            {action.shortcut && (
                                <kbd className="text-[10px] text-fg-faint font-mono px-1.5 py-0.5 rounded border border-border bg-bg">
                                    {action.shortcut}
                                </kbd>
                            )}
                        </button>
                    ))}
                    {filtered.length === 0 && (
                        <div className="px-4 py-8 text-center text-fg-faint text-[13px]">
                            No results for "{query}"
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

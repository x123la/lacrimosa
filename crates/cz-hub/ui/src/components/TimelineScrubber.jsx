import React, { useState } from 'react';
import { Play, Pause, RotateCcw, FastForward, Clock } from 'lucide-react';

export const TimelineScrubber = ({ history, onSeek }) => {
    const [isPlaying, setIsPlaying] = useState(false);
    const [currentIndex, setCurrentIndex] = useState(history.length - 1);

    const togglePlay = () => setIsPlaying(!isPlaying);

    return (
        <div className="glass p-4 border-white/5 bg-white/[0.02] flex items-center gap-6">
            <div className="flex items-center gap-2">
                <button
                    onClick={() => setCurrentIndex(0)}
                    className="p-2 hover:bg-white/5 rounded-lg text-muted transition-colors"
                >
                    <RotateCcw size={16} />
                </button>
                <button
                    onClick={togglePlay}
                    className="w-10 h-10 flex items-center justify-center bg-primary rounded-xl text-white shadow-lg shadow-primary/20 glow-primary transition-smooth"
                >
                    {isPlaying ? <Pause size={18} fill="currentColor" /> : <Play size={18} fill="currentColor" />}
                </button>
            </div>

            <div className="flex-1 flex flex-col gap-2">
                <div className="flex justify-between text-[9px] font-black uppercase tracking-widest text-muted">
                    <div className="flex items-center gap-2">
                        <Clock size={10} className="text-primary" />
                        <span>Timeline Replay</span>
                    </div>
                    <span className="text-primary-light">{history[currentIndex]?.time || 'LIVE'}</span>
                </div>
                <input
                    type="range"
                    min="0"
                    max={Math.max(0, history.length - 1)}
                    value={currentIndex}
                    onChange={(e) => {
                        const idx = parseInt(e.target.value);
                        setCurrentIndex(idx);
                        onSeek?.(history[idx]);
                    }}
                    className="w-full h-1.5 bg-white/5 rounded-full appearance-none cursor-pointer accent-primary"
                />
            </div>

            <div className="flex items-center gap-3">
                <div className="text-right">
                    <div className="text-xs font-black tracking-tight">{history[currentIndex]?.tps || 0}</div>
                    <div className="text-[8px] text-muted font-bold uppercase">TPS</div>
                </div>
                <button className="p-2 hover:bg-white/5 rounded-lg text-muted transition-colors">
                    <FastForward size={16} />
                </button>
            </div>
        </div>
    );
};

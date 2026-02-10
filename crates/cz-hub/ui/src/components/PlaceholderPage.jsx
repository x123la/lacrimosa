
import React from 'react';
import { PageHeader } from './Headers';
import { Construction } from 'lucide-react';

export const PlaceholderPage = ({ title, subtitle }) => (
    <div className="animate-fade-in max-w-4xl mx-auto mt-20 text-center">
        <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-white/5 mb-6">
            <Construction size={40} className="text-white/20" />
        </div>
        <h2 className="text-2xl font-bold text-white mb-2">{title}</h2>
        <p className="text-white/50 max-w-md mx-auto mb-8">{subtitle}</p>
        <div className="inline-block px-4 py-2 rounded bg-white/5 border border-white/10 text-xs font-mono text-white/40">
            Implementation Pending (Batch 2/3)
        </div>
    </div>
);

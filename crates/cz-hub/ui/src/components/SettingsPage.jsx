import React from 'react';
import { PageHeader } from './Headers';
import { Monitor, Moon, Sun, Bell, Volume2, Shield, Keyboard } from 'lucide-react';

export const SettingsPage = () => {
    return (
        <div className="animate-fade-in max-w-4xl mx-auto">
            <PageHeader
                title="Settings"
                subtitle="Manage workspace preferences and interface behavior"
            />

            <div className="space-y-8 mt-6">
                {/* Visual Section */}
                <section>
                    <h3 className="text-[13px] font-bold text-fg uppercase tracking-wider mb-4 pb-2 border-b border-border">Interface</h3>
                    <div className="grid gap-4">
                        <div className="flex items-center justify-between p-4 bg-bg-elevated border border-border rounded-lg">
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-bg-surface rounded-md">
                                    <Monitor size={16} className="text-fg" />
                                </div>
                                <div>
                                    <h4 className="text-[14px] font-medium text-fg">Theme Preference</h4>
                                    <p className="text-[12px] text-fg-muted">Select your preferred interface appearance</p>
                                </div>
                            </div>
                            <div className="flex bg-bg-surface p-1 rounded-md border border-border">
                                <button className="p-1.5 rounded bg-bg-elevated shadow-sm text-fg">
                                    <Moon size={14} />
                                </button>
                                <button className="p-1.5 rounded text-fg-subtle hover:text-fg transition-colors">
                                    <Sun size={14} />
                                </button>
                                <button className="p-1.5 rounded text-fg-subtle hover:text-fg transition-colors">
                                    <Monitor size={14} />
                                </button>
                            </div>
                        </div>
                    </div>
                </section>

                {/* Notifications Section */}
                <section>
                    <h3 className="text-[13px] font-bold text-fg uppercase tracking-wider mb-4 pb-2 border-b border-border">Notifications</h3>
                    <div className="grid gap-3">
                        <div className="flex items-center justify-between p-4 bg-bg-elevated border border-border rounded-lg group">
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-bg-surface rounded-md">
                                    <Bell size={16} className="text-fg" />
                                </div>
                                <div>
                                    <h4 className="text-[14px] font-medium text-fg">Push Notifications</h4>
                                    <p className="text-[12px] text-fg-muted">Receive browser alerts for critical incidents</p>
                                </div>
                            </div>
                            <div className="w-10 h-5 bg-accent rounded-full relative cursor-pointer shadow-inner">
                                <div className="absolute right-1 top-1 w-3 h-3 bg-white rounded-full shadow-sm" />
                            </div>
                        </div>

                        <div className="flex items-center justify-between p-4 bg-bg-elevated border border-border rounded-lg group">
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-bg-surface rounded-md">
                                    <Volume2 size={16} className="text-fg" />
                                </div>
                                <div>
                                    <h4 className="text-[14px] font-medium text-fg">Sound Effects</h4>
                                    <p className="text-[12px] text-fg-muted">Play audible alerts for system events</p>
                                </div>
                            </div>
                            <div className="w-10 h-5 bg-accent/20 rounded-full relative cursor-pointer shadow-inner">
                                <div className="absolute left-1 top-1 w-3 h-3 bg-fg-muted rounded-full shadow-sm" />
                            </div>
                        </div>
                    </div>
                </section>

                {/* System Section */}
                <section>
                    <h3 className="text-[13px] font-bold text-fg uppercase tracking-wider mb-4 pb-2 border-b border-border">System</h3>
                    <div className="grid gap-3">
                        <div className="flex items-center justify-between p-4 bg-bg-elevated border border-border rounded-lg">
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-bg-surface rounded-md">
                                    <Shield size={16} className="text-fg" />
                                </div>
                                <div>
                                    <h4 className="text-[14px] font-medium text-fg">Verification Mode</h4>
                                    <p className="text-[12px] text-fg-muted">Enforce formal proofs before journal writes</p>
                                </div>
                            </div>
                            <span className="text-[11px] font-bold text-green bg-green/10 px-2 py-1 rounded border border-green/20">ACTIVE</span>
                        </div>
                        <div className="flex items-center justify-between p-4 bg-bg-elevated border border-border rounded-lg">
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-bg-surface rounded-md">
                                    <Keyboard size={16} className="text-fg" />
                                </div>
                                <div>
                                    <h4 className="text-[14px] font-medium text-fg">Command Palette</h4>
                                    <p className="text-[12px] text-fg-muted">Global shortcut <code className="bg-bg-surface px-1.5 py-0.5 rounded border border-border text-[10px]">Cmd+K</code></p>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>

                <div className="pt-8 text-center">
                    <p className="text-[11px] text-fg-faint">
                        LACRIMOSA Control Center v0.3.0 · <span className="font-mono">ref: 32a1b4</span>
                    </p>
                </div>
            </div>
        </div>
    );
};

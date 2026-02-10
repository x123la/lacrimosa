
import React from 'react';
import {
    LayoutGrid, Radio, BarChart2, Settings,
    BarChart3, Bell, Eye, HardDrive, Search,
    Database, Terminal, GitGraph, Workflow, Key, ShieldAlert, LayoutTemplate
} from 'lucide-react';

const SidebarItem = ({ id, label, icon: Icon, active, badge, onClick }) => (
    <button
        onClick={() => onClick(id)}
        className={`w-full group flex items-center justify-between px-3 py-1.5 rounded-md transition-all cursor-pointer outline-none focus-visible:ring-1 focus-visible:ring-accent focus-visible:ring-inset
      ${active
                ? 'bg-accent/10 text-accent font-medium shadow-[inset_2px_0_0_0_currentColor]'
                : 'text-fg-muted hover:bg-white/[0.04] hover:text-fg'}`}
    >
        <div className="flex items-center gap-2.5">
            <Icon size={14} strokeWidth={active ? 2 : 1.6} className={active ? 'text-accent' : 'text-fg-faint group-hover:text-fg-muted'} />
            <span className="text-[13px] tracking-tight">{label}</span>
        </div>
        {badge > 0 && (
            <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-mono font-medium
        ${active ? 'bg-accent/20 text-accent' : 'bg-bg-surface text-fg-faint group-hover:text-fg-muted border border-border'}`}>
                {badge}
            </span>
        )}
    </button>
);

const SidebarSection = ({ title, children }) => (
    <div className="mb-6">
        <h3 className="px-3 mb-2 text-[10px] font-bold text-fg-faint uppercase tracking-[0.08em]">{title}</h3>
        <div className="space-y-0.5">
            {children}
        </div>
    </div>
);

export const Sidebar = ({ activePage, setPage, alerts }) => {
    const alertCount = alerts?.filter(a => a.severity === 'critical').length || 0;

    return (
        <aside className="w-60 bg-bg-elevated border-r border-border flex flex-col pt-6 pb-4 shrink-0">
            <div className="px-6 mb-8 flex items-center gap-3 group cursor-default">
                <div className="w-6 h-6 rounded bg-accent flex items-center justify-center text-white font-bold text-xs shadow-[0_0_10px_rgba(94,106,210,0.3)] group-hover:shadow-[0_0_15px_rgba(94,106,210,0.5)] transition-all">
                    Ω
                </div>
                <div className="flex flex-col">
                    <h1 className="text-[13px] font-bold tracking-tight text-fg">LACRIMOSA</h1>
                    <span className="text-[10px] font-medium text-fg-faint uppercase">Control Center</span>
                </div>
            </div>

            <nav className="flex-1 px-3 overflow-y-auto">
                <SidebarSection title="Core">
                    <SidebarItem id="overview" label="Overview" icon={LayoutGrid} active={activePage === 'overview'} onClick={setPage} />
                    <SidebarItem id="alerts" label="Incidents" icon={ShieldAlert} active={activePage === 'alerts'} badge={alertCount} onClick={setPage} />
                    <SidebarItem id="analytics" label="Analytics" icon={BarChart2} active={activePage === 'analytics'} onClick={setPage} />
                </SidebarSection>

                <SidebarSection title="Data & Streams">
                    <SidebarItem id="connectors" label="Connectors" icon={Database} active={activePage === 'connectors'} onClick={setPage} />
                    <SidebarItem id="query" label="Query Console" icon={Terminal} active={activePage === 'query'} onClick={setPage} />
                    <SidebarItem id="explorer" label="Event Explorer" icon={Search} active={activePage === 'explorer'} onClick={setPage} />
                </SidebarSection>

                <SidebarSection title="Intelligence">
                    <SidebarItem id="traces" label="Traces" icon={GitGraph} active={activePage === 'traces'} onClick={setPage} />
                    <SidebarItem id="pipelines" label="Pipelines" icon={Workflow} active={activePage === 'pipelines'} onClick={setPage} />
                </SidebarSection>

                <SidebarSection title="Platform">
                    <SidebarItem id="dashboards" label="Dashboards" icon={LayoutTemplate} active={activePage === 'dashboards'} onClick={setPage} />
                </SidebarSection>
            </nav>

            <div className="px-3 pt-4 border-t border-border mt-auto">
                <SidebarItem id="settings" label="Settings" icon={Settings} active={activePage === 'settings'} onClick={setPage} />
            </div>
        </aside>
    );
};

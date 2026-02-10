
import React, { useState, useEffect } from 'react';
import { Responsive, WidthProvider } from 'react-grid-layout/legacy';
import 'react-grid-layout/css/styles.css';
import 'react-resizable/css/styles.css';
// ...
import {
    Plus,
    Save,
    Edit,
    Layout,
    Trash2,
    BarChart2,
    Activity,
    Table as TableIcon,
    FileText
} from 'lucide-react';
import _ from 'lodash';

// Wrap ResponsiveGridLayout with WidthProvider to make it responsive
const ResponsiveGridLayout = WidthProvider(Responsive);

// Widget Component
const WidgetItem = ({ widget, onDelete, isEditing }) => {
    return (
        <div className="h-full w-full bg-bg-elevated border border-border rounded shadow-sm flex flex-col overflow-hidden">
            <div className="bg-bg-surface px-3 py-1.5 border-b border-border flex justify-between items-center cursor-move draggable-handle">
                <span className="text-xs font-bold text-fg uppercase flex items-center gap-2">
                    {widget.type === 'time_series' && <Activity size={12} className="text-blue-400" />}
                    {widget.type === 'value' && <BarChart2 size={12} className="text-green-400" />}
                    {widget.type === 'table' && <TableIcon size={12} className="text-purple-400" />}
                    {widget.type === 'log_stream' && <FileText size={12} className="text-yellow-400" />}
                    {widget.title}
                </span>
                {isEditing && (
                    <button
                        onClick={(e) => { e.stopPropagation(); onDelete(widget.id); }}
                        className="text-fg-muted hover:text-red-400"
                    >
                        <Trash2 size={12} />
                    </button>
                )}
            </div>
            <div className="flex-1 p-4 overflow-auto">
                <div className="text-xs text-fg-muted font-mono">
                    {/* Placeholder for actual visualization */}
                    {JSON.stringify(widget, null, 2)}
                </div>
            </div>
        </div>
    );
};

export const DashboardsPage = () => {
    const [dashboards, setDashboards] = useState([]);
    const [selectedDashboard, setSelectedDashboard] = useState(null);
    const [isEditing, setIsEditing] = useState(false);
    const [layout, setLayout] = useState([]);
    const [widgets, setWidgets] = useState([]);

    useEffect(() => {
        fetchDashboards();
    }, []);

    const fetchDashboards = async () => {
        try {
            const res = await fetch('/api/dashboards');
            const data = await res.json();
            setDashboards(data);
        } catch (e) {
            console.error(e);
        }
    };

    const createDashboard = async () => {
        const name = prompt("Dashboard Name:");
        if (!name) return;
        try {
            const res = await fetch('/api/dashboards', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name, description: "" })
            });
            const newDash = await res.json();
            setDashboards([...dashboards, newDash]);
            selectDashboard(newDash);
        } catch (e) {
            console.error(e);
        }
    };

    const selectDashboard = (dash) => {
        setSelectedDashboard(dash);
        setLayout(dash.layout || []);
        setWidgets(dash.widgets || []);
        setIsEditing(false);
    };

    const saveDashboard = async () => {
        if (!selectedDashboard) return;
        try {
            const res = await fetch(`/api/dashboards/${selectedDashboard.id}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ layout, widgets })
            });
            const updated = await res.json();
            setDashboards(dashboards.map(d => d.id === updated.id ? updated : d));
            setSelectedDashboard(updated);
            setIsEditing(false);
        } catch (e) {
            console.error(e);
        }
    };

    const deleteDashboard = async (id) => {
        if (!confirm("Delete dashboard?")) return;
        try {
            await fetch(`/api/dashboards/${id}`, { method: 'DELETE' });
            setDashboards(dashboards.filter(d => d.id !== id));
            if (selectedDashboard?.id === id) setSelectedDashboard(null);
        } catch (e) {
            console.error(e);
        }
    };

    const addWidget = (type) => {
        const id = `w-${Date.now()}`;
        const newWidget = {
            id,
            type,
            title: `New ${type}`,
            // Default configs
            ...(type === 'time_series' ? { query: 'SELECT * FROM metrics' } : {}),
            ...(type === 'value' ? { stream: 'cpu', field: 'usage', unit: '%' } : {}),
            ...(type === 'table' ? { query: 'SELECT * FROM logs LIMIT 10', columns: [] } : {}),
            ...(type === 'log_stream' ? { stream: 'app-logs' } : {}),
        };

        const newItem = {
            i: id,
            x: (layout.length * 4) % 12,
            y: Infinity, // puts it at the bottom
            w: 4,
            h: 4,
        };

        setWidgets([...widgets, newWidget]);
        setLayout([...layout, newItem]);
    };

    const removeWidget = (id) => {
        setWidgets(widgets.filter(w => w.id !== id));
        setLayout(layout.filter(l => l.i !== id));
    };

    const onLayoutChange = (newLayout) => {
        setLayout(newLayout);
    };

    return (
        <div className="flex h-full animate-fade-in">
            {/* Sidebar List */}
            <div className="w-64 border-r border-border bg-bg-surface flex flex-col">
                <div className="p-3 border-b border-border flex justify-between items-center">
                    <span className="text-xs font-bold text-fg-muted uppercase">Dashboards</span>
                    <button onClick={createDashboard} className="p-1 hover:bg-white/10 rounded">
                        <Plus size={14} />
                    </button>
                </div>
                <div className="flex-1 overflow-y-auto">
                    {dashboards.map(d => (
                        <div
                            key={d.id}
                            onClick={() => selectDashboard(d)}
                            className={`p-3 border-b border-border cursor-pointer hover:bg-white/5 ${selectedDashboard?.id === d.id ? 'bg-accent/10 border-l-2 border-l-accent' : 'border-l-2 border-l-transparent'}`}
                        >
                            <div className="font-medium text-sm truncate">{d.name}</div>
                            <div className="flex justify-between items-center mt-2">
                                <span className="text-[10px] text-fg-muted">{d.widgets?.length || 0} Widgets</span>
                                <button onClick={(e) => { e.stopPropagation(); deleteDashboard(d.id); }}>
                                    <Trash2 size={12} className="text-fg-muted hover:text-red-400" />
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Main Area */}
            <div className="flex-1 h-full bg-bg flex flex-col overflow-hidden">
                {selectedDashboard ? (
                    <>
                        <div className="h-12 border-b border-border bg-bg-surface flex items-center justify-between px-4">
                            <h2 className="font-bold flex items-center gap-2">
                                <Layout size={16} />
                                {selectedDashboard.name}
                            </h2>
                            <div className="flex gap-2">
                                {isEditing ? (
                                    <>
                                        <div className="flex bg-bg-elevated rounded border border-border mr-4">
                                            <button onClick={() => addWidget('time_series')} className="p-1.5 hover:bg-white/10" title="Add Chart"><Activity size={14} /></button>
                                            <button onClick={() => addWidget('value')} className="p-1.5 hover:bg-white/10" title="Add Value"><BarChart2 size={14} /></button>
                                            <button onClick={() => addWidget('table')} className="p-1.5 hover:bg-white/10" title="Add Table"><TableIcon size={14} /></button>
                                            <button onClick={() => addWidget('log_stream')} className="p-1.5 hover:bg-white/10" title="Add Logs"><FileText size={14} /></button>
                                        </div>
                                        <button onClick={saveDashboard} className="px-3 py-1.5 bg-accent text-white rounded text-xs font-medium flex items-center gap-1.5">
                                            <Save size={14} /> Save
                                        </button>
                                    </>
                                ) : (
                                    <button onClick={() => setIsEditing(true)} className="px-3 py-1.5 bg-bg-elevated border border-border rounded text-xs font-medium flex items-center gap-1.5 hover:bg-white/5">
                                        <Edit size={14} /> Edit Layout
                                    </button>
                                )}
                            </div>
                        </div>
                        <div className="flex-1 overflow-y-auto p-4">
                            <ResponsiveGridLayout
                                className="layout"
                                layouts={{ lg: layout }}
                                breakpoints={{ lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }}
                                cols={{ lg: 12, md: 10, sm: 6, xs: 4, xxs: 2 }}
                                rowHeight={30}
                                draggableHandle=".draggable-handle"
                                isDraggable={isEditing}
                                isResizable={isEditing}
                                onLayoutChange={(_, allLayouts) => onLayoutChange(allLayouts.lg || [])} // Simpler for now, just take lg
                            >
                                {widgets.map(w => (
                                    <div key={w.id} data-grid={layout.find(l => l.i === w.id)}>
                                        <WidgetItem
                                            widget={w}
                                            onDelete={removeWidget}
                                            isEditing={isEditing}
                                        />
                                    </div>
                                ))}
                            </ResponsiveGridLayout>
                        </div>
                    </>
                ) : (
                    <div className="flex items-center justify-center h-full text-fg-muted">
                        Select a dashboard or create new
                    </div>
                )}
            </div>
        </div>
    );
};

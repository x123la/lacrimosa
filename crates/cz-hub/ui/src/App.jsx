import React, { useState, useEffect, useCallback } from 'react';
import {
  LayoutDashboard, Database, Share2, ShieldCheck,
  BarChart3, Bell, Eye, Download,
  Settings, Zap
} from 'lucide-react';

// Specialized Sub-pages
import { OverviewPage } from './components/OverviewPage';
import { ExplorerPage } from './components/ExplorerPage';
import { TopologyPage } from './components/TopologyPage';
import { VerifyPage } from './components/VerifyPage';
import { MirrorPage } from './components/MirrorPage';
import { AnalyticsPage } from './components/AnalyticsPage';
import { ConnectorsPage } from './components/ConnectorsPage';
import { QueryConsolePage } from './components/QueryConsolePage';
import { TracesPage } from './components/TracesPage';
import { PipelinesPage } from './components/PipelinesPage';
import { IncidentsPage } from './components/IncidentsPage';
import { DashboardsPage } from './components/DashboardsPage';
import { SettingsPage } from './components/SettingsPage';

// Shared Components
import { Sidebar } from './components/Sidebar';
import { StatusBar } from './components/StatusBar';
import { ToastContainer } from './components/Toast';
import { CommandPalette } from './components/CommandPalette';

export default function App() {
  // Navigation & UI State
  const [activePage, setActivePage] = useState('overview');
  const [showCmdPalette, setShowCmdPalette] = useState(false);
  const [toasts, setToasts] = useState([]);
  const [status, setStatus] = useState('Connecting...');
  const [theme, setTheme] = useState(localStorage.getItem('cz-theme') || 'indigo');
  const [apiKey, setApiKey] = useState(localStorage.getItem('cz-api-key') || '');
  const [apiKeyDraft, setApiKeyDraft] = useState(localStorage.getItem('cz-api-key') || '');
  const [showAuthModal, setShowAuthModal] = useState(!localStorage.getItem('cz-api-key'));

  // Data State
  const [metrics, setMetrics] = useState({
    events: 0, bytes: 0, tps: 0, bps: 0, utilization_pct: 0, uptime_seconds: 0, head: 0, tail: 0
  });
  const [history, setHistory] = useState([]);
  const [system, setSystem] = useState(null);
  const [events, setEvents] = useState({ events: [], total: 0 });
  const [topology, setTopology] = useState(null);
  const [alerts, setAlerts] = useState([]);
  const [journalLayout, setJournalLayout] = useState(null);
  const [streamStats, setStreamStats] = useState(null);
  const [verifyResults, setVerifyResults] = useState(null);
  const [isVerifying, setIsVerifying] = useState(false);

  // Explorer State
  const [eventOffset, setEventOffset] = useState(0);
  const [selectedSlot, setSelectedSlot] = useState(null);
  const [eventDetail, setEventDetail] = useState(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [loadingStates, setLoadingStates] = useState({});

  const addToast = (msg, type = 'info') => {
    const id = Date.now();
    setToasts(prev => [...prev, { id, message: msg, type }]);
  };

  const removeToast = (id) => setToasts(prev => prev.filter(t => t.id !== id));

  useEffect(() => {
    const originalFetch = window.fetch.bind(window);
    window.fetch = async (input, init = {}) => {
      const headers = new Headers(init.headers || {});
      if (apiKey) {
        headers.set('Authorization', `Bearer ${apiKey}`);
      }

      const response = await originalFetch(input, { ...init, headers });
      if (response.status === 401) {
        setShowAuthModal(true);
        setStatus('Unauthorized');
      }
      return response;
    };

    return () => {
      window.fetch = originalFetch;
    };
  }, [apiKey]);

  const saveApiKey = useCallback(() => {
    const trimmed = apiKeyDraft.trim();
    if (!trimmed) return;
    localStorage.setItem('cz-api-key', trimmed);
    setApiKey(trimmed);
    setShowAuthModal(false);
    addToast('API key saved', 'success');
  }, [apiKeyDraft]);

  // --- THEME SYNC ---
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('cz-theme', theme);
  }, [theme]);

  // --- API FETCHERS ---
  const fetchApi = useCallback(async (endpoint, setter, key) => {
    if (key) setLoadingStates(prev => ({ ...prev, [key]: true }));
    try {
      const res = await fetch(`/api/${endpoint}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const contentType = res.headers.get('content-type') || '';
      const data = contentType.includes('application/json') ? await res.json() : null;
      if (setter && data !== null) {
        setter(data);
      }
    } catch (e) {
      console.error(`Fetch ${endpoint} failed:`, e);
    } finally {
      if (key) setLoadingStates(prev => ({ ...prev, [key]: false }));
    }
  }, []);

  // --- FAVICON BADGE ---
  useEffect(() => {
    const alertCount = alerts.filter(a => a.severity === 'critical').length;
    const canvas = document.createElement('canvas');
    canvas.width = 32;
    canvas.height = 32;
    const ctx = canvas.getContext('2d');

    // Draw base Ω
    ctx.fillStyle = alertCount > 0 ? '#ef4444' : '#6366f1';
    ctx.beginPath();
    ctx.arc(16, 16, 14, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = '#fff';
    ctx.font = 'bold 20px Inter';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText('Ω', 16, 16);

    if (alertCount > 0) {
      ctx.fillStyle = '#ef4444';
      ctx.beginPath();
      ctx.arc(26, 6, 6, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = 1.5;
      ctx.stroke();
    }

    const link = document.querySelector("link[rel~='icon']") || document.createElement('link');
    link.type = 'image/x-icon';
    link.rel = 'icon';
    link.href = canvas.toDataURL('image/x-icon');
    document.getElementsByTagName('head')[0].appendChild(link);
  }, [alerts]);

  const handlePrint = () => {
    window.print();
  };

  // --- INITIAL DATA & POLLING ---
  useEffect(() => {
    fetchApi('status', setSystem, 'system');
    fetchApi('metrics/history', setHistory);

    const fastPoll = setInterval(() => {
      if (activePage === 'overview') fetchApi('metrics/history', setHistory);
    }, 2000);

    const slowPoll = setInterval(() => {
      fetchApi('system', setSystem);
      fetchApi('alerts', setAlerts);
      if (activePage === 'topology') fetchApi('topology', setTopology, 'topology');
      if (activePage === 'analytics') fetchApi('streams', setStreamStats, 'analytics');
      if (activePage === 'mirror') fetchApi('journal/layout', setJournalLayout, 'mirror');
    }, 5000);

    return () => { clearInterval(fastPoll); clearInterval(slowPoll); };
  }, [activePage, fetchApi]);

  // --- EVENT EXPLORER LOGIC ---
  useEffect(() => {
    if (activePage === 'explorer') {
      fetchApi(`events?offset=${eventOffset}&limit=50`, setEvents, 'events');
    }
  }, [activePage, eventOffset, fetchApi]);

  const selectEvent = async (slot) => {
    setSelectedSlot(slot);
    setDetailLoading(true);
    try {
      const res = await fetch(`/api/events/${slot}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setEventDetail({ ...(data.event || {}), ...data });
    } catch (e) {
      addToast('Failed to fetch event detail', 'error');
    } finally {
      setDetailLoading(false);
    }
  };

  // --- ACTIONS ---
  const handleVerify = async () => {
    setIsVerifying(true);
    addToast('Starting formal verification build...', 'info');
    try {
      const res = await fetch('/api/verify', { method: 'POST' });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setVerifyResults(data);
      if (data.success) addToast('Verification PASSED', 'success');
      else addToast('Verification FAILED', 'error');
    } catch (e) {
      addToast('Verification process failed to start', 'error');
    } finally {
      setIsVerifying(false);
    }
  };

  const handleSimulate = async () => {
    try {
      addToast('Injecting synthetic load...', 'info');
      const res = await fetch('/api/simulate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ count: 500 })
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      addToast(`Sequenced ${data.events_created} synthetic events`, 'success');
    } catch (e) {
      addToast('Simulation failed', 'error');
    }
  };

  // --- WEBSOCKET ---
  useEffect(() => {
    const ws = new WebSocket(`ws://${window.location.host}/ws`);
    ws.onopen = () => { setStatus('Connected'); addToast('Sequencer Uplink Active', 'success'); };
    ws.onclose = () => { setStatus('Disconnected'); addToast('Uplink Interrupted', 'error'); };
    ws.onmessage = (e) => {
      const msg = JSON.parse(e.data);
      if (msg.type === 'metrics') {
        const d = msg.data;
        setMetrics(d);
        setHistory(prev => {
          const newPoint = { time: new Date().toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' }), tps: d.tps, bps: d.bps };
          const next = [...prev, newPoint];
          return next.slice(-100);
        });
      }
    };
    return () => ws.close();
  }, []);

  // --- KEYBOARD SHORTCUTS ---
  useEffect(() => {
    const handleKey = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') { e.preventDefault(); setShowCmdPalette(prev => !prev); }
      if (e.key === '1') setActivePage('overview');
      if (e.key === '2') setActivePage('explorer');
      if (e.key === '3') setActivePage('topology');
      if (e.key === '4') setActivePage('analytics');
      if (e.key === '5') setActivePage('mirror');
      if (e.key === '6') setActivePage('alerts');
      if (e.key === '7') setActivePage('verify');
    };
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  }, []);

  // --- COMMAND PALETTE ACTIONS ---
  const cmdActions = [
    { id: 'p1', label: 'Go to Overview', icon: <LayoutDashboard />, shortcut: '1', onSelect: () => setActivePage('overview') },
    { id: 'p2', label: 'Go to Event Explorer', icon: <Database />, shortcut: '2', onSelect: () => setActivePage('explorer') },
    { id: 'p3', label: 'Go to Topology', icon: <Share2 />, shortcut: '3', onSelect: () => setActivePage('topology') },
    { id: 'p4', label: 'Go to Analytics', icon: <BarChart3 />, shortcut: '4', onSelect: () => setActivePage('analytics') },
    { id: 'p5', label: 'Go to Journal Mirror', icon: <Eye />, shortcut: '5', onSelect: () => setActivePage('mirror') },
    { id: 'p6', label: 'Go to Alerts', icon: <Bell />, shortcut: '6', onSelect: () => setActivePage('alerts') },
    { id: 'p7', label: 'Go to Verification', icon: <ShieldCheck />, shortcut: '7', onSelect: () => setActivePage('verify') },
    { id: 'a1', label: 'Simulate Load', icon: <Zap />, description: 'Inject 500 events into the ring', onSelect: handleSimulate },
    { id: 'a2', label: 'Run Verification', icon: <ShieldCheck />, description: 'Execute full testing suite', onSelect: handleVerify },
    { id: 't1', label: 'Theme: Indigo Night', icon: <Settings />, onSelect: () => setTheme('indigo') },
    { id: 't2', label: 'Theme: OLED Black', icon: <Settings />, onSelect: () => setTheme('oled') },
    { id: 't3', label: 'Theme: Emerald Hub', icon: <Settings />, onSelect: () => setTheme('emerald') },
    { id: 'a4', label: 'Print/PDF Report', icon: <Download />, onSelect: handlePrint },
    { id: 'a3', label: 'Settings', icon: <Settings />, onSelect: () => setActivePage('settings') },
    { id: 'a5', label: 'Set API Key', icon: <ShieldCheck />, onSelect: () => setShowAuthModal(true) },
  ];

  return (
    <div className="h-screen flex flex-col bg-bg text-fg">
      <div className="flex-1 flex overflow-hidden">
        <Sidebar activePage={activePage} setPage={setActivePage} status={status} alerts={alerts} />

        <main className="flex-1 overflow-y-auto px-6 py-5 relative">
          <div className="max-w-[1400px] mx-auto">
            {activePage === 'overview' && (
              <OverviewPage
                metrics={metrics}
                history={history}
                system={system}
                topology={topology}
                streamStats={streamStats}
                alerts={alerts}
                triggerSimulate={handleSimulate}
              />
            )}

            {activePage === 'connectors' && <ConnectorsPage />}
            {activePage === 'query' && <QueryConsolePage />}

            {activePage === 'traces' && <TracesPage />}
            {activePage === 'pipelines' && <PipelinesPage />}
            {activePage === 'dashboards' && <DashboardsPage />}

            {activePage === 'explorer' && (
              <ExplorerPage
                data={events.events || []}
                loading={loadingStates.events}
                offset={eventOffset}
                limit={50}
                setOffset={setEventOffset}
                onSelectEvent={selectEvent}
                selectedSlot={selectedSlot}
                detail={eventDetail}
                detailLoading={detailLoading}
              />
            )}
            {activePage === 'topology' && (
              <TopologyPage
                topology={topology}
                loading={loadingStates.topology}
              />
            )}
            {activePage === 'analytics' && (
              <AnalyticsPage
                data={streamStats}
                loading={loadingStates.analytics}
              />
            )}
            {activePage === 'mirror' && (
              <MirrorPage
                layout={journalLayout}
                loading={loadingStates.mirror}
              />
            )}
            {activePage === 'alerts' && (
              <IncidentsPage />
            )}
            {activePage === 'settings' && (
              <SettingsPage />
            )}
            {activePage === 'verify' && (
              <VerifyPage
                onRunTests={handleVerify}
                results={verifyResults}
                running={isVerifying}
              />
            )}
          </div>
        </main>
      </div>

      <StatusBar status={status} metrics={metrics} system={system} />
      <ToastContainer toasts={toasts} removeToast={removeToast} />
      <CommandPalette isOpen={showCmdPalette} onClose={() => setShowCmdPalette(false)} actions={cmdActions} />
      {showAuthModal && (
        <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-6">
          <div className="w-full max-w-lg bg-bg-elevated border border-border rounded-xl p-6 shadow-2xl">
            <h2 className="text-lg font-semibold mb-2">Enter API Key</h2>
            <p className="text-sm text-fg-muted mb-4">
              Copy the startup key from `server.log` (`GENERATED ROOT API KEY`) and paste it here.
            </p>
            <input
              value={apiKeyDraft}
              onChange={(e) => setApiKeyDraft(e.target.value)}
              placeholder="cz_..."
              className="w-full bg-bg border border-border rounded-md px-3 py-2 text-sm focus:border-accent focus:outline-none"
            />
            <div className="mt-4 flex justify-end gap-2">
              <button
                onClick={() => setShowAuthModal(false)}
                className="px-4 py-2 text-sm text-fg-muted hover:text-fg"
              >
                Close
              </button>
              <button
                onClick={saveApiKey}
                className="px-4 py-2 text-sm bg-accent hover:bg-accent-hover text-white rounded-md"
              >
                Save Key
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

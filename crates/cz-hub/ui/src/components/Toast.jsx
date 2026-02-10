import React from 'react';
import { CheckCircle2, XCircle, Info, AlertTriangle, X } from 'lucide-react';

export const Toast = ({ message, type = 'info', onClose }) => {
    React.useEffect(() => {
        const timer = setTimeout(onClose, 4000);
        return () => clearTimeout(timer);
    }, [onClose]);

    const icon = {
        success: <CheckCircle2 className="text-green" size={15} />,
        error: <XCircle className="text-red" size={15} />,
        warn: <AlertTriangle className="text-amber" size={15} />,
        info: <Info className="text-accent" size={15} />,
    }[type];

    return (
        <div className="bg-bg-overlay border border-border rounded-lg px-3 py-2.5 min-w-[280px] flex items-center gap-2.5 shadow-[0_8px_24px_rgba(0,0,0,0.3)] animate-slide-in">
            {icon}
            <span className="text-[13px] text-fg font-medium flex-1">{message}</span>
            <button onClick={onClose} className="text-fg-subtle hover:text-fg transition-colors cursor-pointer">
                <X size={14} />
            </button>
        </div>
    );
};

export const ToastContainer = ({ toasts, removeToast }) => (
    <div className="fixed top-4 right-4 z-[100] flex flex-col gap-2">
        {toasts.map(t => (
            <Toast key={t.id} {...t} onClose={() => removeToast(t.id)} />
        ))}
    </div>
);

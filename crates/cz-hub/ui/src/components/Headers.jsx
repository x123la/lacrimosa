import React from 'react';

export const PageHeader = ({ title, subtitle, actions }) => (
    <header className="mb-5 flex justify-between items-center">
        <div>
            <h2 className="text-[16px] font-semibold tracking-tight text-fg">{title}</h2>
            {subtitle && <p className="text-[13px] text-fg-muted mt-0.5">{subtitle}</p>}
        </div>
        {actions && <div className="flex gap-2">{actions}</div>}
    </header>
);

export const SectionHeader = ({ icon, title, subtitle, extra }) => (
    <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
            {icon && <span className="text-fg-subtle">{React.cloneElement(icon, { size: 14, strokeWidth: 1.6 })}</span>}
            <div>
                <h3 className="text-[13px] font-semibold text-fg">{title}</h3>
                {subtitle && <p className="text-[11px] text-fg-muted">{subtitle}</p>}
            </div>
        </div>
        {extra && <div>{extra}</div>}
    </div>
);

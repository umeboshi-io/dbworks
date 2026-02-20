import { useState, useEffect, useRef } from 'react';
import { api } from '../api/client';
import type { Organization } from '../types';
import './OrgSelector.css';

export type Scope = 'personal' | `org:${string}`;

interface OrgSelectorProps {
  currentScope: Scope;
  onScopeChange: (scope: Scope) => void;
}

function OrgSelector({ currentScope, onScopeChange }: OrgSelectorProps) {
  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    api.listOrganizations().then(setOrganizations).catch(console.error);
  }, []);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  const currentLabel = (() => {
    if (currentScope === 'personal') return 'Personal';
    const orgId = currentScope.replace('org:', '');
    const org = organizations.find((o) => o.id === orgId);
    return org?.name ?? orgId.slice(0, 8) + '...';
  })();

  return (
    <div className="org-selector" ref={ref}>
      <button
        className="org-selector-trigger"
        onClick={() => setOpen(!open)}
        title="Switch scope"
      >
        <svg className="org-selector-icon" width="16" height="16" viewBox="0 0 20 20" fill="none">
          <rect x="2" y="4" width="16" height="14" rx="3" stroke="currentColor" strokeWidth="1.5" fill="none" />
          <path d="M7 4V2.5A1.5 1.5 0 0 1 8.5 1h3A1.5 1.5 0 0 1 13 2.5V4" stroke="currentColor" strokeWidth="1.5" />
          <line x1="2" y1="10" x2="18" y2="10" stroke="currentColor" strokeWidth="1.5" />
        </svg>
        <span className="org-selector-label">{currentLabel}</span>
        <svg className={`org-selector-chevron ${open ? 'open' : ''}`} width="10" height="10" viewBox="0 0 10 10">
          <path d="M2.5 3.75 L5 6.25 L7.5 3.75" stroke="currentColor" strokeWidth="1.5" fill="none" strokeLinecap="round" />
        </svg>
      </button>
      {open && (
        <div className="org-selector-dropdown">
          <div
            className={`org-selector-option ${currentScope === 'personal' ? 'active' : ''}`}
            onClick={() => { onScopeChange('personal'); setOpen(false); }}
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <circle cx="7" cy="5" r="3" stroke="currentColor" strokeWidth="1.2" />
              <path d="M2 13c0-2.76 2.24-5 5-5s5 2.24 5 5" stroke="currentColor" strokeWidth="1.2" fill="none" />
            </svg>
            <span>Personal</span>
          </div>
          {organizations.length > 0 && <div className="org-selector-divider" />}
          {organizations.map((org) => (
            <div
              key={org.id}
              className={`org-selector-option ${currentScope === `org:${org.id}` ? 'active' : ''}`}
              onClick={() => { onScopeChange(`org:${org.id}`); setOpen(false); }}
            >
              <svg width="14" height="14" viewBox="0 0 20 20" fill="none">
                <rect x="2" y="4" width="16" height="14" rx="3" stroke="currentColor" strokeWidth="1.5" fill="none" />
                <path d="M7 4V2.5A1.5 1.5 0 0 1 8.5 1h3A1.5 1.5 0 0 1 13 2.5V4" stroke="currentColor" strokeWidth="1.5" />
                <line x1="2" y1="10" x2="18" y2="10" stroke="currentColor" strokeWidth="1.5" />
              </svg>
              <span>{org.name}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default OrgSelector;

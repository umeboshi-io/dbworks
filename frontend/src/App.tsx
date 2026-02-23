import { useState, useEffect, useCallback, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import DbTypeSelectPage from './pages/DbTypeSelectPage';
import ConnectionPage from './pages/ConnectionPage';
import OrganizationPage from './pages/OrganizationPage';
import TablePage from './pages/TablePage';
import LoginPage from './pages/LoginPage';
import OrgSelector from './components/OrgSelector';
import LanguageSelector from './components/LanguageSelector';
import type { Scope } from './components/OrgSelector';
import { useAuth } from './AuthContext';
import { api } from './api/client';
import type { Connection, TableInfo, Organization } from './types';
import './App.css';

function App() {
  const { user, isLoading, logout } = useAuth();
  const { t } = useTranslation();
  const [connections, setConnections] = useState<Connection[]>([]);
  const [scope, setScope] = useState<Scope>(
    () => (sessionStorage.getItem('dbw:scope') as Scope) || 'personal'
  );
  const [scopedTabs, setScopedTabs] = useState<Record<string, Connection[]>>(
    () => {
      const saved = sessionStorage.getItem('dbw:scopedTabs');
      return saved ? JSON.parse(saved) : {};
    }
  );
  const openTabs = scopedTabs[scope] || [];
  const setOpenTabs = (updater: Connection[] | ((prev: Connection[]) => Connection[])) => {
    setScopedTabs((prev) => {
      const current = prev[scope] || [];
      const next = typeof updater === 'function' ? updater(current) : updater;
      return { ...prev, [scope]: next };
    });
  };
  const [scopedActiveConnId, setScopedActiveConnId] = useState<Record<string, string | null>>(
    () => {
      const saved = sessionStorage.getItem('dbw:scopedActiveConnId');
      return saved ? JSON.parse(saved) : {};
    }
  );
  const [activeConnection, setActiveConnection] = useState<Connection | null>(null);
  const [tables, setTables] = useState<TableInfo[]>([]);
  const [activeTable, setActiveTable] = useState<string | null>(
    () => sessionStorage.getItem('dbw:activeTable')
  );
  const [showConnectionForm, setShowConnectionForm] = useState(false);
  const [showDbTypeSelect, setShowDbTypeSelect] = useState(false);
  const [selectedDbType, setSelectedDbType] = useState<string | null>(null);
  const [showOrgPage, setShowOrgPage] = useState(false);
  const [showConnModal, setShowConnModal] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [tablesOpen, setTablesOpen] = useState(true);

  // Skip sessionStorage saves until initial restore is complete
  const restoredRef = useRef(false);

  // Persist selection state to sessionStorage (only after initial restore)
  useEffect(() => {
    if (!restoredRef.current) return;
    if (activeTable) {
      sessionStorage.setItem('dbw:activeTable', activeTable);
    } else {
      sessionStorage.removeItem('dbw:activeTable');
    }
  }, [activeTable]);

  useEffect(() => {
    if (!restoredRef.current) return;
    if (activeConnection) {
      sessionStorage.setItem('dbw:activeConnId', activeConnection.id);
      setScopedActiveConnId((prev) => ({ ...prev, [scope]: activeConnection.id }));
    } else {
      sessionStorage.removeItem('dbw:activeConnId');
      setScopedActiveConnId((prev) => ({ ...prev, [scope]: null }));
    }
  }, [activeConnection, scope]);

  useEffect(() => {
    if (!restoredRef.current) return;
    sessionStorage.setItem('dbw:scopedActiveConnId', JSON.stringify(scopedActiveConnId));
  }, [scopedActiveConnId]);

  useEffect(() => {
    if (!restoredRef.current) return;
    sessionStorage.setItem('dbw:scope', scope);
  }, [scope]);

  useEffect(() => {
    if (!restoredRef.current) return;
    sessionStorage.setItem('dbw:scopedTabs', JSON.stringify(scopedTabs));
  }, [scopedTabs]);

  const loadConnections = useCallback(async (s: Scope) => {
    try {
      const data = await api.listConnections(s);
      setConnections(data);

      // Restore active connection from session
      if (!restoredRef.current) {
        restoredRef.current = true;
        const savedActiveId = sessionStorage.getItem('dbw:activeConnId');

        // Restore open tabs for current scope from scopedTabs
        const currentScopeTabs = scopedTabs[s] || [];
        if (currentScopeTabs.length > 0) {
          // Re-validate tabs against loaded connections
          const validTabs = currentScopeTabs
            .map((tab: Connection) => data.find((c: Connection) => c.id === tab.id))
            .filter(Boolean) as Connection[];
          if (validTabs.length !== currentScopeTabs.length) {
            setOpenTabs(validTabs);
          }
          if (savedActiveId) {
            const active = validTabs.find((c) => c.id === savedActiveId);
            if (active) setActiveConnection(active);
          }
        }
      }
    } catch (err) {
      console.error('Failed to load connections:', err);
    }
  }, []);

  const loadTables = async (connId: string) => {
    try {
      const data = await api.listTables(connId);
      setTables(data);
    } catch (err) {
      console.error('Failed to load tables:', err);
    }
  };

  useEffect(() => {
    loadConnections(scope);
  }, [scope, loadConnections]);

  useEffect(() => {
    if (activeConnection) {
      loadTables(activeConnection.id);
    } else {
      setTables([]);
      setActiveTable(null);
    }
  }, [activeConnection]);

  const handleConnectionCreated = (conn: Connection) => {
    setConnections((prev) => [...prev, conn]);
    // Add to tabs and activate
    setOpenTabs((prev) => prev.some((t) => t.id === conn.id) ? prev : [...prev, conn]);
    setActiveConnection(conn);
    setActiveTable(null);
    setShowConnectionForm(false);
  };

  const handleSelectConnection = (conn: Connection) => {
    // Add to open tabs if not already there, and make active
    setOpenTabs((prev) => prev.some((t) => t.id === conn.id) ? prev : [...prev, conn]);
    setActiveConnection(conn);
    setActiveTable(null);
    setShowConnModal(false);
  };

  const handleCloseTab = (connId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const remaining = openTabs.filter((t) => t.id !== connId);
    setOpenTabs(remaining);
    if (activeConnection?.id === connId) {
      setActiveConnection(remaining.length > 0 ? remaining[remaining.length - 1] : null);
    }
  };

  const handleDeleteConnection = async (connId: string) => {
    try {
      await api.deleteConnection(connId);
      setConnections((prev) => prev.filter((c) => c.id !== connId));
      setOpenTabs((prev) => prev.filter((t) => t.id !== connId));
      if (activeConnection?.id === connId) {
        const remaining = openTabs.filter((t) => t.id !== connId);
        setActiveConnection(remaining.length > 0 ? remaining[remaining.length - 1] : null);
      }
    } catch (err) {
      console.error('Failed to delete connection:', err);
    }
  };

  const handleOrgJoined = (_org: Organization) => {
    setShowOrgPage(false);
  };

  const handleScopeChange = (newScope: Scope) => {
    setScope(newScope);
    setActiveTable(null);
    setTables([]);

    // Restore last active connection for the new scope
    const lastActiveId = scopedActiveConnId[newScope];
    const newScopeTabs = scopedTabs[newScope] || [];
    if (lastActiveId) {
      const restored = newScopeTabs.find((c) => c.id === lastActiveId) || null;
      setActiveConnection(restored);
    } else {
      setActiveConnection(null);
    }
  };

  const dbIcon = (dbType?: string, size = 16) =>
    dbType === 'mysql' ? (
      <svg width={size} height={size} viewBox="0 0 64 64" fill="none">
        <ellipse cx="32" cy="32" rx="28" ry="28" fill="#00758F" />
        <text x="32" y="42" textAnchor="middle" fontFamily="serif" fontWeight="bold" fontSize="24" fill="#F29111">My</text>
      </svg>
    ) : (
      <svg width={size} height={size} viewBox="0 0 64 64" fill="none">
        <ellipse cx="32" cy="32" rx="28" ry="28" fill="#336791" />
        <text x="32" y="44" textAnchor="middle" fontFamily="serif" fontWeight="bold" fontSize="32" fill="#fff">P</text>
      </svg>
    );

  if (isLoading) {
    return (
      <div className="app-loading">
        <div className="spinner" />
      </div>
    );
  }

  if (!user) {
    return <LoginPage />;
  }

  return (
    <div className="app">
      {/* Connection Tabs Bar */}
      <div className="conn-topbar">
        <div className="conn-topbar-left">
          {/* Open connection tabs */}
          <div className="conn-tabs">
            {openTabs.map((conn) => (
              <button
                key={conn.id}
                className={`conn-tab ${activeConnection?.id === conn.id ? 'active' : ''}`}
                onClick={() => { setActiveConnection(conn); setActiveTable(null); }}
              >
                <span className="conn-tab-icon">{dbIcon(conn.db_type, 14)}</span>
                <span className="conn-tab-name">{conn.name}</span>
                <span
                  className="conn-tab-close"
                  onClick={(e) => handleCloseTab(conn.id, e)}
                >
                  ×
                </span>
              </button>
            ))}
          </div>
        </div>

        <div className="conn-topbar-right">
          <OrgSelector currentScope={scope} onScopeChange={handleScopeChange} />
          <LanguageSelector />
          <button
            className="conn-manager-btn"
            onClick={() => setShowConnModal(true)}
            title={t('common.manageConnections')}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <ellipse cx="12" cy="5" rx="9" ry="3" />
              <path d="M3 5v6c0 1.66 4.03 3 9 3s9-1.34 9-3V5" />
              <path d="M3 11v6c0 1.66 4.03 3 9 3s9-1.34 9-3v-6" />
            </svg>
          </button>
          <button className="btn-icon" onClick={() => setShowOrgPage(true)} title={t('common.manageOrgs')}>
            ⚙
          </button>
          <button className="btn-icon" onClick={logout} title={t('common.logout')}>
            ⏻
          </button>
        </div>
      </div>

      <div className="app-body">
        {/* Sidebar — Tables only */}
        <aside className={`sidebar ${sidebarCollapsed ? 'collapsed' : ''}`}>
          <div className="sidebar-header">
            <div className="logo">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
                <rect x="3" y="3" width="7" height="7" rx="2" fill="var(--accent)" />
                <rect x="14" y="3" width="7" height="7" rx="2" fill="var(--accent)" opacity="0.7" />
                <rect x="3" y="14" width="7" height="7" rx="2" fill="var(--accent)" opacity="0.5" />
                <rect x="14" y="14" width="7" height="7" rx="2" fill="var(--accent)" opacity="0.3" />
              </svg>
              {!sidebarCollapsed && <span>{t('app.title')}</span>}
            </div>
            <button
              className="sidebar-toggle"
              onClick={() => setSidebarCollapsed(!sidebarCollapsed)}
            >
              {sidebarCollapsed ? '→' : '←'}
            </button>
          </div>

          {!sidebarCollapsed && (
            <>
              {activeConnection && tables.length > 0 ? (
                <div className="sidebar-section">
                  <div className="section-header" onClick={() => setTablesOpen(!tablesOpen)} style={{ cursor: 'pointer' }}>
                    <h3>
                      <span className={`chevron ${tablesOpen ? 'open' : ''}`}>▶</span>
                      {t('app.tables')}
                    </h3>
                    <span className="badge">{tables.length}</span>
                  </div>
                  {tablesOpen && (
                  <div className="table-list">
                    {tables.map((t) => (
                      <div
                        key={t.table_name}
                        className={`table-item ${activeTable === t.table_name ? 'active' : ''}`}
                        onClick={() => setActiveTable(t.table_name)}
                      >
                        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                          <rect x="1" y="1" width="12" height="12" rx="2" stroke="currentColor" strokeWidth="1.2" />
                          <line x1="1" y1="5" x2="13" y2="5" stroke="currentColor" strokeWidth="1.2" />
                          <line x1="5" y1="5" x2="5" y2="13" stroke="currentColor" strokeWidth="1.2" />
                        </svg>
                        <span title={t.table_name}>{t.table_name}</span>
                      </div>
                    ))}
                  </div>
                  )}
                </div>
              ) : (
                <div className="sidebar-section">
                  <div className="empty-state">
                    <p>{activeConnection ? t('app.loadingTables') : t('app.selectConnection')}</p>
                  </div>
                </div>
              )}
            </>
          )}
        </aside>

        {/* Main Content */}
        <main className="main-content">
          {showOrgPage && (
            <OrganizationPage
              onClose={() => setShowOrgPage(false)}
              onJoined={handleOrgJoined}
            />
          )}
          {!showOrgPage && showDbTypeSelect && !showConnectionForm && (
            <DbTypeSelectPage
              onSelect={(dbType) => {
                setSelectedDbType(dbType);
                setShowDbTypeSelect(false);
                setShowConnectionForm(true);
              }}
              onCancel={() => setShowDbTypeSelect(false)}
            />
          )}
          {!showOrgPage && showConnectionForm && selectedDbType && (
            <ConnectionPage
              dbType={selectedDbType}
              scope={scope}
              onCreated={handleConnectionCreated}
              onCancel={() => { setShowConnectionForm(false); setSelectedDbType(null); }}
              onBack={() => { setShowConnectionForm(false); setShowDbTypeSelect(true); }}
            />
          )}
          {!showOrgPage && !showConnectionForm && !showDbTypeSelect && activeConnection && activeTable && (
            <TablePage
              key={`${activeConnection.id}:${activeTable}`}
              connectionId={activeConnection.id}
              connectionName={activeConnection.name}
              connectionDetail={`${activeConnection.host}:${activeConnection.port}/${activeConnection.database}`}
              tableName={activeTable}
            />
          )}
          {!showOrgPage && !showConnectionForm && !showDbTypeSelect && !activeTable && (
            <div className="welcome">
              <div className="welcome-content">
                <div className="welcome-icon">
                  <svg width="64" height="64" viewBox="0 0 64 64" fill="none">
                    <rect x="8" y="8" width="20" height="20" rx="4" fill="var(--accent)" opacity="0.8" />
                    <rect x="36" y="8" width="20" height="20" rx="4" fill="var(--accent)" opacity="0.6" />
                    <rect x="8" y="36" width="20" height="20" rx="4" fill="var(--accent)" opacity="0.4" />
                    <rect x="36" y="36" width="20" height="20" rx="4" fill="var(--accent)" opacity="0.2" />
                  </svg>
                </div>
                <h1>{t('app.title')}</h1>
                <p>{t('app.subtitle')}</p>
                {!activeConnection ? (
                  <button
                    className="btn btn-primary btn-lg"
                    onClick={() => setShowDbTypeSelect(true)}
                  >
                    {t('app.connectToDb')}
                  </button>
                ) : (
                  <>
                    <div className="welcome-connection">
                      {dbIcon(activeConnection.db_type)}
                      <span className="welcome-conn-name">{activeConnection.name}</span>
                      <span className="welcome-conn-detail">{activeConnection.host}:{activeConnection.port}/{activeConnection.database}</span>
                    </div>
                    <p className="hint">{t('app.selectTable')}</p>
                  </>
                )}
              </div>
            </div>
          )}
        </main>
      </div>

      {/* Connections Management Modal */}
      {showConnModal && (
        <div className="modal-overlay" onClick={() => setShowConnModal(false)}>
          <div className="modal-content conn-modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>{t('app.connections')}</h2>
              <button className="modal-close" onClick={() => setShowConnModal(false)}>×</button>
            </div>


            <div className="modal-body">
              {connections.length === 0 ? (
                <div className="conn-modal-empty">
                  <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" opacity="0.5">
                    <ellipse cx="12" cy="5" rx="9" ry="3" />
                    <path d="M3 5v6c0 1.66 4.03 3 9 3s9-1.34 9-3V5" />
                    <path d="M3 11v6c0 1.66 4.03 3 9 3s9-1.34 9-3v-6" />
                  </svg>
                  <p>{t('app.noConnections')}</p>
                  <button
                    className="btn btn-primary"
                    onClick={() => { setShowConnModal(false); setShowDbTypeSelect(true); }}
                  >
                    {t('app.addConnection')}
                  </button>
                </div>
              ) : (
                <div className="conn-modal-list">
                  {connections.map((conn) => {
                    const isOpen = openTabs.some((t) => t.id === conn.id);
                    return (
                      <div
                        key={conn.id}
                        className={`conn-modal-item ${isOpen ? 'open' : ''}`}
                      >
                        <div className="conn-modal-item-left" onClick={() => handleSelectConnection(conn)}>
                          <div className="conn-modal-icon">{dbIcon(conn.db_type, 20)}</div>
                          <div className="conn-modal-info">
                            <span className="conn-modal-name">{conn.name}</span>
                            <span className="conn-modal-detail">
                              {conn.host}:{conn.port}/{conn.database}
                            </span>
                          </div>
                        </div>
                        <div className="conn-modal-actions">
                          {isOpen && <span className="conn-modal-badge">{t('app.open')}</span>}
                          <button
                            className="btn-icon btn-danger"
                            onClick={() => handleDeleteConnection(conn.id)}
                            title={t('common.delete')}
                          >
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                              <polyline points="3 6 5 6 21 6" />
                              <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" />
                            </svg>
                          </button>
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>

            <div className="modal-footer">
              <button
                className="btn btn-primary"
                onClick={() => { setShowConnModal(false); setShowDbTypeSelect(true); }}
              >
                {t('app.newConnectionBtn')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;

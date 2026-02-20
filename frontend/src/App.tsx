import { useState, useEffect, useCallback } from 'react';
import ConnectionPage from './pages/ConnectionPage';
import OrganizationPage from './pages/OrganizationPage';
import TablePage from './pages/TablePage';
import LoginPage from './pages/LoginPage';
import OrgSelector from './components/OrgSelector';
import type { Scope } from './components/OrgSelector';
import { useAuth } from './AuthContext';
import { api } from './api/client';
import type { Connection, TableInfo, Organization } from './types';
import './App.css';

function App() {
  const { user, isLoading, logout } = useAuth();
  const [connections, setConnections] = useState<Connection[]>([]);
  const [activeConnection, setActiveConnection] = useState<Connection | null>(null);
  const [tables, setTables] = useState<TableInfo[]>([]);
  const [activeTable, setActiveTable] = useState<string | null>(null);
  const [showConnectionForm, setShowConnectionForm] = useState(false);
  const [showOrgPage, setShowOrgPage] = useState(false);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [connectionsOpen, setConnectionsOpen] = useState(true);
  const [tablesOpen, setTablesOpen] = useState(true);
  const [scope, setScope] = useState<Scope>('personal');

  const loadConnections = useCallback(async (s: Scope) => {
    try {
      const data = await api.listConnections(s);
      setConnections(data);
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
    setActiveConnection(conn);
    setShowConnectionForm(false);
  };

  const handleOrgJoined = (_org: Organization) => {
    setShowOrgPage(false);
  };

  const handleScopeChange = (newScope: Scope) => {
    setScope(newScope);
    setActiveConnection(null);
    setActiveTable(null);
  };

  const handleDeleteConnection = async (connId: string) => {
    try {
      await api.deleteConnection(connId);
      setConnections((prev) => prev.filter((c) => c.id !== connId));
      if (activeConnection?.id === connId) {
        setActiveConnection(null);
      }
    } catch (err) {
      console.error('Failed to delete connection:', err);
    }
  };

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
      {/* Sidebar */}
      <aside className={`sidebar ${sidebarCollapsed ? 'collapsed' : ''}`}>
        <div className="sidebar-header">
          <div className="logo">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
              <rect x="3" y="3" width="7" height="7" rx="2" fill="var(--accent)" />
              <rect x="14" y="3" width="7" height="7" rx="2" fill="var(--accent)" opacity="0.7" />
              <rect x="3" y="14" width="7" height="7" rx="2" fill="var(--accent)" opacity="0.5" />
              <rect x="14" y="14" width="7" height="7" rx="2" fill="var(--accent)" opacity="0.3" />
            </svg>
            {!sidebarCollapsed && <span>DBWorks</span>}
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
            {/* User info */}
            <div className="sidebar-user">
              {user.avatar_url && (
                <img src={user.avatar_url} alt="" className="user-avatar" />
              )}
              {!user.avatar_url && (
                <div className="user-avatar-placeholder">
                  {user.name.charAt(0).toUpperCase()}
                </div>
              )}
              <div className="user-info">
                <span className="user-name">{user.name}</span>
                <span className="user-email">{user.email}</span>
              </div>
              <button className="btn-icon" onClick={() => { setShowOrgPage(true); setShowConnectionForm(false); }} title="Manage Organizations">
                ⚙
              </button>
              <button className="btn-icon" onClick={logout} title="ログアウト">
                ⏻
              </button>
            </div>
            {/* Org selector */}
            <OrgSelector currentScope={scope} onScopeChange={handleScopeChange} />
            {/* Connections */}
            <div className="sidebar-section">
              <div className="section-header" onClick={() => setConnectionsOpen(!connectionsOpen)} style={{ cursor: 'pointer' }}>
                <h3>
                  <span className={`chevron ${connectionsOpen ? 'open' : ''}`}>▶</span>
                  Connections
                </h3>
                <button
                  className="btn-icon"
                  onClick={(e) => { e.stopPropagation(); setShowConnectionForm(true); }}
                  title="Add Connection"
                >
                  +
                </button>
              </div>
              {connectionsOpen && (
              <div className="connection-list">
                {connections.map((conn) => (
                  <div
                    key={conn.id}
                    className={`connection-item ${activeConnection?.id === conn.id ? 'active' : ''}`}
                    onClick={() => {
                      setActiveTable(null);
                      setActiveConnection(conn);
                    }}
                  >
                    <div className="connection-icon">
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <ellipse cx="8" cy="4" rx="6" ry="2.5" stroke="currentColor" strokeWidth="1.5" fill="none" />
                        <path d="M2 4v4c0 1.38 2.69 2.5 6 2.5S14 9.38 14 8V4" stroke="currentColor" strokeWidth="1.5" />
                        <path d="M2 8v4c0 1.38 2.69 2.5 6 2.5S14 13.38 14 12V8" stroke="currentColor" strokeWidth="1.5" />
                      </svg>
                    </div>
                    <div className="connection-info">
                      <span className="connection-name">{conn.name}</span>
                      <span className="connection-detail">{conn.host}:{conn.port}/{conn.database}</span>
                    </div>
                    <button
                      className="btn-icon btn-danger"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteConnection(conn.id);
                      }}
                      title="Delete"
                    >
                      ×
                    </button>
                  </div>
                ))}
                {connections.length === 0 && (
                  <div className="empty-state">
                    <p>No connections yet</p>
                    <button
                      className="btn btn-primary"
                      onClick={() => setShowConnectionForm(true)}
                    >
                      Add Connection
                    </button>
                  </div>
                )}
              </div>
              )}
            </div>

            {/* Tables */}
            {activeConnection && tables.length > 0 && (
              <div className="sidebar-section">
                <div className="section-header" onClick={() => setTablesOpen(!tablesOpen)} style={{ cursor: 'pointer' }}>
                  <h3>
                    <span className={`chevron ${tablesOpen ? 'open' : ''}`}>▶</span>
                    Tables
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
        {!showOrgPage && showConnectionForm && (
          <ConnectionPage
            onCreated={handleConnectionCreated}
            onCancel={() => setShowConnectionForm(false)}
          />
        )}
        {!showOrgPage && !showConnectionForm && activeConnection && activeTable && (
          <TablePage
            connectionId={activeConnection.id}
            connectionName={activeConnection.name}
            connectionDetail={`${activeConnection.host}:${activeConnection.port}/${activeConnection.database}`}
            tableName={activeTable}
          />
        )}
        {!showOrgPage && !showConnectionForm && !activeTable && (
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
              <h1>DBWorks</h1>
              <p>Database schema-driven CRUD manager</p>
              {!activeConnection ? (
                <button
                  className="btn btn-primary btn-lg"
                  onClick={() => setShowConnectionForm(true)}
                >
                  Connect to Database
                </button>
              ) : (
                <>
                  <div className="welcome-connection">
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                      <ellipse cx="8" cy="4" rx="6" ry="2.5" stroke="currentColor" strokeWidth="1.5" fill="none" />
                      <path d="M2 4v4c0 1.38 2.69 2.5 6 2.5S14 9.38 14 8V4" stroke="currentColor" strokeWidth="1.5" />
                      <path d="M2 8v4c0 1.38 2.69 2.5 6 2.5S14 13.38 14 12V8" stroke="currentColor" strokeWidth="1.5" />
                    </svg>
                    <span className="welcome-conn-name">{activeConnection.name}</span>
                    <span className="welcome-conn-detail">{activeConnection.host}:{activeConnection.port}/{activeConnection.database}</span>
                  </div>
                  <p className="hint">← Select a table from the sidebar</p>
                </>
              )}
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;

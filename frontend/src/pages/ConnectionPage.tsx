import { useState, type ChangeEvent, type FormEvent } from 'react';
import { api } from '../api/client';
import type { Connection } from '../types';
import './ConnectionPage.css';

interface ConnectionPageProps {
  dbType: string;
  onCreated: (conn: Connection) => void;
  onCancel: () => void;
  onBack: () => void;
}

interface ConnectionForm {
  name: string;
  host: string;
  port: string;
  database: string;
  user: string;
  password: string;
}

const DB_DEFAULTS: Record<string, { port: string; user: string; label: string }> = {
  postgres: { port: '5432', user: 'postgres', label: 'PostgreSQL' },
  mysql: { port: '3306', user: 'root', label: 'MySQL' },
};

function ConnectionPage({ dbType, onCreated, onCancel, onBack }: ConnectionPageProps) {
  const defaults = DB_DEFAULTS[dbType] ?? DB_DEFAULTS.postgres;

  const [form, setForm] = useState<ConnectionForm>({
    name: '',
    host: 'localhost',
    port: defaults.port,
    database: '',
    user: '',
    password: '',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    setForm((prev) => ({ ...prev, [e.target.name]: e.target.value }));
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const conn = await api.createConnection({
        ...form,
        db_type: dbType,
        port: parseInt(form.port, 10),
      });
      onCreated(conn);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="connection-page">
      <div className="connection-card">
        <div className="card-header">
          <div className="card-header-top">
            <button className="btn-back" onClick={onBack} title="Back to DB type selection">
              ← Back
            </button>
          </div>
          <h2>New {defaults.label} Connection</h2>
          <p>Enter your {defaults.label} connection details</p>
        </div>
        {error && <div className="alert alert-error">{error}</div>}
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="conn-name">Connection Name</label>
            <input
              id="conn-name"
              type="text"
              name="name"
              value={form.name}
              onChange={handleChange}
              placeholder="My Database"
              required
            />
          </div>
          <div className="form-row">
            <div className="form-group flex-grow">
              <label htmlFor="conn-host">Host</label>
              <input
                id="conn-host"
                type="text"
                name="host"
                value={form.host}
                onChange={handleChange}
                placeholder="localhost"
                required
              />
            </div>
            <div className="form-group" style={{ width: '120px' }}>
              <label htmlFor="conn-port">Port</label>
              <input
                id="conn-port"
                type="number"
                name="port"
                value={form.port}
                onChange={handleChange}
                placeholder={defaults.port}
                required
              />
            </div>
          </div>
          <div className="form-group">
            <label htmlFor="conn-database">Database</label>
            <input
              id="conn-database"
              type="text"
              name="database"
              value={form.database}
              onChange={handleChange}
              placeholder="my_database"
              required
            />
          </div>
          <div className="form-row">
            <div className="form-group flex-grow">
              <label htmlFor="conn-user">User</label>
              <input
                id="conn-user"
                type="text"
                name="user"
                value={form.user}
                onChange={handleChange}
                placeholder={defaults.user}
                required
              />
            </div>
            <div className="form-group flex-grow">
              <label htmlFor="conn-password">Password</label>
              <input
                id="conn-password"
                type="password"
                name="password"
                value={form.password}
                onChange={handleChange}
                placeholder="••••••••"
              />
            </div>
          </div>
          <div className="form-actions">
            <button type="button" className="btn btn-ghost" onClick={onCancel}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary" disabled={loading}>
              {loading ? (
                <span className="loading-spinner" />
              ) : (
                'Connect'
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default ConnectionPage;

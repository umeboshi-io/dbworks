import { useState, useEffect, type ChangeEvent, type FormEvent } from "react";
import { api } from "../api/client";
import type { Organization } from "../types";
import "./OrganizationPage.css";

interface OrganizationPageProps {
  onClose: () => void;
  onJoined: (org: Organization) => void;
}

function OrganizationPage({ onClose, onJoined }: OrganizationPageProps) {
  const [organizations, setOrganizations] = useState<Organization[]>([]);
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [listLoading, setListLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"create" | "list">("create");

  useEffect(() => {
    loadOrganizations();
  }, []);

  const loadOrganizations = async () => {
    setListLoading(true);
    try {
      const orgs = await api.listOrganizations();
      setOrganizations(orgs);
    } catch (err) {
      console.error("Failed to load organizations:", err);
    } finally {
      setListLoading(false);
    }
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      const org = await api.createOrganization({ name });
      setOrganizations((prev) => [...prev, org]);
      onJoined(org);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="org-page">
      <div className="org-card">
        <div className="card-header">
          <h2>Organizations</h2>
          <p>チームで DB を共有するための組織を管理します</p>
        </div>

        <div className="org-tabs">
          <button
            className={`org-tab ${activeTab === "create" ? "active" : ""}`}
            onClick={() => setActiveTab("create")}
          >
            新規作成
          </button>
          <button
            className={`org-tab ${activeTab === "list" ? "active" : ""}`}
            onClick={() => setActiveTab("list")}
          >
            一覧 ({organizations.length})
          </button>
        </div>

        {activeTab === "create" && (
          <>
            {error && <div className="alert alert-error">{error}</div>}
            <form onSubmit={handleSubmit}>
              <div className="form-group">
                <label htmlFor="org-name">Organization Name</label>
                <input
                  id="org-name"
                  type="text"
                  value={name}
                  onChange={(e: ChangeEvent<HTMLInputElement>) =>
                    setName(e.target.value)
                  }
                  placeholder="My Team"
                  required
                  autoFocus
                />
              </div>
              <div className="form-actions">
                <button
                  type="button"
                  className="btn btn-ghost"
                  onClick={onClose}
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={loading || !name.trim()}
                >
                  {loading ? (
                    <span className="loading-spinner" />
                  ) : (
                    "Create Organization"
                  )}
                </button>
              </div>
            </form>
          </>
        )}

        {activeTab === "list" && (
          <div className="org-list">
            {listLoading ? (
              <div className="org-list-loading">
                <span className="spinner" />
              </div>
            ) : organizations.length === 0 ? (
              <div className="org-empty">
                <p>まだ組織がありません</p>
                <button
                  className="btn btn-primary"
                  onClick={() => setActiveTab("create")}
                >
                  Create Organization
                </button>
              </div>
            ) : (
              <>
                {organizations.map((org) => (
                  <div key={org.id} className="org-item">
                    <div className="org-item-icon">
                      <svg
                        width="20"
                        height="20"
                        viewBox="0 0 20 20"
                        fill="none"
                      >
                        <rect
                          x="2"
                          y="4"
                          width="16"
                          height="14"
                          rx="3"
                          stroke="currentColor"
                          strokeWidth="1.5"
                          fill="none"
                        />
                        <path
                          d="M7 4V2.5A1.5 1.5 0 0 1 8.5 1h3A1.5 1.5 0 0 1 13 2.5V4"
                          stroke="currentColor"
                          strokeWidth="1.5"
                        />
                        <line
                          x1="2"
                          y1="10"
                          x2="18"
                          y2="10"
                          stroke="currentColor"
                          strokeWidth="1.5"
                        />
                      </svg>
                    </div>
                    <div className="org-item-info">
                      <span className="org-item-name">{org.name}</span>
                      <span className="org-item-id">
                        {org.id.slice(0, 8)}...
                      </span>
                    </div>
                  </div>
                ))}
                <div className="form-actions" style={{ marginTop: "1rem" }}>
                  <button className="btn btn-ghost" onClick={onClose}>
                    Close
                  </button>
                </div>
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default OrganizationPage;

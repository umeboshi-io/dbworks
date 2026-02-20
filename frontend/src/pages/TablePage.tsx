import { useState, useEffect, useCallback } from 'react';
import { api } from '../api/client';
import DataTable from '../components/DataTable';
import DynamicForm from '../components/DynamicForm';
import type { TableSchema, RowsResponse, RowData, ListRowsParams } from '../types';
import './TablePage.css';

interface TablePageProps {
  connectionId: string;
  connectionName: string;
  connectionDetail: string;
  tableName: string;
}

function TablePage({ connectionId, connectionName, connectionDetail, tableName }: TablePageProps) {
  const [schema, setSchema] = useState<TableSchema | null>(null);
  const [rowsData, setRowsData] = useState<RowsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Query state
  const [page, setPage] = useState(1);
  const [perPage] = useState(20);
  const [sortBy, setSortBy] = useState<string | null>(null);
  const [sortOrder, setSortOrder] = useState('asc');
  const [filterColumn, setFilterColumn] = useState('');
  const [filterOp, setFilterOp] = useState('like');
  const [filterValue, setFilterValue] = useState('');
  const [activeFilter, setActiveFilter] = useState<string | null>(null);

  // Modal state
  const [showForm, setShowForm] = useState(false);
  const [editingRow, setEditingRow] = useState<RowData | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<RowData | null>(null);

  const loadSchema = useCallback(async () => {
    try {
      const data = await api.getTableSchema(connectionId, tableName);
      setSchema(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  }, [connectionId, tableName]);

  const loadRows = useCallback(async () => {
    setLoading(true);
    try {
      const params: ListRowsParams = { page, per_page: perPage };
      if (sortBy) {
        params.sort_by = sortBy;
        params.sort_order = sortOrder;
      }
      if (activeFilter) {
        params.filter = activeFilter;
      }
      const data = await api.listRows(connectionId, tableName, params);
      setRowsData(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, [connectionId, tableName, page, perPage, sortBy, sortOrder, activeFilter]);

  useEffect(() => {
    setPage(1);
    setSortBy(null);
    setSortOrder('asc');
    setActiveFilter(null);
    setFilterColumn('');
    setFilterOp('like');
    setFilterValue('');
    loadSchema();
  }, [connectionId, tableName, loadSchema]);

  useEffect(() => {
    if (schema) loadRows();
  }, [schema, page, sortBy, sortOrder, activeFilter, loadRows]);

  const handleSort = (column: string) => {
    if (sortBy === column) {
      setSortOrder((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortBy(column);
      setSortOrder('asc');
    }
  };

  const handleFilter = () => {
    if (filterColumn && filterValue) {
      setActiveFilter(`${filterColumn}:${filterOp}:${filterValue}`);
      setPage(1);
    }
  };

  const clearFilter = () => {
    setActiveFilter(null);
    setFilterColumn('');
    setFilterOp('like');
    setFilterValue('');
    setPage(1);
  };

  const handleCreate = async (data: RowData) => {
    await api.createRow(connectionId, tableName, data);
    setShowForm(false);
    loadRows();
  };

  const handleUpdate = async (data: RowData) => {
    if (!schema || !editingRow) return;
    const pkCol = schema.primary_key_columns[0];
    const pkValue = String(editingRow[pkCol]);
    await api.updateRow(connectionId, tableName, pkValue, data);
    setEditingRow(null);
    setShowForm(false);
    loadRows();
  };

  const handleDelete = async () => {
    if (!schema || !deleteConfirm) return;
    const pkCol = schema.primary_key_columns[0];
    const pkValue = String(deleteConfirm[pkCol]);
    try {
      await api.deleteRow(connectionId, tableName, pkValue);
      setDeleteConfirm(null);
      loadRows();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  const totalPages = rowsData ? Math.ceil(rowsData.total_count / perPage) : 0;

  return (
    <div className="table-page">
      <div className="connection-breadcrumb">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <ellipse cx="8" cy="4" rx="6" ry="2.5" stroke="currentColor" strokeWidth="1.5" fill="none" />
          <path d="M2 4v4c0 1.38 2.69 2.5 6 2.5S14 9.38 14 8V4" stroke="currentColor" strokeWidth="1.5" />
          <path d="M2 8v4c0 1.38 2.69 2.5 6 2.5S14 13.38 14 12V8" stroke="currentColor" strokeWidth="1.5" />
        </svg>
        <span className="breadcrumb-name">{connectionName}</span>
        <span className="breadcrumb-sep">/</span>
        <span className="breadcrumb-detail">{connectionDetail}</span>
      </div>
      <div className="table-page-header">
        <div className="table-title">
          <h2>{tableName}</h2>
          {rowsData && (
            <span className="row-count">{rowsData.total_count} rows</span>
          )}
        </div>
        <button
          className="btn btn-primary"
          onClick={() => {
            setEditingRow(null);
            setShowForm(true);
          }}
        >
          + New Row
        </button>
      </div>

      {/* Filter Bar */}
      {schema && (
        <div className="filter-bar">
          <select
            value={filterColumn}
            onChange={(e) => setFilterColumn(e.target.value)}
            className="filter-select"
          >
            <option value="">Column...</option>
            {schema.columns.map((col) => (
              <option key={col.column_name} value={col.column_name}>
                {col.column_name}
              </option>
            ))}
          </select>
          <select
            value={filterOp}
            onChange={(e) => setFilterOp(e.target.value)}
            className="filter-select"
          >
            <option value="like">contains</option>
            <option value="eq">equals</option>
            <option value="neq">not equals</option>
            <option value="gt">greater than</option>
            <option value="gte">greater or equal</option>
            <option value="lt">less than</option>
            <option value="lte">less or equal</option>
          </select>
          <input
            type="text"
            value={filterValue}
            onChange={(e) => setFilterValue(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleFilter()}
            placeholder="Value..."
            className="filter-input"
          />
          <button className="btn btn-ghost" onClick={handleFilter}>
            Filter
          </button>
          {activeFilter && (
            <button className="btn btn-ghost btn-danger" onClick={clearFilter}>
              Clear
            </button>
          )}
        </div>
      )}

      {error && <div className="alert alert-error">{error}</div>}

      {/* Data Table */}
      {schema && rowsData && (
        <DataTable
          schema={schema}
          rows={rowsData.rows}
          loading={loading}
          sortBy={sortBy}
          sortOrder={sortOrder}
          onSort={handleSort}
          onEdit={(row) => {
            setEditingRow(row);
            setShowForm(true);
          }}
          onDelete={(row) => setDeleteConfirm(row)}
        />
      )}

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="pagination">
          <button
            className="btn btn-ghost"
            disabled={page <= 1}
            onClick={() => setPage((p) => p - 1)}
          >
            ← Prev
          </button>
          <span className="page-info">
            Page {page} of {totalPages}
          </span>
          <button
            className="btn btn-ghost"
            disabled={page >= totalPages}
            onClick={() => setPage((p) => p + 1)}
          >
            Next →
          </button>
        </div>
      )}

      {/* Create/Edit Modal */}
      {showForm && schema && (
        <div className="modal-overlay" onClick={() => setShowForm(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3>{editingRow ? 'Edit Row' : 'New Row'}</h3>
              <button className="btn-icon" onClick={() => setShowForm(false)}>
                ×
              </button>
            </div>
            <DynamicForm
              schema={schema}
              initialData={editingRow}
              onSubmit={editingRow ? handleUpdate : handleCreate}
              onCancel={() => setShowForm(false)}
            />
          </div>
        </div>
      )}

      {/* Delete Confirmation */}
      {deleteConfirm && (
        <div className="modal-overlay" onClick={() => setDeleteConfirm(null)}>
          <div className="modal modal-sm" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h3>Delete Row</h3>
            </div>
            <p style={{ padding: '0 1.5rem', color: 'var(--text-muted)' }}>
              Are you sure you want to delete this row? This action cannot be undone.
            </p>
            <div className="modal-actions">
              <button
                className="btn btn-ghost"
                onClick={() => setDeleteConfirm(null)}
              >
                Cancel
              </button>
              <button className="btn btn-danger" onClick={handleDelete}>
                Delete
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default TablePage;

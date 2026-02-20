import type { ColumnInfo, TableSchema, RowData } from '../types';
import './DataTable.css';

interface DataTableProps {
  schema: TableSchema;
  rows: RowData[];
  loading: boolean;
  sortBy: string | null;
  sortOrder: string;
  onSort: (column: string) => void;
  onEdit: (row: RowData) => void;
  onDelete: (row: RowData) => void;
}

function DataTable({ schema, rows, loading, sortBy, sortOrder, onSort, onEdit, onDelete }: DataTableProps) {
  const columns: ColumnInfo[] = schema.columns;

  const getSortIcon = (colName: string): string => {
    if (sortBy !== colName) return '↕';
    return sortOrder === 'asc' ? '↑' : '↓';
  };

  const formatValue = (value: unknown, dataType: string): React.ReactNode => {
    if (value === null || value === undefined) {
      return <span className="null-value">NULL</span>;
    }
    if (typeof value === 'boolean') {
      return (
        <span className={`bool-badge ${value ? 'true' : 'false'}`}>
          {value ? 'true' : 'false'}
        </span>
      );
    }
    if (dataType === 'timestamp without time zone' || dataType === 'timestamp with time zone') {
      return new Date(value as string).toLocaleString('ja-JP');
    }
    if (typeof value === 'object') {
      return JSON.stringify(value);
    }
    const str = String(value);
    if (str.length > 100) return str.slice(0, 100) + '…';
    return str;
  };

  return (
    <div className="data-table-wrapper">
      <table className="data-table">
        <thead>
          <tr>
            {columns.map((col) => (
              <th
                key={col.column_name}
                onClick={() => onSort(col.column_name)}
                className={`sortable ${col.is_primary_key ? 'pk' : ''}`}
              >
                <span className="th-content">
                  <span className="th-name">
                    {col.column_name}
                    {col.is_primary_key && <span className="pk-badge">PK</span>}
                  </span>
                  <span className="sort-icon">{getSortIcon(col.column_name)}</span>
                </span>
                <span className="th-type">{col.data_type}</span>
              </th>
            ))}
            <th className="actions-th">Actions</th>
          </tr>
        </thead>
        <tbody>
          {loading ? (
            <tr>
              <td colSpan={columns.length + 1} className="loading-cell">
                <div className="loading-spinner" />
                Loading...
              </td>
            </tr>
          ) : rows.length === 0 ? (
            <tr>
              <td colSpan={columns.length + 1} className="empty-cell">
                No rows found
              </td>
            </tr>
          ) : (
            rows.map((row, idx) => (
              <tr key={idx}>
                {columns.map((col) => (
                  <td key={col.column_name}>
                    {formatValue(row[col.column_name], col.data_type)}
                  </td>
                ))}
                <td className="actions-cell">
                  <button
                    className="btn-icon btn-edit"
                    onClick={() => onEdit(row)}
                    title="Edit"
                  >
                    ✎
                  </button>
                  <button
                    className="btn-icon btn-danger"
                    onClick={() => onDelete(row)}
                    title="Delete"
                  >
                    ✕
                  </button>
                </td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

export default DataTable;

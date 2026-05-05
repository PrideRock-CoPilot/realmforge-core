import { useId } from 'react'

// Column definition — consumers declare these for their row type.
export interface ColumnDef<TRow> {
  key: string
  header: string
  // Render function receives the row, returns the cell content
  cell: (row: TRow) => React.ReactNode
  sortable?: boolean
  // Current sort direction for this column (controlled by parent)
  sortDirection?: 'ascending' | 'descending' | 'none'
  onSort?: () => void
}

interface PaginationProps {
  pageIndex: number    // 0-based
  pageSize: number
  onPageChange: (page: number) => void
}

interface DataTableProps<TRow> {
  columns: ColumnDef<TRow>[]
  data: TRow[]
  totalCount: number
  pagination: PaginationProps
  isLoading?: boolean
  emptyState?: React.ReactNode
  caption: string   // Required — describes the table to screen readers
}

export function DataTable<TRow>({
  columns,
  data,
  totalCount,
  pagination,
  isLoading = false,
  emptyState,
  caption,
}: DataTableProps<TRow>) {
  const navId = useId()
  const totalPages = Math.ceil(totalCount / pagination.pageSize)
  const currentPage = pagination.pageIndex   // 0-based internally
  const displayPage = currentPage + 1        // 1-based for display

  return (
    <div className="data-table-wrapper">
      {/* aria-busy signals loading state to screen readers */}
      <table className="data-table" aria-busy={isLoading}>
        {/* Native <caption> is preferred over aria-label for tables */}
        <caption className="data-table__caption">{caption}</caption>

        <thead>
          <tr>
            {columns.map((col) => (
              <th
                key={col.key}
                scope="col"
                aria-sort={col.sortable ? (col.sortDirection ?? 'none') : undefined}
                className={`data-table__th${col.sortable ? ' data-table__th--sortable' : ''}`}
              >
                {col.sortable ? (
                  <button
                    type="button"
                    onClick={col.onSort}
                    className="data-table__sort-btn"
                    aria-label={`Sort by ${col.header}${
                      col.sortDirection === 'ascending'
                        ? ', currently ascending'
                        : col.sortDirection === 'descending'
                          ? ', currently descending'
                          : ''
                    }`}
                  >
                    {col.header}
                    <SortIndicator direction={col.sortDirection ?? 'none'} />
                  </button>
                ) : (
                  col.header
                )}
              </th>
            ))}
          </tr>
        </thead>

        <tbody>
          {isLoading ? (
            <tr>
              <td colSpan={columns.length} className="data-table__loading">
                {/* aria-busy on table handles announcement; this is visual feedback */}
                Loading…
              </td>
            </tr>
          ) : data.length === 0 ? (
            <tr>
              <td colSpan={columns.length} className="data-table__empty">
                {emptyState ?? 'No results.'}
              </td>
            </tr>
          ) : (
            data.map((row, rowIndex) => (
              <tr key={rowIndex} className="data-table__row">
                {columns.map((col) => (
                  <td key={col.key} className="data-table__td">
                    {col.cell(row)}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>

      {/* Pagination is a <nav> landmark — keyboard and AT users can navigate to it */}
      <nav aria-label="Table pagination" id={navId} className="data-table__pagination">
        <button
          type="button"
          onClick={() => pagination.onPageChange(currentPage - 1)}
          disabled={currentPage === 0}
          aria-label="Previous page"
          className="data-table__page-btn"
        >
          ‹ Prev
        </button>

        <span aria-live="polite" aria-atomic="true" className="data-table__page-info">
          Page {displayPage} of {totalPages || 1}
        </span>

        <button
          type="button"
          onClick={() => pagination.onPageChange(currentPage + 1)}
          disabled={currentPage >= totalPages - 1}
          aria-label="Next page"
          className="data-table__page-btn"
        >
          Next ›
        </button>
      </nav>
    </div>
  )
}

function SortIndicator({ direction }: { direction: 'ascending' | 'descending' | 'none' }) {
  // aria-hidden — the parent th's aria-sort attribute carries the semantic meaning
  const icon = direction === 'ascending' ? '↑' : direction === 'descending' ? '↓' : '↕'
  return (
    <span aria-hidden="true" className="data-table__sort-icon">
      {icon}
    </span>
  )
}

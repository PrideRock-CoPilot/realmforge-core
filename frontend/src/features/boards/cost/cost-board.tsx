import { useState } from 'react'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { formatCost, formatCostDelta, relativeTime } from '@/lib/formatters'
import { DollarSign, TrendingUp, TrendingDown } from 'lucide-react'

// ─────────────────────────────────────────────
// Cost Board VIEW-BOARDS-COST
// Token, build, storage, runtime, rework costs
// NOTE: Uses mock data until cost-tracking API is available.
// ─────────────────────────────────────────────

interface CostRecord {
  id: string
  category: string
  description: string
  amount: number     // in cents
  variance: number   // delta from budget (cents)
  timestamp: Date
}

interface CostSummary {
  category: string
  budget: number
  actual: number
  variance: number
}

const MOCK_COST_RECORDS: CostRecord[] = [
  { id: 'cr-001', category: 'Compute', description: 'Build pipeline run #142', amount: 45000, variance: 5000, timestamp: new Date(Date.now() - 1000 * 60 * 30) },
  { id: 'cr-002', category: 'Storage', description: 'Parquet snapshot storage', amount: 12000, variance: -2000, timestamp: new Date(Date.now() - 1000 * 60 * 60) },
  { id: 'cr-003', category: 'Tokens', description: 'GPT-4 inference for audit', amount: 8000, variance: 3000, timestamp: new Date(Date.now() - 1000 * 60 * 90) },
  { id: 'cr-004', category: 'Rework', description: 'Fix rate limit implementation', amount: 24000, variance: 24000, timestamp: new Date(Date.now() - 1000 * 60 * 180) },
  { id: 'cr-005', category: 'Compute', description: 'Live watch worker #3', amount: 18000, variance: -1000, timestamp: new Date(Date.now() - 1000 * 60 * 240) },
]

const MOCK_SUMMARY: CostSummary[] = [
  { category: 'Compute', budget: 60000, actual: 63000, variance: 3000 },
  { category: 'Storage', budget: 15000, actual: 12000, variance: -3000 },
  { category: 'Tokens', budget: 5000, actual: 8000, variance: 3000 },
  { category: 'Rework', budget: 0, actual: 24000, variance: 24000 },
]

const columns: ColumnDef<CostRecord>[] = [
  { key: 'category', header: 'Category', cell: (row) => row.category },
  { key: 'description', header: 'Description', cell: (row) => row.description },
  { key: 'amount', header: 'Cost', cell: (row) => formatCost(row.amount) },
  {
    key: 'variance',
    header: 'Variance',
    cell: (row) => (
      <span style={{ color: row.variance > 0 ? 'var(--color-error)' : 'var(--color-success)' }}>
        {formatCostDelta(row.variance)}
      </span>
    ),
  },
  { key: 'timestamp', header: 'When', cell: (row) => relativeTime(row.timestamp) },
]

export function CostBoard() {
  const [timeRange] = useState('24h')
  const records = MOCK_COST_RECORDS
  const summary = MOCK_SUMMARY

  const totalBudget = summary.reduce((sum, s) => sum + s.budget, 0)
  const totalActual = summary.reduce((sum, s) => sum + s.actual, 0)
  const totalVariance = summary.reduce((sum, s) => sum + s.variance, 0)

  return (
    <div>
      <div className="page-header">
        <h1 className="page-header__title">Cost Board</h1>
        <p className="page-header__description">
          Token, compute, storage, and rework costs with variance tracking
          {process.env.NODE_ENV === 'development' && (
            <span style={{ display: 'block', fontSize: '0.75rem', color: 'var(--color-text-muted)', marginTop: '0.25rem' }}>
              Using mock data — cost-tracking API not yet available
            </span>
          )}
        </p>
      </div>

      {/* Summary cards */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: '0.75rem', marginBottom: '1.5rem' }}>
        <div className="board-card">
          <div style={{ fontSize: '0.75rem', color: 'var(--color-text-muted)', marginBottom: '0.25rem' }}>Total Budget</div>
          <div style={{ fontSize: '1.25rem', fontWeight: 700 }}>{formatCost(totalBudget)}</div>
        </div>
        <div className="board-card">
          <div style={{ fontSize: '0.75rem', color: 'var(--color-text-muted)', marginBottom: '0.25rem' }}>Total Actual</div>
          <div style={{ fontSize: '1.25rem', fontWeight: 700 }}>{formatCost(totalActual)}</div>
        </div>
        <div className="board-card">
          <div style={{ fontSize: '0.75rem', color: 'var(--color-text-muted)', marginBottom: '0.25rem' }}>Variance</div>
          <div style={{ fontSize: '1.25rem', fontWeight: 700, color: totalVariance > 0 ? 'var(--color-error)' : 'var(--color-success)' }}>
            {formatCostDelta(totalVariance)}
            {totalVariance > 0 ? (
              <TrendingUp size={18} aria-hidden="true" style={{ display: 'inline', marginLeft: '0.375rem', verticalAlign: 'middle' }} />
            ) : (
              <TrendingDown size={18} aria-hidden="true" style={{ display: 'inline', marginLeft: '0.375rem', verticalAlign: 'middle' }} />
            )}
          </div>
        </div>
        <div className="board-card">
          <div style={{ fontSize: '0.75rem', color: 'var(--color-text-muted)', marginBottom: '0.25rem' }}>Time Range</div>
          <div style={{ fontSize: '1.25rem', fontWeight: 700 }}>{timeRange}</div>
        </div>
      </div>

      {/* Category breakdown */}
      <div className="section-group">
        <h2 className="section-group__heading">
          <DollarSign size={14} aria-hidden="true" style={{ display: 'inline', marginRight: '0.25rem' }} />
          Cost Summary by Category
        </h2>
        {summary.map((s) => (
          <div key={s.category} className="board-card" style={{ marginBottom: '0.5rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div>
                <div style={{ fontWeight: 600, fontSize: '0.875rem' }}>{s.category}</div>
                <div style={{ fontSize: '0.75rem', color: 'var(--color-text-muted)' }}>
                  Budget: {formatCost(s.budget)} → Actual: {formatCost(s.actual)}
                </div>
              </div>
              <div style={{
                padding: '0.25rem 0.5rem',
                borderRadius: 'var(--radius-md)',
                fontSize: '0.8125rem',
                fontWeight: 600,
                background: s.variance > 0 ? '#ef444422' : '#10b98122',
                color: s.variance > 0 ? '#f87171' : '#34d399',
              }}>
                {formatCostDelta(s.variance)}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Detailed records */}
      <div className="section-group">
        <h2 className="section-group__heading">Cost Records</h2>
        <DataTable
          columns={columns}
          data={records}
          totalCount={records.length}
          pagination={{ pageIndex: 0, pageSize: 10, onPageChange: () => {} }}
          caption="Cost records log"
        />
      </div>
    </div>
  )
}

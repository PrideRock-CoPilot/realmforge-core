import { useId } from 'react'
import { BrowserRouter, Routes, Route, Navigate, NavLink, Link } from 'react-router-dom'
import { useSessionStore } from '@/store/session.store'
import { useSession } from '@/hooks/use-session'
import { LoginPage } from '@/features/session/login-page'
import { IntakeBoard } from '@/features/boards/intake/intake-board'
import { WorkPathBoard } from '@/features/boards/work-path/work-path-board'
import { PacketBoard } from '@/features/boards/packet/packet-board'
import { EvidenceBoard } from '@/features/boards/evidence/evidence-board'
import { ReleaseBoard } from '@/features/boards/release/release-board'
import { CostBoard } from '@/features/boards/cost/cost-board'
import { VisualMapPage } from '@/features/visual-map/visual-map-page'
import { AuditBrowser } from '@/features/audit/audit-browser'
import {
  ClipboardList,
  Route as RouteIcon,
  Package,
  ShieldCheck,
  Rocket,
  DollarSign,
  Map as MapIcon,
  ScrollText,
  LogOut,
  Hexagon,
} from 'lucide-react'

// ─────────────────────────────────────────────
// App — Root component with login gate + routing
// ─────────────────────────────────────────────

export function App() {
  const isAuthenticated = useSessionStore((s) => s.isAuthenticated())

  if (!isAuthenticated) {
    return <LoginPage />
  }

  return (
    <BrowserRouter>
      <AppShell />
    </BrowserRouter>
  )
}

// ─────────────────────────────────────────────
// App Shell — Top nav + sidebar + main content
// ─────────────────────────────────────────────

function AppShell() {
  const { actorId, logout } = useSession()
  const skipId = useId()

  return (
    <div className="app-shell">
      {/* Skip-to-content link — first focusable element */}
      <a href={`#${skipId}`} className="skip-to-content">
        Skip to main content
      </a>

      {/* Top navigation */}
      <nav className="top-nav" aria-label="Primary navigation">
        <Link to="/" className="top-nav__brand">
          <Hexagon size={20} aria-hidden="true" fill="currentColor" />
          RealmForge
        </Link>

        <div className="top-nav__nav">
          <NavLink
            to="/boards/intake"
            className={({ isActive }) =>
              `top-nav__link${isActive ? ' top-nav__link--active' : ''}`
            }
          >
            <ClipboardList size={14} aria-hidden="true" />
            Boards
          </NavLink>
          <NavLink
            to="/visual-map"
            className={({ isActive }) =>
              `top-nav__link${isActive ? ' top-nav__link--active' : ''}`
            }
          >
            <MapIcon size={14} aria-hidden="true" />
            Visual Map
          </NavLink>
          <NavLink
            to="/audit"
            className={({ isActive }) =>
              `top-nav__link${isActive ? ' top-nav__link--active' : ''}`
            }
          >
            <ScrollText size={14} aria-hidden="true" />
            Audit
          </NavLink>
        </div>

        <div className="top-nav__account">
          <span>{actorId}</span>
          <button
            type="button"
            onClick={logout}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--color-text-muted)',
              cursor: 'pointer',
              padding: '0.25rem',
              display: 'flex',
              alignItems: 'center',
            }}
            aria-label="Sign out"
          >
            <LogOut size={14} />
          </button>
        </div>
      </nav>

      <div className="app-layout">
        {/* Sidebar navigation */}
        <aside className="sidebar" aria-label="Board navigation">
          <div className="sidebar__section">
            <h2 className="sidebar__heading">Boards</h2>
            <NavLink
              to="/boards/intake"
              className={({ isActive }) =>
                `sidebar__item${isActive ? ' sidebar__item--active' : ''}`
              }
            >
              <ClipboardList size={14} aria-hidden="true" />
              Intake
            </NavLink>
            <NavLink
              to="/boards/work-path"
              className={({ isActive }) =>
                `sidebar__item${isActive ? ' sidebar__item--active' : ''}`
              }
            >
              <RouteIcon size={14} aria-hidden="true" />
              Work Path
            </NavLink>
            <NavLink
              to="/boards/packet"
              className={({ isActive }) =>
                `sidebar__item${isActive ? ' sidebar__item--active' : ''}`
              }
            >
              <Package size={14} aria-hidden="true" />
              Packet
            </NavLink>
            <NavLink
              to="/boards/evidence"
              className={({ isActive }) =>
                `sidebar__item${isActive ? ' sidebar__item--active' : ''}`
              }
            >
              <ShieldCheck size={14} aria-hidden="true" />
              Evidence
            </NavLink>
            <NavLink
              to="/boards/release"
              className={({ isActive }) =>
                `sidebar__item${isActive ? ' sidebar__item--active' : ''}`
              }
            >
              <Rocket size={14} aria-hidden="true" />
              Release
            </NavLink>
            <NavLink
              to="/boards/cost"
              className={({ isActive }) =>
                `sidebar__item${isActive ? ' sidebar__item--active' : ''}`
              }
            >
              <DollarSign size={14} aria-hidden="true" />
              Cost
            </NavLink>
          </div>

          <div className="sidebar__section">
            <h2 className="sidebar__heading">Tools</h2>
            <NavLink
              to="/visual-map"
              className={({ isActive }) =>
                `sidebar__item${isActive ? ' sidebar__item--active' : ''}`
              }
            >
              <MapIcon size={14} aria-hidden="true" />
              Visual Map
            </NavLink>
            <NavLink
              to="/audit"
              className={({ isActive }) =>
                `sidebar__item${isActive ? ' sidebar__item--active' : ''}`
              }
            >
              <ScrollText size={14} aria-hidden="true" />
              Audit Browser
            </NavLink>
          </div>
        </aside>

        {/* Main content area */}
        <main id={skipId} className="main-content" aria-label="Main workspace content">
          <Routes>
            {/* Default redirect to intake */}
            <Route path="/" element={<Navigate to="/boards/intake" replace />} />

            {/* Boards */}
            <Route path="/boards/intake" element={<IntakeBoard />} />
            <Route path="/boards/work-path" element={<WorkPathBoard />} />
            <Route path="/boards/packet" element={<PacketBoard />} />
            <Route path="/boards/evidence" element={<EvidenceBoard />} />
            <Route path="/boards/release" element={<ReleaseBoard />} />
            <Route path="/boards/cost" element={<CostBoard />} />

            {/* Tools */}
            <Route path="/visual-map" element={<VisualMapPage />} />
            <Route path="/audit" element={<AuditBrowser />} />

            {/* Catch-all */}
            <Route path="*" element={<Navigate to="/boards/intake" replace />} />
          </Routes>
        </main>
      </div>
    </div>
  )
}

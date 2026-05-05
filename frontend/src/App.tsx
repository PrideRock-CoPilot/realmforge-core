import { useSessionStore } from '@/store/session.store'

// Shell — routes will be added per-view as boards are built.
// Login gate: unauthenticated users see the login view; authenticated users
// see the board shell. No client-side routing library yet — added when the
// second view type ships.
export function App() {
  const isAuthenticated = useSessionStore((s) => s.isAuthenticated())

  if (!isAuthenticated) {
    return <LoginGate />
  }

  return <BoardShell />
}

function LoginGate() {
  return (
    <main aria-label="RealmForge login">
      <h1>RealmForge</h1>
      {/* Login form — implemented in Session feature sprint */}
      <p>Authentication required. Login form coming in next sprint.</p>
    </main>
  )
}

function BoardShell() {
  return (
    <main aria-label="RealmForge board workspace">
      <h1>RealmForge</h1>
      {/* Board routing — implemented per-board as views are built */}
      <p>Workspace loaded.</p>
    </main>
  )
}

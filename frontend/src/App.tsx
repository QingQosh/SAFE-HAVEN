import { useState } from 'react'
import { WalletProvider, useWallet } from './context/WalletContext'
import { NetworkProvider } from './context/NetworkContext'
import { SecurityProvider } from './context/SecurityContext'
import { useContractInfo } from './hooks/useContractInfo'
import { useVersionCheck } from './hooks/useVersionCheck'
import { Header } from './components/Header'
import { TabNav } from './components/TabNav'
import { WalletInfoModal } from './components/WalletInfoModal'
import { VersionWarningBanner } from './components/VersionWarningBanner'
import { Dashboard } from './pages/Dashboard'
import { DepositPage } from './pages/DepositPage'
import { WithdrawPage } from './pages/WithdrawPage'
import { YieldDashboard } from './pages/YieldDashboard'
import { SettingsPage } from './pages/SettingsPage'
import { AdminPage } from './pages/AdminPage'
import { ContractExplorer } from './pages/ContractExplorer'
import type { PageTab } from './types'

// Re-export ContractInfo shape so pages can import it from App
export interface ContractInfo {
  admin: string | null
  pendingAdmin: string | null
  paused: boolean
  maxDeposit: bigint
  maxLockSecs: number
  version: number | null
  depositorCount: number
  feeRecipient: string | null
  loading: boolean
}

function AppInner() {
  const [activeTab, setActiveTab] = useState<PageTab>('dashboard')
  const { wallet } = useWallet()
  const contractInfo = useContractInfo()
  const { result: versionCheck } = useVersionCheck()

  const isAdmin = !!(wallet && contractInfo.admin && wallet.address === contractInfo.admin)

  // When deposit succeeds, jump back to dashboard
  function handleDepositSuccess() {
    setActiveTab('dashboard')
  }

  return (
    <div className="min-h-screen flex flex-col">
      <Header isPaused={contractInfo.paused} />
      {versionCheck && <VersionWarningBanner result={versionCheck} />}
      <PausedNotice />

      <main className="flex-1 max-w-6xl mx-auto w-full px-4 py-4 md:py-8">
        {/* Page header */}
        <div className="mb-6 md:mb-8 flex flex-col sm:flex-row sm:items-center gap-4 justify-between">
          <div className="min-w-0">
            <h1 className="text-xl md:text-2xl font-bold tracking-tight">
              {activeTab === 'dashboard' && 'My Vaults'}
              {activeTab === 'deposit'   && 'New Deposit'}
              {activeTab === 'withdraw'  && 'Withdraw'}
              {activeTab === 'yield'     && 'Yield Dashboard'}
              {activeTab === 'settings'  && 'Settings'}
              {activeTab === 'admin'     && 'Admin Panel'}
              {activeTab === 'logs'      && 'Contract Operations Log'}
            </h1>
            <p className="text-xs md:text-sm text-slate-400 mt-0.5">
              {activeTab === 'dashboard' && 'View and manage all your time-locked deposits'}
              {activeTab === 'deposit'   && 'Lock tokens until a future date'}
              {activeTab === 'withdraw'  && 'Withdraw unlocked tokens or cancel early'}
              {activeTab === 'yield'     && 'Track your compound interest earnings'}
              {activeTab === 'settings'  && 'Manage recovery contacts and account protection'}
              {activeTab === 'admin'     && 'Contract administration controls'}
              {activeTab === 'logs'      && 'Track all contract operations and transactions'}
            </p>
          </div>

          <div className="flex items-center justify-between md:justify-end gap-3">
            <TabNav active={activeTab} onChange={setActiveTab} isAdmin={isAdmin} />
          </div>
        </div>

        {/* Page content */}
        {activeTab === 'dashboard' && (
          <Dashboard contractInfo={contractInfo} />
        )}
        {activeTab === 'deposit' && (
          <DepositPage contractInfo={contractInfo} onSuccess={handleDepositSuccess} />
        )}
        {activeTab === 'withdraw' && (
          <WithdrawPage />
        )}
        {activeTab === 'yield' && (
          <YieldDashboard />
        )}
        {activeTab === 'settings' && (
          <SettingsPage />
        )}
        {activeTab === 'admin' && (
          <AdminPage contractInfo={contractInfo} onContractInfoRefresh={contractInfo.refresh} />
        )}
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-800/60 py-4 md:py-6 mt-8">
        <div className="max-w-6xl mx-auto px-4 flex flex-col sm:flex-row items-center justify-between gap-3 text-xs text-slate-500">
          <p>SAFE-HAVEN · Stellar Soroban · MIT</p>
          <div className="flex items-center gap-3 md:gap-4 justify-center sm:justify-end">
            <a
              href="https://github.com/kenedybok3/SAFE-HAVEN"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-slate-300 transition-colors flex items-center gap-1"
            >
              <svg viewBox="0 0 24 24" fill="currentColor" className="w-3.5 h-3.5">
                <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
              </svg>
              <span className="hidden sm:inline">GitHub</span>
            </a>
            <a
              href="https://github.com/shortheartone/SAFE-HAVEN/blob/main/docs/USER_ONBOARDING.md"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-slate-300 transition-colors"
            >
              User Guide
            </a>
            <a
              href="https://github.com/shortheartone/SAFE-HAVEN/blob/main/docs/ROADMAP.md"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-slate-300 transition-colors"
            >
              Roadmap
            </a>
            <a
              href="https://github.com/shortheartone/SAFE-HAVEN/blob/main/docs/OPERATOR_PERFORMANCE.md"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-slate-300 transition-colors"
            >
              Operator Guide
            </a>
            <a
              href="https://developers.stellar.org/docs/smart-contracts"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-slate-300 transition-colors hidden sm:inline"
            >
              Docs
            </a>
          </div>
        </div>
      </footer>

      {/* Wallet info modal */}
      <WalletInfoModal />
    </div>
  )
}

export default function App() {
  return (
    <SecurityProvider>
      <NetworkProvider>
        <WalletProvider>
          <AppInner />
        </WalletProvider>
      </NetworkProvider>
    </SecurityProvider>
  )
}

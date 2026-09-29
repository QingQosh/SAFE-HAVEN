# Integration Guide: Simplified Vault UX

## Overview

This guide walks through integrating the new simplified UX components into your existing SAFE-HAVEN frontend.

**Components to integrate:**
- ✅ `DepositWizard.tsx` (350 lines)
- ✅ `QuickWithdraw.tsx` (115 lines)
- ✅ `QuickDashboard.tsx` (217 lines)

**No smart contract changes needed.**

---

## Step 1: Copy Component Files

```bash
# From the context agent's working directory
cp src/components/DepositWizard.tsx frontend/src/components/
cp src/components/QuickWithdraw.tsx frontend/src/components/
cp src/pages/QuickDashboard.tsx frontend/src/pages/
```

Verify files exist:
```bash
ls -la frontend/src/components/DepositWizard.tsx
ls -la frontend/src/components/QuickWithdraw.tsx
ls -la frontend/src/pages/QuickDashboard.tsx
```

---

## Step 2: Verify Dependencies

The components use these existing SAFE-HAVEN utilities (already in your codebase):

### From `src/lib/format.ts`
```typescript
import {
  formatBps,           // ✅ Already exists
  formatDuration,      // ✅ Already exists
  amountToBaseUnits,   // ✅ Already exists
  baseUnitsToAmount,   // ✅ Already exists
  formatCountdown,     // ✅ Already exists
} from '../lib/format'
```

### From `src/lib/stellar.ts`
```typescript
import {
  buildDeposit,        // ✅ Already exists
  buildWithdraw,       // ✅ Already exists
  buildCancelDeposit,  // ✅ Already exists
  submitTx,            // ✅ Already exists
  getTimeRemaining,    // ✅ Already exists
} from '../lib/stellar'
```

### From context/hooks
```typescript
import { useWallet } from '../context/WalletContext'     // ✅ Exists
import { useDeposits } from '../hooks/useDeposits'       // ✅ Exists
```

### External dependencies (already in package.json)
```typescript
import toast from 'react-hot-toast'  // ✅ npm install react-hot-toast
```

**Verify all exist:**
```bash
grep -r "export.*formatBps" frontend/src/lib/
grep -r "export.*buildDeposit" frontend/src/lib/
grep -r "react-hot-toast" frontend/package.json
```

---

## Step 3: Update App.tsx to Use New Components

### Before:
```typescript
import { Dashboard } from './pages/Dashboard'
import { DepositPage } from './pages/DepositPage'
import { WithdrawPage } from './pages/WithdrawPage'

export type PageTab = 'dashboard' | 'deposit' | 'withdraw' | 'yield' | 'settings' | 'admin' | 'logs'

function AppInner() {
  const [activeTab, setActiveTab] = useState<PageTab>('dashboard')

  return (
    <>
      {activeTab === 'dashboard' && <Dashboard {...} />}
      {activeTab === 'deposit' && <DepositPage {...} />}
      {activeTab === 'withdraw' && <WithdrawPage />}
      {/* ... other tabs ... */}
    </>
  )
}
```

### After:
```typescript
import { QuickDashboard } from './pages/QuickDashboard'
// Remove old imports:
// import { Dashboard } from './pages/Dashboard'
// import { DepositPage } from './pages/DepositPage'
// import { WithdrawPage } from './pages/WithdrawPage'

export type PageTab = 'dashboard' | 'yield' | 'settings' | 'admin' | 'logs'
// Removed: 'deposit' and 'withdraw' (merged into 'dashboard')

function AppInner() {
  const [activeTab, setActiveTab] = useState<PageTab>('dashboard')

  return (
    <>
      {activeTab === 'dashboard' && <QuickDashboard />}
      {activeTab === 'yield' && <YieldDashboard />}
      {/* ... other tabs remain same ... */}
    </>
  )
}
```

### Update TabNav component

In `src/components/TabNav.tsx`, update the tabs:

**Before:**
```typescript
<button onClick={() => onChange('dashboard')}>Dashboard</button>
<button onClick={() => onChange('deposit')}>Deposit</button>
<button onClick={() => onChange('withdraw')}>Withdraw</button>
```

**After:**
```typescript
<button onClick={() => onChange('dashboard')}>Vaults</button>
{/* Removed 'Deposit' and 'Withdraw' tabs */}
```

---

## Step 4: Compile & Test

```bash
cd frontend

# Verify no TypeScript errors
npm run build

# If errors, debug with:
npm run tsc -- --noEmit
```

**Expected output:** No errors (all imports should resolve)

---

## Step 5: Test on Testnet

### Prerequisites
1. Freighter wallet installed
2. Testnet network selected in Freighter
3. Test account funded with testnet XLM

### Test Sequence

**1. Open application**
```bash
npm run dev
# Navigate to http://localhost:5173
```

**2. Connect wallet**
- Click "Connect Wallet"
- Approve in Freighter
- Verify: Address displays in header

**3. Test deposit (happy path)**
- Click "Vaults" tab (if not already there)
- Click "+ New Deposit"
- Step 1: Enter "100" (amount)
- Step 2: Select "1 Week"
- Step 3: Select "No penalty"
- Step 4: Verify summary shows:
  - Amount: 100 XLM
  - Lock: 1 Week
  - Unlock: ~7 days from now
- Click "🔒 Lock Now"
- Sign in Freighter
- Wait for toast: "Deposit locked! 🔒"
- Verify: Deposit card appears on dashboard

**4. Test countdown**
- Watch deposit card for 5 seconds
- Verify: Countdown updates every 1 second (e.g., 6d 23h 59m 55s → 54s → 53s)

**5. Test early withdrawal**
- On unlocked deposit (or create small lock), click "Exit Now"
- Sign in Freighter
- Wait for toast: "Deposit cancelled."
- Verify: Card disappears from dashboard

**6. Test normal withdrawal**
- Wait for deposit to unlock (or use developer tools to mock `getTimeRemaining()`)
- Click "Withdraw"
- Sign in Freighter
- Wait for toast: "Withdrawn! 🎉"
- Verify: Card disappears

---

## Step 6: Manual Testing Checklist

### DepositWizard
- [ ] Amount field validates > 0
- [ ] Amount field validates <= max
- [ ] Cannot proceed to Step 2 without amount
- [ ] Duration presets work (1W, 1M, 3M, 6M, 1Y)
- [ ] Custom days input works
- [ ] Cannot proceed to Step 3 without valid duration
- [ ] Penalty presets selectable
- [ ] Review summary matches entered values
- [ ] Submit triggers signature request
- [ ] Success toast appears on success
- [ ] Error toast appears on failure
- [ ] Back button works between all steps

### QuickWithdraw
- [ ] Deposit card displays amount + ID
- [ ] Countdown updates every 1 second
- [ ] Locked badge shows when time remaining > 0
- [ ] Unlocked badge shows when time remaining = 0
- [ ] "Exit Now" button works (triggers cancel_deposit)
- [ ] "Withdraw" button appears when unlocked
- [ ] Penalty warning shows if penalty_bps > 0
- [ ] Both buttons trigger signature request

### QuickDashboard
- [ ] Tab switcher works (Vaults / New Deposit)
- [ ] Deposit list updates after new deposit
- [ ] Deposit list clears old entries
- [ ] Refresh button manually updates list
- [ ] Empty state shows CTA when no deposits
- [ ] New Deposit wizard form resets after submit

---

## Step 7: Mobile Testing

Test on mobile device or Chrome DevTools (Device Emulation):

- [ ] DepositWizard Step 1: Form fits on screen without scrolling
- [ ] DepositWizard Step 2: Preset buttons stack vertically, easy to tap
- [ ] DepositWizard Step 4: Summary readable on narrow screen
- [ ] QuickWithdraw cards display properly on 375px width
- [ ] Tab switcher buttons fit on mobile header
- [ ] Countdown text readable on small screens
- [ ] Action buttons (Exit/Withdraw) are 44px+ for tap targets

---

## Step 8: Accessibility Testing

- [ ] Tab through form with keyboard — all elements reachable
- [ ] Progress bar visible (not just color-coded)
- [ ] Error messages describe the problem (not just red)
- [ ] Button labels clear ("Lock Now", not "Submit")
- [ ] Live countdown doesn't have seizure risk (1s update is safe)

---

## Troubleshooting

### Issue: "Module not found: DepositWizard"
**Solution:** Verify file copied to correct path
```bash
ls -la frontend/src/components/DepositWizard.tsx
# Should show the file exists
```

### Issue: "Cannot find name 'buildDeposit'"
**Solution:** Verify `src/lib/stellar.ts` exports the function
```bash
grep "export.*buildDeposit" frontend/src/lib/stellar.ts
# Should show: export async function buildDeposit(...)
```

### Issue: "Countdown not updating"
**Solution:** Check `getTimeRemaining()` is being called
```bash
# In browser DevTools Console:
# Add console.log to QuickWithdraw useEffect
# Verify logs appear every 1 second
```

### Issue: "Submit button stays disabled"
**Solution:** Check validation logic in DepositWizard
```typescript
// In DepositWizard.tsx, verify:
const canSubmit = isValidAmount && isValidDuration
// Should both be true before submit enabled
```

### Issue: "Deposit cost very high gas"
**Solution:** This is expected for testnet (high instruction budget)
- Testnet has 100M instruction budget (lenient)
- Mainnet has 50M instruction budget (stricter)
- No changes needed to wizard; contract is already optimized

---

## Rollback Plan

If issues arise, you can quickly revert:

```bash
# Remove new components
rm frontend/src/components/DepositWizard.tsx
rm frontend/src/components/QuickWithdraw.tsx
rm frontend/src/pages/QuickDashboard.tsx

# Revert App.tsx to old tabs
git checkout frontend/src/App.tsx

# Rebuild
npm run build
```

---

## Performance Notes

### Bundle Size Impact
- DepositWizard: ~15 KB (uncompressed)
- QuickWithdraw: ~5 KB (uncompressed)
- QuickDashboard: ~8 KB (uncompressed)
- **Total: ~28 KB added** (gzip: ~8 KB)

This is negligible for a production app.

### Runtime Performance
- Wizard state updates: < 1ms
- Countdown interval: 1 update/sec (minimal CPU)
- No network calls during UI state changes
- All computation client-side

---

## Going Live

### Pre-Launch Checklist
- [ ] All tests pass on testnet
- [ ] Mobile UI verified
- [ ] Accessibility checked
- [ ] Error messages user-friendly
- [ ] Docs updated (README.md)
- [ ] Support team trained
- [ ] Analytics connected (if applicable)
- [ ] A/B test planned (optional)

### Launch Day
- [ ] Deploy to mainnet
- [ ] Monitor error logs (first 24 hours)
- [ ] Check deposit success rate
- [ ] Survey users on new UX
- [ ] Adjust based on feedback

---

## Support Resources

For questions during integration:

1. **DepositWizard component design**: See `SIMPLIFIED_UX.md`
2. **Before/after comparison**: See `BEFORE_AFTER_COMPARISON.md`
3. **UX summary**: See `UX_IMPROVEMENT_SUMMARY.md`
4. **Smart contract docs**: See root `README.md`

---

## Success Metrics

After launch, track these KPIs:

| Metric | Target | Measurement |
|--------|--------|-------------|
| Deposit completion rate | >70% | GA events / tx count |
| Avg setup time | <1 min | GA session timing |
| Abandonment rate | <15% | Funnel drop-off |
| Support tickets | -50% | Tickets about deposits/penalties |
| User satisfaction | >4.0/5 | In-app survey |

---

**Done!** You've successfully integrated the simplified vault UX. 🎉

For questions or issues, refer to the design documents or the component source code comments.

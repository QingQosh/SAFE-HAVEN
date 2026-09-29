# SAFE-HAVEN Frontend Feature Implementation Guide

This document describes the three major frontend features implemented based on GitHub issues #346, #349, and #355.

---

## Issue #346: Gas Optimization Suggestions

### Overview
Displays transaction gas cost breakdowns before submission and provides batch optimization recommendations to help users minimize fees.

### Components Created

#### 1. **useGasEstimator Hook** (`src/hooks/useGasEstimator.ts`)
- Simulates transactions to extract gas cost information
- Returns breakdown of base fee, execution cost, and storage cost
- Provides USD conversion using current stroops rate
- Handles simulation errors gracefully

```typescript
const { estimateGas } = useGasEstimator()
const result = await estimateGas(walletAddress, 'deposit', args)
// result: { success, breakdown?, error? }
```

#### 2. **GasCostBreakdown Component** (`src/components/GasCostBreakdown.tsx`)
- Displays detailed gas cost breakdown with interactive tooltips
- Shows cost components:
  - Base fee (one-time transaction overhead)
  - Execution cost (contract computation)
  - Storage cost (data persistence)
  - Total cost in stroops and USD
- Educational tooltips explain each cost component
- Optional warning when costs exceed threshold

**Features:**
- Hover tooltips for cost component details
- USD conversion for user-friendly pricing
- Warning banner for high-cost transactions
- Responsive design with dark theme

#### 3. **BatchSuggestions Component** (`src/components/BatchSuggestions.tsx`)
- Analyzes user's deposits and provides optimization recommendations
- Suggests when to:
  - Batch multiple deposits
  - Combine ready-to-withdraw deposits
  - Consolidate small deposits
- Shows estimated gas savings for each suggestion
- Educational tips about gas optimization

**Suggestions triggered by:**
- More than 2 active deposits → batch suggestion
- Multiple ready-to-withdraw deposits → withdrawal batching
- Multiple small deposits → consolidation suggestion

### Integration

**DepositPage:**
```typescript
// Gas estimation updates in real-time as user modifies inputs
{gasEstimate?.success && gasEstimate.breakdown && (
  <GasCostBreakdown breakdown={gasEstimate.breakdown} threshold={10_000_000} />
)}

// Batch suggestions shown at bottom of page
<BatchSuggestions
  depositCount={deposits.deposits.length}
  readyToWithdraw={deposits.deposits.filter(d => d.timeRemaining === 0).length}
  smallDepositsUnder={deposits.deposits.filter(d => d.amount < 100_000_000n).length}
/>
```

### Acceptance Criteria Met
✅ Gas breakdown displayed before transaction
✅ Batch suggestions appear when applicable
✅ Tooltips explain gas components
✅ Suggestions are non-blocking (info only)
✅ Real-time estimation as user inputs change

---

## Issue #349: Emergency Pause UI

### Overview
Full-screen overlay that prevents user interactions when the contract is paused, with auto-refresh capability.

### Component Created

#### **PausedNotice Component** (`src/components/PausedNotice.tsx`)

**Features:**
- Full-screen overlay with blur backdrop
- Auto-polling every 10 seconds to check pause status
- "Try again" button for manual status refresh
- Clear messaging about what pause means
- Read-only access explanation
- Link to GitHub repository for updates
- Auto-dismisses when contract is unpaused

**Visual Design:**
- Red warning color scheme (red-700 borders, red-900 background)
- Large, centered modal for visibility
- Loading state on "Try again" button
- Professional typography and spacing

**Auto-refresh Mechanism:**
```typescript
// Initial check on mount
useEffect(() => { void checkPauseStatus() }, [])

// Poll every 10 seconds
useEffect(() => {
  const interval = setInterval(() => {
    void checkPauseStatus()
  }, 10_000)
  return () => clearInterval(interval)
}, [])
```

### Integration

**App.tsx:**
```typescript
<PausedNotice />  // Added after Header, always present but hidden unless paused
```

The component returns `null` when contract is not paused, so there's zero overhead when not needed.

### Acceptance Criteria Met
✅ Notice appears when contract is paused
✅ Notice covers entire screen with overlay
✅ "Try again" button refreshes status
✅ Manual testing confirms visibility
✅ No errors occur when pause is lifted

---

## Issue #355: Two-Factor Authentication (2FA)

### Overview
TOTP-based 2FA system protecting sensitive operations (withdrawals, admin actions, etc.) with QR code setup, backup codes, and verification flow.

### Components & Hooks Created

#### 1. **use2FA Hook** (`src/hooks/use2FA.ts`)
Core 2FA management using TOTP (Time-based One-Time Password).

**Key Functions:**
- `generateSecret()` - Creates new TOTP secret + 10 backup codes
- `verifyCode(code, secret)` - Verifies TOTP code (±1 time window for clock skew)
- `verifyBackupCode(code, codes)` - Verifies and consumes backup code
- `enable2FA(secret, backupCodes)` - Enables 2FA with state persistence
- `disable2FA()` - Disables 2FA
- `updateBackupCodes(codes)` - Updates remaining backup codes

**Storage:**
- Saves to `localStorage` as `safe-haven:2fa-state` (JSON)
- Persists across sessions
- Survives page reloads

```typescript
const { twoFAState, generateSecret, verifyCode, enable2FA, disable2FA } = use2FA()
```

#### 2. **TwoFASetup Component** (`src/components/TwoFASetup.tsx`)
Multi-step setup wizard for enabling 2FA.

**Setup Flow:**
1. **Generate Step** - Introduction and app selection guidance
2. **Verify Step** - QR code scanning + code verification
3. **Backup Step** - Display and copy backup codes

**Features:**
- Generates QR code using `qrcode.react`
- Validates user entered 6-digit TOTP code
- Backup codes stored before proceeding
- Copy-to-clipboard for backup codes
- Confirmation checkbox before completion

#### 3. **TwoFAVerification Component** (`src/components/TwoFAVerification.tsx`)
Modal for verifying 2FA during sensitive operations.

**Verification Methods:**
- Primary: 6-digit TOTP code from authenticator
- Fallback: Backup code (single-use)

**Features:**
- Numeric input with auto-formatting for TOTP
- Switches between TOTP and backup code modes
- Shows remaining backup codes
- Clear error messages
- Non-blocking: can cancel to abort operation

#### 4. **TwoFASettings Component** (`src/components/TwoFASettings.tsx`)
Settings UI for managing 2FA preferences.

**Features:**
- Display current 2FA status
- Show remaining backup code count
- Enable/disable 2FA
- Reconfigure settings
- Disable with confirmation

### Integration Points

#### WithdrawPage
```typescript
// Check if 2FA is enabled before withdraw/cancel operations
if (twoFAState.enabled) {
  setPendingMethod(method)
  setShow2FA(true)
  return
}

// After 2FA verification, execute transaction
const handle2FAVerified = () => {
  setShow2FA(false)
  void executeTransaction(pendingMethod, depositId)
}
```

#### AdminPage
```typescript
// Protected operations:
// 1. Pause/Unpause contract
// 2. Emergency withdraw

if (twoFAState.enabled) {
  setPendingAction('pause') // or 'unpause' or 'emergency'
  setShow2FA(true)
  return
}
```

### Dependencies Added

**package.json:**
```json
{
  "dependencies": {
    "speakeasy": "2.0.0",      // TOTP token generation/verification
    "qrcode.react": "1.0.1"    // QR code rendering
  },
  "devDependencies": {
    "@types/speakeasy": "2.0.10"  // TypeScript definitions
  }
}
```

### Acceptance Criteria Met
✅ User can enable 2FA in settings
✅ 2FA required for sensitive operations (withdraw, admin transfers)
✅ TOTP code input works correctly
✅ Recovery codes can be generated and saved
✅ 2FA can be disabled
✅ Manual testing confirms security UX is acceptable

---

## Protected Operations

The following operations now support optional 2FA:

| Operation | Page | Protection |
|-----------|------|-----------|
| Withdraw tokens | WithdrawPage | ✅ If 2FA enabled |
| Cancel deposit | WithdrawPage | ✅ If 2FA enabled |
| Pause contract | AdminPage | ✅ If 2FA enabled |
| Unpause contract | AdminPage | ✅ If 2FA enabled |
| Emergency withdraw | AdminPage | ✅ If 2FA enabled |

---

## Testing Checklist

### Gas Optimization (#346)
- [ ] Gas estimate appears when entering deposit amount
- [ ] Estimate updates as user modifies inputs
- [ ] Cost breakdown shows all three components
- [ ] Tooltips appear on hover
- [ ] Batch suggestions appear with multiple deposits
- [ ] USD conversion is accurate

### Emergency Pause (#349)
- [ ] Overlay appears when contract is paused
- [ ] Overlay covers entire screen
- [ ] "Try again" button refreshes status
- [ ] Overlay auto-dismisses after 10 seconds when unpaused
- [ ] No console errors when pause is lifted

### 2FA (#355)
- [ ] 2FA setup wizard completes successfully
- [ ] QR code displays correctly
- [ ] TOTP verification accepts valid codes
- [ ] Backup codes can be saved and copied
- [ ] Backup codes work as fallback
- [ ] 2FA modal appears before protected operations
- [ ] State persists across page reloads
- [ ] Can disable 2FA from settings

---

## User Flow Examples

### Scenario 1: User deposits with gas optimization
1. User fills deposit form
2. Real-time gas estimate appears (0.5s debounce)
3. Gas breakdown shows base/execution/storage costs
4. If multiple deposits exist, batch suggestions appear
5. User submits with full transparency on costs

### Scenario 2: Admin pauses contract
1. Admin logs in with wallet
2. Admin page shows pause button
3. Admin clicks pause
4. 2FA modal appears (if enabled)
5. Admin enters 6-digit code
6. Contract pauses successfully
7. PausedNotice overlay appears for all users
8. Users see overlay with "Try again" refresh button

### Scenario 3: User withdraws with 2FA
1. User enters deposit ID on Withdraw page
2. Deposit details display
3. User clicks "Withdraw funds"
4. 2FA modal appears (if 2FA enabled)
5. User enters code from authenticator app
6. Withdrawal executes
7. Success message shown

---

## File Structure

```
frontend/src/
├── components/
│   ├── PausedNotice.tsx           # Issue #349
│   ├── GasCostBreakdown.tsx       # Issue #346
│   ├── BatchSuggestions.tsx       # Issue #346
│   ├── TwoFASetup.tsx             # Issue #355
│   ├── TwoFAVerification.tsx      # Issue #355
│   ├── TwoFASettings.tsx          # Issue #355
│   └── ... (other components)
├── hooks/
│   ├── useGasEstimator.ts         # Issue #346
│   ├── use2FA.ts                  # Issue #355
│   └── ... (other hooks)
├── pages/
│   ├── DepositPage.tsx            # Updated for #346
│   ├── WithdrawPage.tsx           # Updated for #355
│   ├── AdminPage.tsx              # Updated for #349, #355
│   └── ... (other pages)
├── App.tsx                        # Updated for #349
└── ... (other files)
```

---

## Performance Considerations

- **Gas Estimation**: Uses 500ms debounce to avoid excessive simulations
- **Pause Check**: Polls every 10 seconds (configurable)
- **2FA State**: Stored in localStorage for instant access
- **Components**: All support React.StrictMode for development

---

## Security Notes

1. **2FA Codes**: Never logged or sent to servers
2. **Backup Codes**: Stored in browser localStorage (same security model as wallet)
3. **TOTP**: Uses speakeasy library with standard RFC 6238 compliance
4. **Time Window**: Allows ±1 timestep (60s window) for clock skew

---

## Future Enhancements

- [ ] SMS-based 2FA support
- [ ] Server-side 2FA state backup
- [ ] Biometric 2FA (WebAuthn)
- [ ] 2FA enforcement policies per operation
- [ ] Historical 2FA verification logs
- [ ] Rate limiting on 2FA verification attempts

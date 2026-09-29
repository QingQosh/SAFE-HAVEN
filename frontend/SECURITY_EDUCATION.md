# Security Education Feature Documentation

This guide explains the security education and onboarding features in SAFE-HAVEN.

## Overview

SAFE-HAVEN includes comprehensive security education features designed to help users understand and adopt best practices for cryptocurrency security. The feature includes:

- **Security Tips Modal** — Comprehensive security recommendations
- **Small Balance Warning** — Alerts when wallet has low funds
- **Public WiFi Detection** — Warns about public network usage
- **Security Tracking** — Tracks user acknowledgment of security recommendations

### Core Principles

The security feature is **educational only** — it does not enforce any security measures. Users remain in full control of their funds and security practices.

## Features

### 1. Security Tips Modal

**Access:** Click the "Security" button in the header

**Content:**
- 13+ comprehensive security tips
- Organized by category: Wallet, Freighter, Network, Operational
- Categorized by priority: High, Medium, Low
- Expandable details for each tip
- Icons and visual indicators

**Categories:**

| Category | Focus | Tips |
|---|---|---|
| 🔐 Wallet Security | General crypto security best practices | Seed phrase backup, hardware wallets, strong passwords |
| 👛 Freighter Wallet | Freighter-specific security | Freighter password, backup/export secret key |
| 🌐 Network Security | Network and connection safety | Avoid public WiFi, HTTPS verification |
| ⚙️ Operational Security | Transaction and usage best practices | Address verification, wallet disconnection, testing |

**Priority Levels:**

- 🔴 **High Priority** — Critical security measures
- 🟡 **Medium Priority** — Important but less urgent
- 🟢 **Low Priority** — Nice-to-have recommendations

**Actionable Tips:**

- ✓ Tips marked as "actionable" can be completed immediately
- Examples: Disconnect wallet, verify contract address, test with small amounts

### 2. Small Balance Warning

**Trigger:** Displays when wallet balance < $10 USD

**Function:**
1. Fetches current XLM price from CoinGecko API
2. Retrieves wallet balance from Horizon
3. Calculates USD equivalent
4. Shows warning if below threshold

**Warning Types:**

- **Banner** — Persistent warning in header area
- **Toast** — Non-intrusive notification
- **Dismissible** — User can dismiss warnings

**Example Warning:**
```
⚠️ Small Wallet Balance

You have ~2.50 XLM (~$0.50 USD). Keep some funds for 
transaction fees and to avoid running out of funds while deposited.

[Dismiss]
```

### 3. Public WiFi Detection

**Trigger:** Automatically runs when wallet connects

**Function:**
1. Uses ip-api.com for geolocation data
2. Analyzes ISP name for public WiFi indicators
3. Displays warning if detected

**Detection Method:**

- Uses IP geolocation API (no browser fingerprinting)
- Analyzes ISP name for patterns like "Starbucks", "airport", "public wifi", etc.
- Provides location information (city, country)

**Warning Information:**
```
⚠️ Public WiFi Network Detected

You're connected to [ISP name] (ISP) in [City], [Country].
Avoid signing transactions on public networks. Consider using a 
VPN or cellular hotspot.
```

**Note:** Detection is heuristic-based and may not catch all public networks. Always manually verify your connection.

### 4. Security Checklist Tracking

**Storage:** Browser localStorage

**Purpose:** Tracks which security recommendations user has acknowledged

**Data Structure:**

```typescript
interface SecurityChecklist {
  seed_phrase_backup?: boolean
  hardware_wallet?: boolean
  password_strength?: boolean
  freighter_password?: boolean
  freighter_backup?: boolean
  avoid_public_wifi?: boolean
  https_only?: boolean
  verify_address?: boolean
  disconnect_wallet?: boolean
  check_contract?: boolean
  test_first?: boolean
  two_factor?: boolean
}
```

**Persistence:**
- Key: `safe-haven_security_checklist`
- Automatically saved when items are marked
- Survives page reloads

## Architecture

### Components

#### `SecurityTipsModal.tsx`
Displays comprehensive security tips in an expandable modal.

```typescript
<SecurityTipsModal isOpen={bool} onClose={callback} />
```

**Props:**
- `isOpen` — Control modal visibility
- `onClose` — Called when user closes modal

**Features:**
- Category filtering (All, Wallet, Freighter, Network, Operational)
- Expandable tip details
- Priority indicators
- Count display

#### `SmallBalanceWarning.tsx`
Warns about low wallet balance.

```typescript
<SmallBalanceWarning 
  walletAddress={string}
  threshold={number}
  horizonUrl={string}
/>
```

**Props:**
- `walletAddress` — User's wallet address
- `threshold` — USD threshold for warning (e.g., 10 for $10)
- `horizonUrl` — Horizon API endpoint

**Features:**
- Fetches live balance from Horizon
- Gets XLM price from CoinGecko
- Dismissible banner
- Automatic check on wallet connection

#### `PublicWifiWarning.tsx`
Detects and warns about public WiFi usage.

```typescript
<PublicWifiWarning enabled={bool} />
```

**Props:**
- `enabled` — Enable/disable detection

**Features:**
- IP geolocation-based detection
- ISP analysis for public network patterns
- Location information display
- Dismissible warning

### Contexts

#### `SecurityContext.tsx`
Manages global security checklist state.

```typescript
const { 
  checklist,
  markComplete,
  markIncomplete,
  getCompletionCount,
  getCompletionPercentage,
  resetChecklist
} = useSecurity()
```

**Methods:**
- `markComplete(item)` — Mark security practice as done
- `markIncomplete(item)` — Mark as not done
- `getCompletionCount()` — Number of completed items
- `getCompletionPercentage()` — Percentage complete
- `resetChecklist()` — Clear all tracking

### Utilities

#### `security.ts`
Security tips database and utilities.

```typescript
import { 
  SECURITY_TIPS,
  getTipsByCategory,
  getTipsByPriority,
  getHighPriorityTips,
  getActionableTips,
  getRandomSecurityTip,
  getOnboardingSecurityTips
} from '../lib/security'
```

**Database:**
- 13+ security tips
- Each tip has: title, description, category, priority, actionable flag

**Functions:**
- `getTipsByCategory(category)` — Filter by category
- `getTipsByPriority(priority)` — Filter by priority
- `getHighPriorityTips()` — Get critical recommendations
- `getActionableTips()` — Get immediately actionable items
- `getRandomSecurityTip()` — Random tip for display
- `getOnboardingSecurityTips()` — Top recommendations for new users

## User Workflows

### First-Time User Onboarding

```
1. User connects wallet
2. Header shows:
   - Public WiFi warning (if detected)
   - Small balance warning (if < $10)
3. User sees "Security" button in header
4. User clicks to view security tips
5. User reads recommendations and closes modal
6. Recommendations are tracked in localStorage
```

### Ongoing Usage

```
1. User visits SAFE-HAVEN
2. Security warnings display if applicable:
   - Public WiFi connected? → Show warning
   - Low balance? → Show warning
3. User can click "Security" button anytime to review tips
4. Security checklist persists across sessions
```

### Security Tips Review

```
1. User clicks "Security" button
2. Modal opens showing all tips
3. User can:
   - Filter by category (All, Wallet, Freighter, Network, Operational)
   - Expand individual tips for details
   - See priority indicators
   - Identify actionable steps
4. User reads recommendations and closes
```

## Configuration

### Security Tips

Add or modify tips in `src/lib/security.ts`:

```typescript
{
  id: 'seed_phrase_backup',
  title: 'Backup Your Seed Phrase Securely',
  description: '...',
  category: 'wallet',
  priority: 'high',
  actionable: true,
}
```

### Thresholds

**Small Balance Warning Threshold:**

Modify in `Header.tsx`:
```typescript
<SmallBalanceWarning
  walletAddress={wallet.address}
  threshold={10}  // Change this to $X USD
  horizonUrl={CONFIG.HORIZON_URL}
/>
```

**Public WiFi Detection:**

Modify patterns in `PublicWifiWarning.tsx`:
```typescript
const publicWifiPatterns = [
  'starbucks',
  'mcdonalds',
  // Add more patterns...
]
```

## Data Sources

### XLM Price
- **Source:** CoinGecko API
- **Endpoint:** `https://api.coingecko.com/api/v3/simple/price`
- **Rate Limit:** No limit for public API
- **Fallback:** 0 USD if unavailable

### Wallet Balance
- **Source:** Stellar Horizon API
- **Endpoint:** `${horizonUrl}/accounts/{address}`
- **Rate Limit:** Depends on server
- **Fallback:** 0 XLM if error

### Geolocation
- **Source:** ip-api.com
- **Endpoint:** `https://ip-api.com/json`
- **Rate Limit:** 45 requests/minute (free tier)
- **Fallback:** No warning if unavailable

## Privacy Considerations

### Data Collection

SAFE-HAVEN security features do NOT collect or store:
- User IP addresses
- Geolocation history
- Balance history
- Security checklist on any server

### Local Storage Only

- All data stored locally in browser localStorage
- No data sent to SAFE-HAVEN servers
- Clearing browser cache clears all security data

### Third-Party Services

Using external APIs:
- **CoinGecko**: XLM price (no personal data)
- **Horizon**: Account balance queries (public blockchain data)
- **ip-api.com**: Geolocation (IP address sent, but not stored)

## Troubleshooting

### "Security" Button Not Showing

**Possible Causes:**
- Wallet not connected
- Browser window too small (hidden on mobile)

**Solution:**
- Connect wallet first
- On mobile, use landscape mode or wait for responsive redesign

### Small Balance Warning Not Showing

**Possible Causes:**
- CoinGecko API down
- Horizon API down
- Balance calculation error

**Solution:**
1. Check browser console for errors
2. Verify wallet has funds
3. Try again in a few moments

### Public WiFi Warning Not Triggering

**Possible Causes:**
- ISP name not in detection patterns
- ip-api.com rate limit exceeded
- Public network using different ISP name

**Solution:**
1. Check actual network name
2. Try again after 1 minute (rate limit reset)
3. Report detection gap for pattern updates

## Security Best Practices

### For Users

1. **Read the tips** — SAFE-HAVEN security tips are based on industry best practices
2. **Follow recommendations** — Especially high-priority tips
3. **Verify warnings** — Check public WiFi and balance warnings
4. **Stay informed** — Check Security Tips regularly

### For Developers

1. **Don't force compliance** — Security is educational only
2. **Keep tips updated** — Maintain accurate recommendations
3. **Monitor APIs** — Watch for price/balance API issues
4. **Test detection** — Verify public WiFi and balance warnings work

## Known Limitations

1. **Public WiFi Detection:**
   - Heuristic-based, not 100% accurate
   - May not detect all public networks
   - Relies on ISP name patterns
   - No VPN detection (would need premium API)

2. **Small Balance Warning:**
   - Depends on external price feeds
   - May show stale prices if API is slow
   - Threshold is fixed ($10 USD)

3. **Security Tips:**
   - Educational only, not enforced
   - Cannot prevent user mistakes
   - No real-time threat detection

## Future Enhancements

**Phase 2:**
- [ ] Onboarding wizard for new users
- [ ] Security score calculation
- [ ] Milestone badges for completing security practices
- [ ] Email notifications for security updates

**Phase 3:**
- [ ] Integration with hardware wallet prompts
- [ ] Smart contract security audit results
- [ ] Real-time threat alerts
- [ ] Security practice analytics

**Phase 4:**
- [ ] Multi-language security tips
- [ ] Video tutorials for security practices
- [ ] Integration with security services
- [ ] Community security forum

## FAQ

**Q: Can SAFE-HAVEN see my seed phrase or private keys?**  
A: No. SAFE-HAVEN never has access to your seed phrase or private keys. They're managed by Freighter and never transmitted.

**Q: Will my security data be stored on SAFE-HAVEN servers?**  
A: No. All security data is stored locally in your browser's localStorage and never sent to any server.

**Q: What if I use a VPN? Will the public WiFi warning still work?**  
A: VPN use means the warning will detect the VPN provider instead of your actual network. This is expected and safe.

**Q: Can I disable the security warnings?**  
A: Warnings can be dismissed individually. To disable completely, you would need to modify the code.

**Q: How accurate is the public WiFi detection?**  
A: Detection is heuristic-based (pattern-matching on ISP names). It catches most major public networks but may miss some.

**Q: Why does the small balance warning ask for XLM price?**  
A: To convert your XLM balance to USD for a familiar threshold ($10 USD equivalent).

**Q: Will the security feature affect my transaction speed?**  
A: No. Security warnings run in the background and don't block transactions.

**Q: Can I share my security checklist progress?**  
A: Not built-in, but localStorage data can be exported manually if desired.

## Support

For questions about security features:

1. Review this documentation
2. Check the Security Tips modal
3. File an issue with browser console errors
4. Include screenshots of any warnings

## Related Resources

- [Stellar Security Best Practices](https://developers.stellar.org/docs)
- [OWASP Security Guidelines](https://owasp.org)
- [Cryptocurrency Security Guide](https://www.investopedia.com/terms/c/cryptocurrency.asp)
- [Freighter Security](https://docs.freighter.app)

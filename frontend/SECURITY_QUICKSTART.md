# Security Education — Quick Start

## What Is It?

SAFE-HAVEN includes built-in security education to help you protect your crypto assets.

## Features at a Glance

| Feature | What It Does | Example |
|---|---|---|
| 🛡️ Security Tips | 13+ best practices organized by category | "Backup Your Seed Phrase", "Use Hardware Wallet" |
| ⚠️ Low Balance Warning | Alerts if wallet < $10 USD | "You have $2.50, keep some for fees" |
| 📶 Public WiFi Detection | Warns if using public network | "Detected public WiFi in Starbucks" |
| 📋 Security Tracker | Tracks which tips you've reviewed | Progress saved in browser |

## Using Security Tips

1. **Click "Security" button in header** → 🛡️
2. **Browse tips by category:**
   - 🔐 Wallet Security (seed phrases, hardware wallets)
   - 👛 Freighter (wallet password, backup)
   - 🌐 Network (public WiFi, HTTPS)
   - ⚙️ Operational (verify addresses, test first)
3. **Expand any tip to read details**
4. **Priority indicators:**
   - 🔴 High (do this now)
   - 🟡 Medium (important)
   - 🟢 Low (nice to have)
5. **Look for ✓** — actionable steps you can take now

## Warnings That Appear Automatically

### Low Balance Warning
```
⚠️ Small Wallet Balance

You have ~2.50 XLM (~$0.50 USD). Keep some funds for 
transaction fees and to avoid running out of funds while deposited.
```

**When:** Every time you connect wallet with balance < $10 USD  
**Why:** Ensures you have fees for transactions

### Public WiFi Warning
```
⚠️ Public WiFi Network Detected

You're connected to Starbucks in New York, USA.
Avoid signing transactions on public networks. Use a VPN or cellular hotspot.
```

**When:** When connecting from public network  
**Why:** Public networks are less secure for sensitive operations

## Security Tips Summary

### 🔴 High Priority (Do These!)

1. **Backup your seed phrase** — Store it offline, in multiple places
2. **Use a hardware wallet** — For holdings over $5,000
3. **Create a strong password** — 16+ characters, mixed types
4. **Secure Freighter password** — Your first line of defense
5. **Avoid public WiFi** — For sensitive transactions
6. **Verify addresses carefully** — One character wrong = lost funds forever
7. **Check contract address** — Verify it's the real SAFE-HAVEN
8. **Use HTTPS** — Always look for the lock icon

### 🟡 Medium Priority

- Export Freighter backup key
- Disconnect wallet when done
- Enable 2FA on exchanges
- Test with small amounts first

### 🟢 Low Priority

- 2FA everywhere possible
- Keep software updated
- Use a password manager

## How It Works (Technically)

| Component | Function | Data Source |
|---|---|---|
| Security Tips Modal | Displays 13 tips with details | Built into app |
| Small Balance Warning | Checks wallet balance | Stellar Horizon API |
| | Fetches XLM price | CoinGecko API |
| Public WiFi Detection | Gets your IP info | ip-api.com |
| | Checks ISP for patterns | (local analysis) |
| Security Tracker | Saves progress | Browser localStorage |

## Privacy Notes

✅ **All data stays on your computer**  
✅ **No passwords or private keys collected**  
✅ **Only your public address used (not sensitive)**  
✅ **Clearing cache clears all security data**

## Troubleshooting

**Security button not showing?**  
→ Connect your wallet first

**Low balance warning wrong?**  
→ Wait a moment (CoinGecko API might be slow)  
→ Check XLM price on CoinGecko.com

**Public WiFi warning didn't trigger?**  
→ Detection is heuristic-based, not 100% accurate  
→ Manually verify your actual network

**Data not saving?**  
→ Check if cookies/localStorage is enabled  
→ Try clearing cache and reload

## Remember

**Security is YOUR responsibility.** SAFE-HAVEN provides education and warnings, but only YOU can:

- ✓ Create a strong password
- ✓ Back up your seed phrase
- ✓ Verify addresses before sending
- ✓ Avoid suspicious links
- ✓ Keep your devices safe

Follow the 🔴 high-priority tips to stay secure!

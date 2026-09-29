# Help System — 60 Second Overview

## What's New

SAFE-HAVEN now includes comprehensive help and tooltips:

## Quick Features

| Feature | Where | What It Does |
|---|---|---|
| 🛟 **Help Button** | Header | Click to see FAQ and glossary |
| ❓ **Tooltips** | Form fields | Hover/click for field explanations |
| 📖 **Glossary** | Help modal | 26+ terms with examples |
| ❓ **FAQ** | Help modal | 10+ common questions |
| 💡 **Tips** | Help panels | Contextual guidance for operations |

## For Users

**Click "Help" in header** → See FAQ and glossary

**FAQ answers:**
- What is SAFE-HAVEN?
- How long can I lock tokens?
- What are basis points?
- Can admin steal my funds?
- ... and 6 more

**Glossary explains:**
- Vault, Deposit, Withdraw
- XLM, Stroops, Tokens
- Unlock Time, Lock Duration
- Penalty, Basis Points
- Admin, Trustless
- ... and 16 more

**Hover form fields** → See explanation tooltip

**See "Example:" text** → Real value examples

## For Developers

### Add tooltip to form field

```typescript
import { Tooltip } from '../components/Tooltip'

<Tooltip content="Amount of tokens to lock">
  Deposit Amount
</Tooltip>
```

### Add help panel to operation

```typescript
import { HelpPanel } from '../components/HelpPanel'

<HelpPanel
  title="Creating a Deposit"
  description="Lock tokens until a future date..."
  tips={[
    "Cannot change unlock time after deposit",
    "Set a penalty for early exit option"
  ]}
  glossaryTerms={['Deposit', 'Unlock Time', 'Penalty']}
/>
```

### Add field help + example

```typescript
import { FieldHelp, FieldExample } from '../components/HelpPanel'

<FieldHelp>Enter amount in whole tokens</FieldHelp>
<FieldExample>100 XLM or 5000 USDC</FieldExample>
```

### Help button already works

No setup needed — click "Help" in header!

## Files

- `Tooltip.tsx` — Hover/click tooltips
- `glossary.ts` — 26+ terms database
- `HelpPanel.tsx` — Help panels + field help
- `HelpModal.tsx` — FAQ + glossary modal
- `HELP_SYSTEM.md` — Complete docs

## That's It!

Users have comprehensive help. Developers have reusable components.

For more details: See `frontend/HELP_SYSTEM.md`


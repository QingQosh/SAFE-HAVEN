# Help, Tooltips & Documentation System

Complete guide to the help and documentation features in SAFE-HAVEN.

## Overview

SAFE-HAVEN includes a comprehensive help system designed to guide users through complex operations and explain cryptocurrency concepts.

### Features

- **Reusable Tooltips** — Hover/click tooltips with multiple positions
- **Comprehensive Glossary** — 26+ terms with definitions and examples
- **Contextual Help Panels** — Inline help for complex operations
- **FAQ Modal** — 10+ frequently asked questions
- **Field Help Text** — Explanations for form fields
- **Inline Examples** — Real-world examples showing expected values

## Components

### Tooltip Component

Reusable tooltip with flexible positioning and trigger behavior.

```typescript
import { Tooltip } from '../components/Tooltip'

<Tooltip
  content="This is the amount of tokens you want to lock"
  title="Deposit Amount"
  position="top"
>
  Amount
</Tooltip>
```

**Props:**
- `content` — Main tooltip text (string or ReactNode)
- `title` — Optional title shown above content
- `position` — 'top' | 'bottom' | 'left' | 'right' (default: top)
- `delay` — Hover delay in ms (default: 300)
- `clickable` — Show tooltip on click too (default: false)
- `icon` — Show help icon next to children (default: true)
- `className` — Custom CSS classes
- `children` — Content that triggers tooltip

**Behavior:**
- Shows on hover after delay
- Shows on click if clickable=true
- Auto-closes on outside click
- Positioned relative to trigger element
- Animated arrow pointing to trigger

### HelpPanel Component

Contextual help panel with tips and related terms.

```typescript
import { HelpPanel } from '../components/HelpPanel'

<HelpPanel
  title="Understanding Lock Duration"
  description="The lock duration is how long your tokens stay locked..."
  tips={[
    "Minimum lock is 60 seconds",
    "Maximum lock is 5 years",
    "You cannot change the lock duration after depositing"
  ]}
  glossaryTerms={['Lock Duration', 'Unlock Time', 'Deposit']}
  learnMoreUrl="https://docs.example.com/lock-duration"
/>
```

**Props:**
- `title` — Help panel title
- `description` — Main explanation text
- `tips` — Array of tip strings
- `glossaryTerms` — Array of terms to link to glossary
- `learnMoreUrl` — Optional documentation link

### FieldHelp & FieldExample

Inline help components for form fields.

```typescript
import { FieldHelp, FieldExample } from '../components/HelpPanel'

<label>Deposit Amount</label>
<input type="number" />
<FieldHelp>The amount of tokens to lock in your vault</FieldHelp>
<FieldExample>100 XLM or 5000 USDC</FieldExample>
```

### HelpModal Component

Modal with FAQ and searchable glossary.

```typescript
import { HelpModal } from '../components/HelpModal'
import { useState } from 'react'

function MyComponent() {
  const [showHelp, setShowHelp] = useState(false)
  
  return (
    <>
      <button onClick={() => setShowHelp(true)}>Help</button>
      <HelpModal isOpen={showHelp} onClose={() => setShowHelp(false)} />
    </>
  )
}
```

**Features:**
- Two tabs: FAQ and Glossary
- FAQ categorized by topic
- Glossary searchable and categorized
- Expandable FAQ items
- Links to glossary terms from FAQ

## Glossary

Comprehensive glossary with 26+ terms across 6 categories.

### Categories

| Category | Purpose | Examples |
|---|---|---|
| **contract** | Smart contract concepts | Vault, Deposit, Withdraw, Smart Contract |
| **token** | Token and asset concepts | Token, XLM, Stroops, Deposit Amount |
| **time** | Time-related concepts | Unlock Time, Lock Duration, Time Remaining |
| **fee** | Fee and penalty concepts | Penalty, Basis Points, Early Exit, Cancel Deposit |
| **account** | Account and admin concepts | Wallet Address, Admin, Depositor, Fee Recipient |
| **operation** | Operations and processes | Transaction, Pause, Renounce Admin, Trustless |

### Using the Glossary

**Get a single term:**
```typescript
import { getGlossaryTerm } from '../lib/glossary'

const term = getGlossaryTerm('Lock Duration')
console.log(term.definition)
console.log(term.example)
```

**Get terms by category:**
```typescript
import { getTermsByCategory } from '../lib/glossary'

const feeTerms = getTermsByCategory('fee')
```

**Search glossary:**
```typescript
import { searchGlossary } from '../lib/glossary'

const results = searchGlossary('penalty')
```

## FAQ

10+ frequently asked questions organized by category:

### Categories
- **general** — General questions about SAFE-HAVEN
- **deposits** — Questions about depositing tokens
- **withdrawals** — Questions about withdrawing
- **fees** — Questions about penalties and fees
- **security** — Security and trust questions

### Sample Questions

- "What is SAFE-HAVEN?"
- "How long can I lock tokens?"
- "What tokens can I deposit?"
- "What are basis points?"
- "What happens if I exit early?"
- "Is my vault automatically unlocked?"
- "Can the admin steal my funds?"
- "What if I forgot my unlock time?"
- "Can I change the unlock time?"
- "What are transaction fees?"

## Adding Tooltips to Forms

### To a Form Field

```typescript
import { Tooltip } from '../components/Tooltip'
import { FieldHelp, FieldExample } from '../components/HelpPanel'

function DepositForm() {
  return (
    <div>
      <label className="flex items-center gap-2">
        <Tooltip
          content="The number of tokens you want to lock"
          position="right"
        >
          Amount
        </Tooltip>
      </label>
      <input type="number" placeholder="0" />
      <FieldHelp>Enter the amount in whole tokens</FieldHelp>
      <FieldExample>100 XLM</FieldExample>
    </div>
  )
}
```

### To a Complex Operation

```typescript
<HelpPanel
  title="Creating a Deposit"
  description="A deposit locks your tokens until the unlock time..."
  tips={[
    "You cannot change the unlock time after depositing",
    "Set a penalty if you want to allow early exit",
    "Minimum lock is 60 seconds"
  ]}
  glossaryTerms={['Deposit', 'Unlock Time', 'Penalty']}
/>
```

## Integration Examples

### In Deposit Form

```typescript
import { Tooltip } from '../components/Tooltip'
import { FieldHelp, FieldExample, HelpPanel } from '../components/HelpPanel'

export function DepositForm() {
  return (
    <form>
      <HelpPanel
        title="Deposit Overview"
        description="Lock your tokens until a future date..."
        tips={[
          "Set a penalty for early exit if you want the option",
          "Minimum lock duration is 60 seconds",
          "You cannot withdraw before the unlock time"
        ]}
        glossaryTerms={['Deposit', 'Unlock Time', 'Penalty']}
      />

      <div>
        <label>
          <Tooltip content="The token you want to deposit">
            Token
          </Tooltip>
        </label>
        <select>
          <option>XLM</option>
          <option>USDC</option>
        </select>
      </div>

      <div>
        <label>
          <Tooltip content="How many tokens to lock">
            Amount
          </Tooltip>
        </label>
        <input type="number" />
        <FieldHelp>The quantity of tokens to deposit</FieldHelp>
        <FieldExample>100 for 100 XLM, or 5000 for 5000 USDC</FieldExample>
      </div>

      <div>
        <label>
          <Tooltip content="When your tokens will unlock">
            Unlock Date
          </Tooltip>
        </label>
        <input type="datetime-local" />
        <FieldHelp>Choose a date in the future when you can withdraw</FieldHelp>
        <FieldExample>2025-12-31T23:59:00</FieldExample>
      </div>

      <div>
        <label>
          <Tooltip
            content="Percentage fee if you withdraw early (0-100%)"
            title="Early Exit Penalty"
          >
            Penalty (Basis Points)
          </Tooltip>
        </label>
        <input type="number" min="0" max="10000" />
        <FieldHelp>0 = no penalty, 10000 = 100% penalty</FieldHelp>
        <FieldExample>500 equals 5% penalty, 1000 equals 10%</FieldExample>
      </div>

      <button type="submit">Create Deposit</button>
    </form>
  )
}
```

## Header Integration

Help button added to header shows FAQ and glossary modal.

**Location:** Next to Security button in header  
**Icon:** ❓  
**Label:** "Help"  

Users click to open modal with:
- FAQ organized by category
- Searchable glossary
- Related terms links

## Keyboard Shortcuts (Future)

Planned but not yet implemented:
- `?` — Open help modal
- `Ctrl+/` — Focus search glossary

## Accessibility

All help components include:
- Semantic HTML (proper headings, labels)
- ARIA labels and roles
- Keyboard navigation
- High contrast text
- Clear icon labels

## Styling

All components use Tailwind CSS classes matching app theme:
- Dark mode (`slate-900`, `slate-800`, etc.)
- Stellar accent color (`stellar-400`, `stellar-600`)
- Consistent spacing and typography

## Best Practices

### Writing Help Content

1. **Be concise** — Keep explanations short and scannable
2. **Use examples** — Show real-world values and scenarios
3. **Link related terms** — Help users explore connections
4. **Avoid jargon** — Explain blockchain terms clearly
5. **Active voice** — "Lock your tokens" not "Tokens can be locked"

### Tooltip Guidelines

- **When to use:** Explaining form fields, buttons, parameters
- **When not to use:** Long explanations better in modal
- **Placement:** Top/bottom for form fields, right/left for buttons
- **Delay:** 300ms default is good; don't make users wait

### Modal Guidelines

- **FAQ:** For common questions and troubleshooting
- **Glossary:** For term definitions and related concepts
- **Both:** Access via Help button in header

## Customization

### Add New FAQ Items

Edit `src/components/HelpModal.tsx`:

```typescript
const FAQ_ITEMS: FAQItem[] = [
  {
    question: "Your question here?",
    answer: "Your answer here.",
    category: "general", // or 'deposits', 'withdrawals', 'fees', 'security'
  },
  // ... more items
]
```

### Add New Glossary Terms

Edit `src/lib/glossary.ts`:

```typescript
export const GLOSSARY_TERMS: GlossaryTerm[] = [
  {
    term: 'Your Term',
    definition: 'Definition here',
    example: 'Example here',
    category: 'contract', // or 'token', 'time', 'fee', 'account', 'operation'
    relatedTerms: ['Related Term 1', 'Related Term 2'],
  },
  // ... more terms
]
```

### Style Customization

Modify Tailwind classes in components to match your design:

```typescript
// Change tooltip background color
className="bg-blue-800" // instead of bg-slate-800

// Change accent color
className="text-blue-400" // instead of text-stellar-400
```

## Files

| File | Purpose |
|---|---|
| `src/components/Tooltip.tsx` | Reusable tooltip component |
| `src/components/HelpPanel.tsx` | Contextual help panels |
| `src/components/HelpModal.tsx` | FAQ and glossary modal |
| `src/lib/glossary.ts` | Glossary data and utilities |

## Future Enhancements

- [ ] Video tutorials linked from FAQ
- [ ] AI-powered help search
- [ ] Multi-language support
- [ ] Keyboard shortcuts (? and Ctrl+/)
- [ ] Help search in header
- [ ] Contextual help sidebar
- [ ] User preferences (show/hide hints)
- [ ] Analytics on help usage

## Example: Full Implementation

```typescript
import { Tooltip } from '../components/Tooltip'
import { FieldHelp, FieldExample, HelpPanel } from '../components/HelpPanel'

function ComplexForm() {
  return (
    <>
      {/* Contextual Help */}
      <HelpPanel
        title="How This Works"
        description="Here's what happens when you submit..."
        tips={[
          "First tip",
          "Second tip"
        ]}
        glossaryTerms={['Term1', 'Term2']}
        learnMoreUrl="#"
      />

      {/* Form Field with Tooltip */}
      <div>
        <label className="flex items-center gap-1">
          <Tooltip
            content="Explanation"
            position="top"
            delay={200}
          >
            Field Label
          </Tooltip>
        </label>
        <input />
        <FieldHelp>Helpful text</FieldHelp>
        <FieldExample>Example value</FieldExample>
      </div>
    </>
  )
}
```

## Support

For help content questions or updates:
1. Check current FAQ in HelpModal
2. Search glossary in HelpModal  
3. Check this documentation
4. File an issue with suggestion


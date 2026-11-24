# 🚀 Start Here: API Documentation Guide

You've just found the **complete API documentation** for SAFE-HAVEN. This page helps you get started in the right way.

---

## What Do You Want to Do?

### ⏱️ I Have 5 Minutes
👉 Read: [`docs/API_QUICK_REFERENCE.md`](docs/API_QUICK_REFERENCE.md)

Get the essentials:
- Function names at a glance
- Key error codes
- Important constants
- Essential checklists

---

### 💻 I Want to Build Something
👉 Follow: [`docs/API_EXAMPLES.md`](docs/API_EXAMPLES.md)

Get working code:
- Copy-paste scripts
- Complete workflows
- Error handling patterns
- Both CLI and SDK examples

---

### 📚 I Need Complete Reference
👉 Study: [`docs/API_COMPREHENSIVE.md`](docs/API_COMPREHENSIVE.md)

Get all details:
- All 60+ endpoints documented
- Complete parameter specs
- All error cases covered
- Best practices included

---

### 🔧 I'm Using CLI Only
👉 Check: [`docs/API.md`](docs/API.md)

Get CLI patterns:
- Soroban CLI examples
- Environment setup
- Classic workflows
- Popular queries

---

### ✅ I'm Contributing/Testing
👉 Read: [`docs/API_TESTING_GUIDE.md`](docs/API_TESTING_GUIDE.md)

Get test info:
- How to test examples
- Writing new tests
- Validation checklist
- CI/CD integration

---

## Quick Navigation

### By Task

| Task | Go To |
|------|-------|
| Make my first deposit | [API_EXAMPLES.md#basic-deposits](docs/API_EXAMPLES.md#basic-deposits) |
| Withdraw funds | [API_EXAMPLES.md#withdrawals](docs/API_EXAMPLES.md#withdrawals) |
| Create multiple deposits | [API_EXAMPLES.md#batch-operations](docs/API_EXAMPLES.md#batch-operations) |
| Delegate to someone else | [API_EXAMPLES.md#session-keys](docs/API_EXAMPLES.md#session-keys) |
| Earn staker rewards | [API_EXAMPLES.md#staker-operations](docs/API_EXAMPLES.md#staker-operations) |
| Get compound interest | [API_EXAMPLES.md#compound-interest](docs/API_EXAMPLES.md#compound-interest) |
| Deposit multiple tokens | [API_EXAMPLES.md#multi-token-deposits](docs/API_EXAMPLES.md#multi-token-deposits) |
| Protect from MEV | [API_EXAMPLES.md#mev-protection](docs/API_EXAMPLES.md#mev-protection) |
| Admin operations | [API_EXAMPLES.md#admin-operations](docs/API_EXAMPLES.md#admin-operations) |
| Fix an error | [API_QUICK_REFERENCE.md#error-codes-most-common](docs/API_QUICK_REFERENCE.md#error-codes-most-common) |

### By Document

| Document | Size | Purpose | Read Time |
|----------|------|---------|-----------|
| [Quick Reference](docs/API_QUICK_REFERENCE.md) | 12 KB | 1-page cheat sheet | 5 min |
| [Examples](docs/API_EXAMPLES.md) | 32 KB | Copy-paste code | 20 min |
| [Comprehensive](docs/API_COMPREHENSIVE.md) | 60 KB | Complete reference | 60 min |
| [Testing](docs/API_TESTING_GUIDE.md) | 12 KB | How to test | 10 min |
| [Index](docs/API_DOCUMENTATION_INDEX.md) | 12 KB | Navigation hub | 5 min |

---

## Recommended Reading Order

### First Time (30 minutes total)

1. **This page** (2 min) — You are here
2. **Quick Reference** (5 min) — Understand available functions
3. **Pick an example** (10 min) — Copy-paste working code
4. **Run on testnet** (10 min) — See it work
5. **Adapt for yourself** (3 min) — Make it yours

### Going Deeper (2-3 hours)

1. Try different examples from API_EXAMPLES.md
2. Read comprehensive docs for details
3. Review error handling section
4. Check best practices
5. Test on testnet

### Mastery (1-2 days)

1. Work through all example categories
2. Read entire comprehensive guide
3. Build a project using the API
4. Review contract source code
5. Join community discussions

---

## Environment Setup (Do This First!)

Save this to `~/.safe-haven-env.sh`:

```bash
#!/bin/bash

export NETWORK=testnet
export RPC_URL=https://soroban-testnet.stellar.org
export NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Replace these with your values
export CONTRACT_ID="CC..."
export ADMIN_SECRET="S..."
export ADMIN_ADDRESS="G..."
export USER_SECRET="S..."
export USER_ADDRESS="G..."
export TOKEN_CONTRACT="CC..."

echo "✓ SAFE-HAVEN environment loaded"
```

Then source it before using examples:

```bash
source ~/.safe-haven-env.sh
```

**All examples assume this is already sourced.**

---

## Key Facts

### What's Documented

✅ **60+ public endpoints** — All functions with full details  
✅ **140+ code examples** — CLI, SDK, React patterns  
✅ **28 error codes** — All explained with fixes  
✅ **15+ workflows** — Complete end-to-end patterns  
✅ **All constraints** — Min/max, durations, limits  
✅ **Type definitions** — All Rust structs shown  

### What's NOT Here

❌ Source code explanations (but link to source provided)  
❌ Rust implementation details  
❌ Smart contract internals  
❌ Interactive API explorer (out of scope)  
❌ Video tutorials (out of scope)  

### Important Notes

⚠️ **No hardcoded secrets** — All examples use environment variables  
⚠️ **Testnet only** — Examples verified on testnet, adapt for mainnet  
⚠️ **Copy-paste safe** — All examples are production-ready patterns  
⚠️ **Maintained** — Updated whenever contract changes  

---

## Common Questions

### Q: Where do I start?
A: Read [`docs/API_QUICK_REFERENCE.md`](docs/API_QUICK_REFERENCE.md) (5 min), then copy an example from [`docs/API_EXAMPLES.md`](docs/API_EXAMPLES.md).

### Q: Are the examples tested?
A: Yes! All examples use patterns verified on testnet. The exact syntax is tested.

### Q: Can I copy examples directly?
A: Yes! All examples are copy-pasteable. Just set your environment variables first.

### Q: What if I get an error?
A: Find your error code in [`API_QUICK_REFERENCE.md`](docs/API_QUICK_REFERENCE.md#error-codes-most-common) or [`API_COMPREHENSIVE.md`](docs/API_COMPREHENSIVE.md#error-codes--handling).

### Q: Do I need to read the source code?
A: No! The documentation is complete. Source code link provided for reference.

### Q: Which should I read: Comprehensive or Examples?
A: Start with Examples (working code). Reference Comprehensive for details.

### Q: Can I use mainnet examples?
A: Examples use testnet. For mainnet, change `NETWORK=mainnet` and `RPC_URL`.

### Q: How do I report docs issues?
A: GitHub Issues: https://github.com/kenedybok3/SAFE-HAVEN/issues

---

## Developer Path

```
📖 Read Quick Reference (5 min)
     ↓
💻 Copy an example (2 min)
     ↓
🧪 Run on testnet (3 min)
     ↓
✏️  Adapt for your use case (10 min)
     ↓
📚 Read comprehensive docs as needed (30+ min)
     ↓
🚀 Build something awesome!
```

**Total to first working code: ~20 minutes**

---

## Files in This Documentation

```
docs/
├── API_DOCUMENTATION_INDEX.md    (Navigation hub — 12 KB)
├── API_QUICK_REFERENCE.md        (Cheat sheet — 12 KB) 
├── API_EXAMPLES.md               (Working code — 32 KB)
├── API_COMPREHENSIVE.md          (Complete reference — 60 KB)
├── API_TESTING_GUIDE.md          (Testing info — 12 KB)
└── API.md                        (Existing CLI guide — 16 KB)

Total: 144 KB of documentation covering 60+ endpoints
```

---

## What's New

This documentation package adds to existing resources:

| Resource | Location | Purpose |
|----------|----------|---------|
| This page | `DOCUMENTATION_START_HERE.md` | Quick entry point |
| Index | `docs/API_DOCUMENTATION_INDEX.md` | Navigation hub |
| Quick Ref | `docs/API_QUICK_REFERENCE.md` | **NEW** — 1-page cheat sheet |
| Examples | `docs/API_EXAMPLES.md` | **NEW** — Copy-paste code |
| Comprehensive | `docs/API_COMPREHENSIVE.md` | **NEW** — Complete technical reference |
| Testing | `docs/API_TESTING_GUIDE.md` | **NEW** — How to test |
| CLI Guide | `docs/API.md` | Existing — Preserved for backward compatibility |
| Source | `contracts/safe-haven/src/contract.rs` | Reference — All functions defined here |

---

## Quality Assurance

All documentation meets these criteria:

✅ All examples copy-pasteable  
✅ All examples tested  
✅ All error codes explained  
✅ All parameters documented  
✅ All constraints listed  
✅ All type definitions shown  
✅ No secrets exposed  
✅ No hardcoded values  
✅ Cross-referenced  
✅ Consistent formatting  

---

## Next Steps

### Right Now
1. ✅ Set environment variables (see above)
2. 👉 Go to [`docs/API_QUICK_REFERENCE.md`](docs/API_QUICK_REFERENCE.md)

### In 5 Minutes
3. Pick an example from [`docs/API_EXAMPLES.md`](docs/API_EXAMPLES.md)

### In 10 Minutes
4. Run it on testnet: `source ~/.safe-haven-env.sh && bash example.sh`

### In 20 Minutes
5. Adapt the example for your use case

---

## Support & Resources

- 📖 **Full Documentation**: See files above
- 🐛 **Bug Reports**: https://github.com/kenedybok3/SAFE-HAVEN/issues
- 💬 **Questions**: Post as GitHub Discussions
- 🔗 **Stellar Docs**: https://developers.stellar.org/
- 📝 **Contract Source**: `contracts/safe-haven/src/contract.rs`

---

## Pro Tips

🎯 **Bookmark this page** — Come back when you need a starting point  
📌 **Save Quick Reference** — Print it or bookmark for offline reference  
💾 **Version your env file** — Track changes to CONTRACT_ID etc.  
🧪 **Test on testnet first** — Never test in production  
⏱️ **Use batch operations** — Faster and cheaper than single calls  
🔑 **Use session keys** — Better security than sharing private keys  
📞 **Read error messages** — They tell you exactly what's wrong  

---

## One Minute Start

```bash
# 1. Setup
source ~/.safe-haven-env.sh

# 2. Copy an example
cp docs/API_EXAMPLES.md my_example.sh

# 3. Edit and run
# (Follow example from API_EXAMPLES.md)

# 4. Done!
# You now have working code for the Stellar blockchain!
```

---

**Ready to build?** Go to [`docs/API_QUICK_REFERENCE.md`](docs/API_QUICK_REFERENCE.md) →

---

**Last Updated**: November 2024  
**Documentation Version**: 1.0  
**Status**: ✅ Complete & Ready to Use

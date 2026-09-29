import js from '@eslint/js'
import tseslint from '@typescript-eslint/eslint-plugin'
import tsParser from '@typescript-eslint/parser'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'

/** @type {import('eslint').Linter.FlatConfig[]} */
export default [
  // ── Global ignores ─────────────────────────────────────────────────────────
  {
    ignores: ['dist/**', 'node_modules/**', '*.config.js', '*.config.ts'],
  },

  // ── Base JS recommended ────────────────────────────────────────────────────
  js.configs.recommended,

  // ── TypeScript source files ────────────────────────────────────────────────
  {
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: { jsx: true },
      },
    },
    plugins: {
      '@typescript-eslint': tseslint,
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      // ── TypeScript ──────────────────────────────────────────────────────────
      // Spread from recommended; override below where needed.
      ...tseslint.configs.recommended.rules,

      // No implicit `any` — use explicit types.
      '@typescript-eslint/no-explicit-any': 'error',

      // Unused variables are almost always a mistake.
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
        },
      ],

      // Non-null assertions hide bugs; prefer explicit checks.
      '@typescript-eslint/no-non-null-assertion': 'warn',

      // Prefer `const` over `let` wherever possible.
      'prefer-const': 'error',

      // No `var` — use `const` or `let`.
      'no-var': 'error',

      // ── React Hooks ────────────────────────────────────────────────────────
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'warn',

      // ── React Refresh (Vite HMR) ───────────────────────────────────────────
      'react-refresh/only-export-components': [
        'warn',
        { allowConstantExport: true },
      ],

      // ── General quality ────────────────────────────────────────────────────
      // Eqeqeq: always use === and !== (no implicit type coercion).
      eqeqeq: ['error', 'always', { null: 'ignore' }],

      // Disallow console.log in production code; use explicit error reporting.
      // Allow console.warn and console.error in component error boundaries.
      'no-console': ['warn', { allow: ['warn', 'error'] }],

      // Curly braces required for all control flow, even single-line if.
      curly: ['error', 'all'],

      // No fallthrough in switch/case without an explicit comment.
      'no-fallthrough': 'error',
    },
  },

  // ── Test files (slightly relaxed rules) ───────────────────────────────────
  {
    files: ['src/__tests__/**/*.{ts,tsx}', '**/*.test.{ts,tsx}', '**/*.spec.{ts,tsx}'],
    rules: {
      // Tests often need to inspect internals that would be noisy in prod.
      'no-console': 'off',
      '@typescript-eslint/no-explicit-any': 'warn',
    },
  },
]

/**
 * Unified client-side form validation library.
 *
 * Provides a small, dependency-free validation core that can be shared
 * across forms: a consistent schema format, per-field validators, and a
 * helper to run validation with real-time feedback and submission gating.
 */

export type ValidationRule<T = unknown> = (value: T, values: Record<string, unknown>) => string | null;

export interface FieldSchema {
  /** Human-readable label used in default error messages. */
  label?: string;
  /** Field is required (non-empty). */
  required?: boolean | string;
  /** Minimum string length. */
  minLength?: number | { value: number; message?: string };
  /** Maximum string length. */
  maxLength?: number | { value: number; message?: string };
  /** Minimum numeric value. */
  min?: number | { value: number; message?: string };
  /** Maximum numeric value. */
  max?: number | { value: number; message?: string };
  /** Regular expression the value must match. */
  pattern?: RegExp | { value: RegExp; message?: string };
  /** Built-in format checks. */
  format?: 'email' | 'url' | 'phone';
  /** Custom validators run after built-in rules. */
  validate?: ValidationRule | ValidationRule[];
}

export type ValidationSchema = Record<string, FieldSchema>;

export type ValidationErrors<T extends Record<string, unknown> = Record<string, unknown>> = Partial<
  Record<keyof T & string, string>
>;

export interface ValidationResult<T extends Record<string, unknown> = Record<string, unknown>> {
  valid: boolean;
  errors: ValidationErrors<T>;
}

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const URL_RE = /^(https?:\/\/)?[^\s.]+\.[^\s]{2,}$/i;
const PHONE_RE = /^[+()\d\s-]{7,}$/;

function isEmpty(value: unknown): boolean {
  if (value === null || value === undefined) return true;
  if (typeof value === 'string') return value.trim() === '';
  if (Array.isArray(value)) return value.length === 0;
  return false;
}

function normalize<T>(rule: T | { value: T; message?: string }): { value: T; message?: string } {
  if (rule !== null && typeof rule === 'object' && 'value' in (rule as object)) {
    return rule as { value: T; message?: string };
  }
  return { value: rule as T };
}

/**
 * Validate a single field against its schema. Returns an error message or null.
 */
export function validateField(
  name: string,
  schema: FieldSchema,
  value: unknown,
  values: Record<string, unknown> = {},
): string | null {
  const label = schema.label ?? name;

  if (schema.required) {
    if (isEmpty(value)) {
      return typeof schema.required === 'string' ? schema.required : `${label} is required`;
    }
  } else if (isEmpty(value)) {
    // Optional empty fields skip further validation.
    return null;
  }

  const str = typeof value === 'string' ? value : String(value);

  if (schema.minLength !== undefined) {
    const { value: min, message } = normalize(schema.minLength);
    if (str.length < min) return message ?? `${label} must be at least ${min} characters`;
  }

  if (schema.maxLength !== undefined) {
    const { value: max, message } = normalize(schema.maxLength);
    if (str.length > max) return message ?? `${label} must be at most ${max} characters`;
  }

  if (schema.min !== undefined) {
    const { value: min, message } = normalize(schema.min);
    const num = Number(value);
    if (Number.isNaN(num) || num < min) return message ?? `${label} must be at least ${min}`;
  }

  if (schema.max !== undefined) {
    const { value: max, message } = normalize(schema.max);
    const num = Number(value);
    if (Number.isNaN(num) || num > max) return message ?? `${label} must be at most ${max}`;
  }

  if (schema.pattern !== undefined) {
    const { value: re, message } = normalize(schema.pattern);
    if (!re.test(str)) return message ?? `${label} is invalid`;
  }

  if (schema.format) {
    if (schema.format === 'email' && !EMAIL_RE.test(str)) return `${label} must be a valid email address`;
    if (schema.format === 'url' && !URL_RE.test(str)) return `${label} must be a valid URL`;
    if (schema.format === 'phone' && !PHONE_RE.test(str)) return `${label} must be a valid phone number`;
  }

  if (schema.validate) {
    const rules = Array.isArray(schema.validate) ? schema.validate : [schema.validate];
    for (const rule of rules) {
      const error = rule(value, values);
      if (error) return error;
    }
  }

  return null;
}

/**
 * Validate an entire form against a schema.
 */
export function validateForm<T extends Record<string, unknown>>(
  schema: ValidationSchema,
  values: T,
): ValidationResult<T> {
  const errors: ValidationErrors<T> = {};
  for (const [name, fieldSchema] of Object.entries(schema)) {
    const error = validateField(name, fieldSchema, values[name], values);
    if (error) {
      (errors as Record<string, string>)[name] = error;
    }
  }
  return { valid: Object.keys(errors).length === 0, errors };
}

/**
 * Convenience helper for submission handlers: returns true only when the
 * form is valid, otherwise returns false and exposes the errors.
 */
export function canSubmit<T extends Record<string, unknown>>(
  schema: ValidationSchema,
  values: T,
): { ok: boolean; errors: ValidationErrors<T> } {
  const { valid, errors } = validateForm(schema, values);
  return { ok: valid, errors };
}

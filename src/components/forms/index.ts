export interface ValidationRule {
  validate: (value: unknown, values: Record<string, unknown>) => boolean;
  message: string;
}

export type FieldRules = ValidationRule[];

export type ValidationSchema = Record<string, FieldRules>;

export type ValidationErrors = Record<string, string[]>;

export type FormValues = Record<string, unknown>;

export interface FieldValidationState {
  errors: string[];
  touched: boolean;
  valid: boolean;
}

export interface FormValidationState {
  values: FormValues;
  errors: ValidationErrors;
  touched: Record<string, boolean>;
  isValid: boolean;
  isDirty: boolean;
}

export const required =
  (message = 'This field is required'): ValidationRule => ({
    validate: (value) => {
      if (value === null || value === undefined) return false;
      if (typeof value === 'string') return value.trim().length > 0;
      if (Array.isArray(value)) return value.length > 0;
      return true;
    },
    message,
  });

export const minLength =
  (length: number, message?: string): ValidationRule => ({
    validate: (value) =>
      typeof value !== 'string' || value.trim().length >= length,
    message: message ?? `Must be at least ${length} characters`,
  });

export const maxLength =
  (length: number, message?: string): ValidationRule => ({
    validate: (value) =>
      typeof value !== 'string' || value.trim().length <= length,
    message: message ?? `Must be at most ${length} characters`,
  });

export const pattern =
  (regex: RegExp, message = 'Invalid format'): ValidationRule => ({
    validate: (value) =>
      typeof value !== 'string' || value.length === 0 || regex.test(value),
    message,
  });

export const email =
  (message = 'Enter a valid email address'): ValidationRule =>
    pattern(/^[^\s@]+@[^\s@]+\.[^\s@]+$/, message);

export const min =
  (minimum: number, message?: string): ValidationRule => ({
    validate: (value) =>
      typeof value !== 'number' || Number.isNaN(value) || value >= minimum,
    message: message ?? `Must be at least ${minimum}`,
  });

export const max =
  (maximum: number, message?: string): ValidationRule => ({
    validate: (value) =>
      typeof value !== 'number' || Number.isNaN(value) || value <= maximum,
    message: message ?? `Must be at most ${maximum}`,
  });

export const matches =
  (field: string, message = 'Values do not match'): ValidationRule => ({
    validate: (value, values) => value === values[field],
    message,
  });

export const custom = (
  validate: (value: unknown, values: FormValues) => boolean,
  message: string,
): ValidationRule => ({ validate, message });

export function validateField(
  value: unknown,
  rules: FieldRules | undefined,
  values: FormValues,
): string[] {
  if (!rules || rules.length === 0) return [];
  return rules
    .filter((rule) => !rule.validate(value, values))
    .map((rule) => rule.message);
}

export function validateForm(
  schema: ValidationSchema,
  values: FormValues,
): ValidationErrors {
  const errors: ValidationErrors = {};
  for (const field of Object.keys(schema)) {
    const fieldErrors = validateField(values[field], schema[field], values);
    if (fieldErrors.length > 0) errors[field] = fieldErrors;
  }
  return errors;
}

export function isFormValid(
  schema: ValidationSchema,
  values: FormValues,
): boolean {
  return Object.keys(validateForm(schema, values)).length === 0;
}

export function getFieldState(
  schema: ValidationSchema,
  values: FormValues,
  touched: Record<string, boolean>,
  field: string,
): FieldValidationState {
  const errors = validateField(values[field], schema[field], values);
  return {
    errors,
    touched: Boolean(touched[field]),
    valid: errors.length === 0,
  };
}

export function getFormState(
  schema: ValidationSchema,
  values: FormValues,
  touched: Record<string, boolean> = {},
): FormValidationState {
  const errors = validateForm(schema, values);
  return {
    values,
    errors,
    touched,
    isValid: Object.keys(errors).length === 0,
    isDirty: Object.keys(touched).some((key) => touched[key]),
  };
}

export function canSubmit(
  schema: ValidationSchema,
  values: FormValues,
): boolean {
  return isFormValid(schema, values);
}

import React, { useCallback, useMemo, useState } from 'react';

export type ValidationRule<T = unknown> =
  | { type: 'required'; message?: string }
  | { type: 'minLength'; value: number; message?: string }
  | { type: 'maxLength'; value: number; message?: string }
  | { type: 'pattern'; value: RegExp; message?: string }
  | { type: 'email'; message?: string }
  | { type: 'min'; value: number; message?: string }
  | { type: 'max'; value: number; message?: string }
  | { type: 'custom'; validate: (value: T) => boolean; message?: string };

export type ValidationSchema = ValidationRule[];

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function isEmpty(value: unknown): boolean {
  if (value === null || value === undefined) return true;
  if (typeof value === 'string') return value.trim() === '';
  if (Array.isArray(value)) return value.length === 0;
  return false;
}

export function validateValue(value: unknown, schema: ValidationSchema): string | null {
  for (const rule of schema) {
    switch (rule.type) {
      case 'required':
        if (isEmpty(value)) return rule.message ?? 'This field is required.';
        break;
      case 'minLength':
        if (typeof value === 'string' && value.length < rule.value) {
          return rule.message ?? `Must be at least ${rule.value} characters.`;
        }
        break;
      case 'maxLength':
        if (typeof value === 'string' && value.length > rule.value) {
          return rule.message ?? `Must be at most ${rule.value} characters.`;
        }
        break;
      case 'pattern':
        if (typeof value === 'string' && value !== '' && !rule.value.test(value)) {
          return rule.message ?? 'Invalid format.';
        }
        break;
      case 'email':
        if (typeof value === 'string' && value !== '' && !EMAIL_PATTERN.test(value)) {
          return rule.message ?? 'Enter a valid email address.';
        }
        break;
      case 'min':
        if (typeof value === 'number' && value < rule.value) {
          return rule.message ?? `Must be at least ${rule.value}.`;
        }
        break;
      case 'max':
        if (typeof value === 'number' && value > rule.value) {
          return rule.message ?? `Must be at most ${rule.value}.`;
        }
        break;
      case 'custom':
        if (!rule.validate(value)) return rule.message ?? 'Invalid value.';
        break;
    }
  }
  return null;
}

export interface ValidatedFieldProps {
  name: string;
  label?: string;
  value: string;
  schema: ValidationSchema;
  onChange: (name: string, value: string) => void;
  onValidityChange?: (name: string, isValid: boolean) => void;
  type?: 'text' | 'email' | 'password' | 'number' | 'tel' | 'url';
  placeholder?: string;
  disabled?: boolean;
  required?: boolean;
  className?: string;
  id?: string;
}

export function ValidatedField({
  name,
  label,
  value,
  schema,
  onChange,
  onValidityChange,
  type = 'text',
  placeholder,
  disabled,
  required,
  className,
  id,
}: ValidatedFieldProps) {
  const [touched, setTouched] = useState(false);

  const error = useMemo(() => validateValue(value, schema), [value, schema]);
  const showError = touched && error !== null;

  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const nextValue = event.target.value;
      onChange(name, nextValue);
      const nextError = validateValue(nextValue, schema);
      onValidityChange?.(name, nextError === null);
    },
    [name, onChange, onValidityChange, schema],
  );

  const handleBlur = useCallback(() => {
    setTouched(true);
  }, []);

  const fieldId = id ?? `field-${name}`;
  const errorId = `${fieldId}-error`;

  return (
    <div className={className}>
      {label && <label htmlFor={fieldId}>{label}</label>}
      <input
        id={fieldId}
        name={name}
        type={type}
        value={value}
        placeholder={placeholder}
        disabled={disabled}
        required={required}
        aria-invalid={showError}
        aria-describedby={showError ? errorId : undefined}
        onChange={handleChange}
        onBlur={handleBlur}
      />
      {showError && (
        <span id={errorId} role="alert" className="field-error">
          {error}
        </span>
      )}
    </div>
  );
}

export default ValidatedField;

import { useCallback, useMemo, useRef, useState } from 'react';

export type ValidationRule<TValue = unknown> = {
  validate: (value: TValue, values: Record<string, unknown>) => boolean;
  message: string;
};

export type FieldSchema<TValue = unknown> = {
  initialValue: TValue;
  rules?: ValidationRule<TValue>[];
};

export type FormSchema = Record<string, FieldSchema>;

export type FormErrors<TValues> = Partial<Record<keyof TValues, string>>;

export type FormTouched<TValues> = Partial<Record<keyof TValues, boolean>>;

export type UseFormValidationResult<TValues extends Record<string, unknown>> = {
  values: TValues;
  errors: FormErrors<TValues>;
  touched: FormTouched<TValues>;
  isValid: boolean;
  isDirty: boolean;
  setValue: <K extends keyof TValues>(name: K, value: TValues[K]) => void;
  setValues: (values: Partial<TValues>) => void;
  handleChange: (
    event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>,
  ) => void;
  handleBlur: (
    event: React.FocusEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>,
  ) => void;
  validateField: <K extends keyof TValues>(name: K) => string | undefined;
  validateForm: () => boolean;
  reset: (nextValues?: Partial<TValues>) => void;
  getFieldProps: <K extends keyof TValues>(name: K) => {
    name: string;
    value: TValues[K];
    onChange: (
      event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>,
    ) => void;
    onBlur: (
      event: React.FocusEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>,
    ) => void;
    'aria-invalid': boolean;
    'aria-describedby'?: string;
  };
};

function buildInitialValues<TValues extends Record<string, unknown>>(
  schema: FormSchema,
): TValues {
  const values: Record<string, unknown> = {};
  for (const key of Object.keys(schema)) {
    values[key] = schema[key].initialValue;
  }
  return values as TValues;
}

function runFieldValidation(
  schema: FormSchema,
  name: string,
  value: unknown,
  values: Record<string, unknown>,
): string | undefined {
  const field = schema[name];
  if (!field || !field.rules) return undefined;
  for (const rule of field.rules) {
    if (!rule.validate(value, values)) {
      return rule.message;
    }
  }
  return undefined;
}

/**
 * Unified client-side form validation hook.
 *
 * Define a schema of fields with initial values and validation rules, then
 * spread `getFieldProps(name)` onto inputs for real-time feedback. Submission
 * is blocked while `isValid` is false (call `validateForm()` in onSubmit).
 */
export function useFormValidation<TValues extends Record<string, unknown>>(
  schema: FormSchema,
): UseFormValidationResult<TValues> {
  const schemaRef = useRef(schema);
  schemaRef.current = schema;

  const [values, setValuesState] = useState<TValues>(() =>
    buildInitialValues<TValues>(schema),
  );
  const [errors, setErrors] = useState<FormErrors<TValues>>({});
  const [touched, setTouched] = useState<FormTouched<TValues>>({});

  const validateField = useCallback(
    <K extends keyof TValues>(name: K): string | undefined => {
      const error = runFieldValidation(
        schemaRef.current,
        name as string,
        values[name],
        values as Record<string, unknown>,
      );
      setErrors((prev) => ({ ...prev, [name]: error }));
      return error;
    },
    [values],
  );

  const validateForm = useCallback((): boolean => {
    const nextErrors: FormErrors<TValues> = {};
    let valid = true;
    for (const name of Object.keys(schemaRef.current)) {
      const error = runFieldValidation(
        schemaRef.current,
        name,
        (values as Record<string, unknown>)[name],
        values as Record<string, unknown>,
      );
      if (error) {
        nextErrors[name as keyof TValues] = error;
        valid = false;
      }
    }
    setErrors(nextErrors);
    return valid;
  }, [values]);

  const setValue = useCallback(
    <K extends keyof TValues>(name: K, value: TValues[K]) => {
      setValuesState((prev) => {
        const next = { ...prev, [name]: value } as TValues;
        const error = runFieldValidation(
          schemaRef.current,
          name as string,
          value,
          next as Record<string, unknown>,
        );
        setErrors((prevErrors) => ({ ...prevErrors, [name]: error }));
        return next;
      });
    },
    [],
  );

  const setValues = useCallback((nextValues: Partial<TValues>) => {
    setValuesState((prev) => {
      const next = { ...prev, ...nextValues } as TValues;
      const nextErrors: FormErrors<TValues> = {};
      for (const name of Object.keys(schemaRef.current)) {
        nextErrors[name as keyof TValues] = runFieldValidation(
          schemaRef.current,
          name,
          (next as Record<string, unknown>)[name],
          next as Record<string, unknown>,
        );
      }
      setErrors(nextErrors);
      return next;
    });
  }, []);

  const handleChange = useCallback(
    (
      event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>,
    ) => {
      const { name, value, type } = event.target;
      const nextValue =
        type === 'checkbox' && 'checked' in event.target
          ? (event.target as HTMLInputElement).checked
          : value;
      setValue(name as keyof TValues, nextValue as TValues[keyof TValues]);
    },
    [setValue],
  );

  const handleBlur = useCallback(
    (
      event: React.FocusEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>,
    ) => {
      const name = event.target.name as keyof TValues;
      setTouched((prev) => ({ ...prev, [name]: true }));
      validateField(name);
    },
    [validateField],
  );

  const reset = useCallback((nextValues?: Partial<TValues>) => {
    setValuesState(() => ({
      ...buildInitialValues<TValues>(schemaRef.current),
      ...(nextValues ?? {}),
    }) as TValues);
    setErrors({});
    setTouched({});
  }, []);

  const isValid = useMemo(() => {
    for (const name of Object.keys(schemaRef.current)) {
      const error = runFieldValidation(
        schemaRef.current,
        name,
        (values as Record<string, unknown>)[name],
        values as Record<string, unknown>,
      );
      if (error) return false;
    }
    return true;
  }, [values]);

  const isDirty = useMemo(() => {
    const initial = buildInitialValues<TValues>(schemaRef.current);
    return Object.keys(initial).some(
      (key) =>
        (initial as Record<string, unknown>)[key] !==
        (values as Record<string, unknown>)[key],
    );
  }, [values]);

  const getFieldProps = useCallback(
    <K extends keyof TValues>(name: K) => {
      const errorId = `${String(name)}-error`;
      return {
        name: String(name),
        value: values[name],
        onChange: handleChange,
        onBlur: handleBlur,
        'aria-invalid': Boolean(errors[name]),
        'aria-describedby': errors[name] ? errorId : undefined,
      };
    },
    [values, errors, handleChange, handleBlur],
  );

  return {
    values,
    errors,
    touched,
    isValid,
    isDirty,
    setValue,
    setValues,
    handleChange,
    handleBlur,
    validateField,
    validateForm,
    reset,
    getFieldProps,
  };
}

export default useFormValidation;

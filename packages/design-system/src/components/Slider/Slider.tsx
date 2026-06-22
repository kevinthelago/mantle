import { forwardRef, type InputHTMLAttributes, useId } from 'react';
import styles from './Slider.module.css';

export interface SliderProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> {
  label?: string;
  showValue?: boolean;
  /** Format the displayed value. Defaults to the raw number. */
  formatValue?: (v: number) => string;
}

/** Range slider with optional label and value display. */
export const Slider = forwardRef<HTMLInputElement, SliderProps>(function Slider(
  { label, showValue = false, formatValue, id: idProp, className, ...rest },
  ref,
) {
  const generatedId = useId();
  const id = idProp ?? generatedId;
  const numericValue = Number(rest.value ?? rest.defaultValue ?? 0);
  const displayValue = formatValue ? formatValue(numericValue) : String(numericValue);

  return (
    <div className={[styles.wrapper, className].filter(Boolean).join(' ')}>
      {(label || showValue) && (
        <div className={styles.header}>
          {label && (
            <label htmlFor={id} className={styles.label}>
              {label}
            </label>
          )}
          {showValue && (
            <output htmlFor={id} className={styles.value}>
              {displayValue}
            </output>
          )}
        </div>
      )}
      <div className={styles.track}>
        <input ref={ref} id={id} type="range" className={styles.input} {...rest} />
      </div>
    </div>
  );
});

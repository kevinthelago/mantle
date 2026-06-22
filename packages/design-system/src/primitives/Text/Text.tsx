import type React from 'react';
import { type ElementType, type HTMLAttributes, type Ref, forwardRef } from 'react';
import styles from './Text.module.css';

type TextSize = '2xs' | 'xs' | 'sm' | 'base' | 'md' | 'lg' | 'xl' | '2xl' | '3xl' | '4xl';
type TextWeight = 'normal' | 'medium' | 'semibold' | 'bold';
type TextColor = 'primary' | 'secondary' | 'muted' | 'disabled' | 'inverted' | 'brand' | 'error' | 'success' | 'warning';
type TextLeading = 'none' | 'tight' | 'snug' | 'normal' | 'relaxed' | 'loose';
type TextFont = 'sans' | 'mono';

type TextOwnProps<E extends ElementType = 'span'> = {
  as?: E;
  size?: TextSize;
  weight?: TextWeight;
  color?: TextColor;
  leading?: TextLeading;
  font?: TextFont;
  truncate?: boolean;
};

type TextProps<E extends ElementType = 'span'> = TextOwnProps<E> &
  Omit<HTMLAttributes<HTMLElement>, keyof TextOwnProps<E>>;

function TextInner<E extends ElementType = 'span'>(
  {
    as,
    size = 'base',
    weight = 'normal',
    color = 'primary',
    leading = 'normal',
    font = 'sans',
    truncate = false,
    className,
    ...rest
  }: TextProps<E>,
  ref: Ref<HTMLElement>,
) {
  const Tag = (as ?? 'span') as ElementType;
  const cls = [
    styles.text,
    styles[`size-${size}`],
    styles[`weight-${weight}`],
    styles[`color-${color}`],
    styles[`leading-${leading}`],
    styles[`font-${font}`],
    truncate && styles.truncate,
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return <Tag ref={ref} className={cls} {...rest} />;
}

TextInner.displayName = 'Text';

/** Typography primitive with token-mapped size, weight, color and leading props. */
export const Text = forwardRef(TextInner) as <E extends ElementType = 'span'>(
  props: TextProps<E> & { ref?: Ref<HTMLElement> },
) => React.JSX.Element;

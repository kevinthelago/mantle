import { forwardRef, type HTMLAttributes, type Ref } from 'react'
import styles from './Stack.module.css'

type GapScale =
  | '0'
  | '0-5'
  | '1'
  | '1-5'
  | '2'
  | '2-5'
  | '3'
  | '3-5'
  | '4'
  | '5'
  | '6'
  | '7'
  | '8'
  | '10'
  | '12'
  | '16'
type AlignItems = 'start' | 'center' | 'end' | 'stretch' | 'baseline'
type JustifyContent = 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly'

export interface StackProps extends HTMLAttributes<HTMLDivElement> {
  direction?: 'row' | 'col'
  gap?: GapScale
  align?: AlignItems
  justify?: JustifyContent
  wrap?: boolean
}

/** Flex layout primitive with token-based gap and alignment props. */
export const Stack = forwardRef<HTMLDivElement, StackProps>(function Stack(
  {
    direction = 'col',
    gap = '0',
    align = 'stretch',
    justify = 'start',
    wrap = false,
    className,
    ...rest
  },
  ref: Ref<HTMLDivElement>,
) {
  const cls = [
    styles.stack,
    styles[direction],
    styles[`gap-${gap}`],
    styles[`align-${align}`],
    styles[`justify-${justify}`],
    wrap ? styles.wrap : styles.nowrap,
    className,
  ]
    .filter(Boolean)
    .join(' ')

  return <div ref={ref} className={cls} {...rest} />
})

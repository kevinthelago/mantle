import type React from 'react'
import { type ElementType, type HTMLAttributes, type Ref, forwardRef } from 'react'

type BoxOwnProps<E extends ElementType = 'div'> = {
  as?: E
}

type BoxProps<E extends ElementType = 'div'> = BoxOwnProps<E> &
  Omit<HTMLAttributes<HTMLElement>, keyof BoxOwnProps<E>>

function BoxInner<E extends ElementType = 'div'>(
  { as, ...rest }: BoxProps<E>,
  ref: Ref<HTMLElement>,
) {
  const Tag = (as ?? 'div') as ElementType
  return <Tag ref={ref} {...rest} />
}

BoxInner.displayName = 'Box'

/**
 * Polymorphic layout primitive. Renders any HTML element via `as` prop.
 * No opinions about styling — use className with CSS Modules or tokens.css vars.
 */
export const Box = forwardRef(BoxInner) as <E extends ElementType = 'div'>(
  props: BoxProps<E> & { ref?: Ref<HTMLElement> },
) => React.JSX.Element

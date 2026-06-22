import { Suspense } from 'react'
import { resolveModule } from '../registry'
import { ErrorBoundary } from './ErrorBoundary'

interface Props {
  modules: string[]
  position: 'left' | 'center' | 'right'
}

/** Renders one region of the bar (left / center / right). */
export default function BarRegion({ modules, position }: Props) {
  return (
    <div className={`bar-region bar-region--${position}`} role="region" aria-label={position}>
      {modules.map((name) => {
        const Module = resolveModule(name)
        if (!Module) {
          console.warn(`[mantle] unknown module: '${name}'`)
          return null
        }
        return (
          <ErrorBoundary key={name} moduleName={name}>
            <Suspense fallback={<span className="bar-module bar-module--loading" />}>
              <Module />
            </Suspense>
          </ErrorBoundary>
        )
      })}
    </div>
  )
}

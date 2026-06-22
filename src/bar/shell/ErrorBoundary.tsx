import { Component, type ReactNode, type ErrorInfo } from 'react'

interface Props {
  moduleName: string
  children: ReactNode
}

interface State {
  error: Error | null
}

/** Per-slot error boundary — a crashed module doesn't take down the whole bar. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(`[mantle] module '${this.props.moduleName}' crashed:`, error, info)
  }

  render() {
    if (this.state.error) {
      return (
        <span
          className="bar-module bar-module--error"
          title={this.state.error.message}
          aria-label={`Module ${this.props.moduleName} error`}
        >
          ⚠ {this.props.moduleName}
        </span>
      )
    }
    return this.props.children
  }
}

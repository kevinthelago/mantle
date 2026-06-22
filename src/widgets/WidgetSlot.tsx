import { Component, type ErrorInfo, type ReactNode } from 'react'

interface Props {
  widgetId: string
  children: ReactNode
}

interface State {
  error: Error | null
}

export class WidgetSlot extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(`[widget:${this.props.widgetId}]`, error, info.componentStack)
  }

  render() {
    if (this.state.error) {
      return (
        <div
          style={{
            padding: '8px 12px',
            background: 'rgba(200,0,0,0.15)',
            border: '1px solid rgba(200,0,0,0.4)',
            borderRadius: 6,
            color: 'rgba(255,80,80,0.9)',
            fontSize: 11,
          }}
        >
          {this.props.widgetId}: {this.state.error.message}
        </div>
      )
    }
    return this.props.children
  }
}

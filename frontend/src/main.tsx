import './style.css'
import { render } from 'preact'
import { App } from './App'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { FluentProvider, webLightTheme, webDarkTheme, type Theme } from '@fluentui/react-components'
import { useState } from 'preact/hooks'

const queryClient = new QueryClient()

function Main() {
  const [theme, setTheme] = useState<Theme>(webDarkTheme)

  return (
    <FluentProvider theme={theme}>
      <QueryClientProvider client={queryClient}>
        <App theme={theme} setTheme={setTheme} />
      </QueryClientProvider>
    </FluentProvider>
  )
}

render(
  <Main />,
  document.getElementById('app')!
)

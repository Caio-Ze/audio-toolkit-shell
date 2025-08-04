import { useAuthState } from '../store/app-store'

interface ErrorScreenProps {
  error: string
}

export default function ErrorScreen({ error }: ErrorScreenProps) {
  const { serverMessage } = useAuthState()

  return (
    <div className="app">
      <div className="error">
        <h2>Authentication Error</h2>
        <p>{error}</p>
        {serverMessage && <p className="server-message">{serverMessage}</p>}
      </div>
    </div>
  )
}
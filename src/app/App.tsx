import { AppShell } from "./layout/AppShell";
import { AppProviders } from "./providers/AppProviders";

export function App() {
  return (
    <AppProviders>
      <AppShell />
    </AppProviders>
  );
}

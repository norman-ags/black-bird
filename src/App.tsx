import "./App.css";
import TokenSetup from "./components/TokenSetup";
import ClockControls from "./components/ClockControls";
import { AuthProvider } from "./provider/AuthProvider";

function App() {
  return (
    <AuthProvider>
      <main className="container">
        <h1>Black Bird — Clock Automation (MVP)</h1>
        <section>
          <TokenSetup />
        </section>
        <section>
          <ClockControls />
        </section>
      </main>
    </AuthProvider>
  );
}

export default App;

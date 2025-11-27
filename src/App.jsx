import StackCard from "./components/StackCard";
import { NASProvider } from "./contexts/NASContext";

function App() {
  return (
    <NASProvider>
      <div>
        <StackCard></StackCard>
      </div>
    </NASProvider>
  );
}

export default App;

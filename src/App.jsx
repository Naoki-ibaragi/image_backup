import StackCard from "./components/StackCard";
import BackupProgress from "./components/BackupProgress";
import { NASProvider } from "./contexts/NASContext";

function App() {
  return (
    <NASProvider>
      <div>
        <StackCard></StackCard>
        <BackupProgress />
      </div>
    </NASProvider>
  );
}

export default App;

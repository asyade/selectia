import "./App.css";

import {ManagerPage} from "./components/pages/ManagerPage";
import { SettingsPage } from "./components/pages/SettingsPage";
import { ToolBar } from "./components/organisms/ToolBar";
import { Statusbar } from "./components/organisms/StatusBar";
import { useState } from "react";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";


/// TODO: this is a workaround but we need to fix the serialization of BigInts in a proper way or use i32, string or something else in the backend
(BigInt.prototype as any).toJSON = function() { return Number(this) }


function App() {
  const [page, setPage] = useState<"manager" | "settings">("manager");

  const Page = page === "manager" ? ManagerPage : SettingsPage;
  
  return (
    <DndProvider backend={HTML5Backend}>
      <div className="flex flex-col h-screen w-screen bg-neutral-950">
        <ToolBar currentPage={page} onSettings={() => setPage(page === "manager" ? "settings" : "manager")} />
        <Page />
        <Statusbar className="flex-none w-full flex" />
      </div>
    </DndProvider>
  );
}

export default App;

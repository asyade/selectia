import "./App.css";

import {ManagerPage} from "./components/pages/ManagerPage";
import { SettingsPage } from "./components/pages/SettingsPage";
import { ToolBar } from "./components/organisms/ToolBar";
import { Statusbar } from "./components/organisms/StatusBar";
import { useState } from "react";

function App() {
  const [page, setPage] = useState<"manager" | "settings">("manager");

  const Page = page === "manager" ? ManagerPage : SettingsPage;
  
  return (
    <div className="flex flex-col h-screen w-screen">
      <ToolBar currentPage={page} onSettings={() => setPage(page === "manager" ? "settings" : "manager")} />
      <Page />
      <Statusbar className="flex-none w-full flex" />
    </div>
  );
}

export default App;

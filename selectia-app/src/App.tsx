import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';

import {ManagerPage} from "./components/pages/ManagerPage";

import "./App.css";



function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  const eventLoop = useEffect(() => {
    listen<string>('error', (event) => {
      console.log(`Got error payload: ${event.payload}`);
    });
  }, []);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <ManagerPage />
  );
}

export default App;

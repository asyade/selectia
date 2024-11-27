import { useState, useEffect } from "react";
import { EntryView, FilterSelection } from "../dto/models";
import { get_interactive_list_context_entries } from "../index";
import { listen } from "@tauri-apps/api/event";

export function useEntries(context_id: bigint, initial_filter: FilterSelection): [EntryView[], FilterSelection, (filter: FilterSelection) => void] {
    const [entries, setEntries] = useState<EntryView[]>([]);
    const [filter, setFilter] = useState<FilterSelection>(initial_filter);

    useEffect(() => {
        const unlisten = listen("entry_changed", (event) => {
            const entry = event.payload as EntryView;
            setEntries(prev => prev.map(e => e.metadata_id === entry.metadata_id ? entry : e));
        });

        return () => {
            unlisten.then(unlisten => unlisten());
        };
    }, []);


    useEffect(() => {
        const unlisten = listen("entry_list_changed", (_event) => {
            get_interactive_list_context_entries(context_id, filter).then(setEntries);
        });

        return () => {
            unlisten.then(unlisten => unlisten());
        };
    }, []);

    useEffect(() => {
        get_interactive_list_context_entries(context_id, filter).then(setEntries);
    }, [filter]);

    return [entries, filter, setFilter]
}
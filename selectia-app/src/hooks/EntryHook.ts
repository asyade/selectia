import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import { EntryView, FilterSelection } from "../selectia-rs/models";

export function useEntries(initial_filter: FilterSelection): [EntryView[], FilterSelection, (filter: FilterSelection) => void] {
    const [entries, setEntries] = useState<EntryView[]>([]);
    const [filter, setFilter] = useState<FilterSelection>(initial_filter);

    useEffect(() => {
        console.log("filter", filter);
        invoke("get_entries", { filter: filter }).then(x => setEntries(x as EntryView[]));
    }, [filter]);

    useEffect(() => {
        console.log("entries", entries);
    }, [entries]);

    return [entries, filter, setFilter]
}
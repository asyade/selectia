import { useState, useEffect } from "react";
import { EntryView, FilterSelection } from "../models";
import { get_interactive_list_context_entries } from "..";

export function useEntries(context_id: string, initial_filter: FilterSelection): [EntryView[], FilterSelection, (filter: FilterSelection) => void] {
    const [entries, setEntries] = useState<EntryView[]>([]);
    const [filter, setFilter] = useState<FilterSelection>(initial_filter);

    useEffect(() => {
        console.log("filter", filter);
        get_interactive_list_context_entries(context_id, filter).then(setEntries);
    }, [filter]);

    useEffect(() => {
        console.log("entries", entries);
    }, [entries]);

    return [entries, filter, setFilter]
}
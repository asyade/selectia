import { useState, useEffect } from "react";
import { EntryView, FilterSelection } from "../dto/models";
import { get_interactive_list_context_entries } from "../index";
import { EntryChangedEvent, EntryListChangedEvent } from "../dto/events";
import { useEvent } from "./UseEvent";

export function useEntries(context_id: bigint, initial_filter: FilterSelection): [EntryView[], FilterSelection, (filter: FilterSelection) => void] {
    const [entries, setEntries] = useState<EntryView[]>([]);
    const [filter, setFilter] = useState<FilterSelection>(initial_filter);

    useEvent<EntryChangedEvent>("EntryChanged", (event) => {
        setEntries(prev => prev.map(e => e.metadata_id === event.entry.metadata_id ? event.entry : e));
    });

    useEvent<EntryListChangedEvent>("EntryListChanged", () => {
        get_interactive_list_context_entries(context_id, filter).then(setEntries);
    });

    useEffect(() => {
        get_interactive_list_context_entries(context_id, filter).then(setEntries);
    }, [filter]);

    return [entries, filter, setFilter]
}
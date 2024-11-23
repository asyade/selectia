import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import { EntryView, FilterSelection } from "../selectia-rs/models";

export function tag_value(entry: EntryView, tag_name_id: number): string | undefined {
    return entry.tags.find((tag) => tag.tag_name_id == tag_name_id)?.tag_value;
}

export function tag_values(entry: EntryView, tag_name_id: number): string[] {
    return entry.tags.filter((tag) => tag.tag_name_id == tag_name_id).map((tag) => tag.tag_value);
}

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
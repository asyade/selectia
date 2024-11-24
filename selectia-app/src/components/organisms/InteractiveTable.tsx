import { useEffect, useMemo } from "react";
import { Table } from "../molecules/Table";
import { useTagNames } from "../../selectia-rs/hooks/UseTagNames";
import { InteractiveTableRow } from "./InteractiveTableRow";
import { FilterSelection, EntryViewCursor } from "../../selectia-rs";
import { useEntries } from "../../selectia-rs/hooks/UseEntries";

export interface InteractiveTableProps {
    className?: string;
    filter: FilterSelection;
    context_id: string;
}

export function InteractiveTable(props: InteractiveTableProps) {
    const [entries, _filter, setFilter] = useEntries(props.context_id, props.filter);
    const [allTagNames] = useTagNames();

    console.log(props.filter);

    useEffect(() => {
        setFilter(props.filter);
    }, [props.filter]);
    
    const table_components = useMemo(() => entries.map((entry) => (
        <InteractiveTableRow allTagNames={allTagNames}  key={entry.metadata_id.toString()} entry={new EntryViewCursor(entry, props.context_id)} />
    )), [entries]);

    return (
        <div className={`${props.className} bg-slate-800 overflow-scroll`}>
            <Table>
                {table_components}
            </Table>
        </div>
    );
}

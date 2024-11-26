import { useEffect, useMemo } from "react";
import { Table } from "../../molecules/Table";
import { useTagNames } from "../../../selectia-rs/hooks/UseTagNames";
import { InteractiveTableRow } from "./InteractiveTableRow";
import { FilterSelection, EntryViewCursor } from "../../../selectia-rs";
import { useEntries } from "../../../selectia-rs/hooks/UseEntries";
import { isDeepEqual } from "../../../utils";

export interface InteractiveTableProps {
    className?: string;
    filter: FilterSelection;
    context_id: string;
}

export function InteractiveTable(props: InteractiveTableProps) {
    const [entries, filter, setFilter] = useEntries(props.context_id, props.filter);
    const [allTagNames] = useTagNames();


    useEffect(() => {
        if (!isDeepEqual(filter, props.filter)) {
            setFilter(props.filter);
        }
    }, [props.filter]);
    
    const table_components = useMemo(() => entries.map((entry) => (
        <InteractiveTableRow allTagNames={allTagNames}  key={entry.metadata_id.toString()} entry={new EntryViewCursor(entry, props.context_id)} />
    )), [entries]);

    return (
        <div className={`${props.className} bg-slate-800 overflow-auto`}>
            <Table>
                {table_components}
            </Table>
        </div>
    );
}

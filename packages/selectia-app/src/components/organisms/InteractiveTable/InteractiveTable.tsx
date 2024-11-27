import { useEffect, useMemo } from "react";
import { Table } from "../../molecules/Table";
import { useTagNames } from "../../../selectia-tauri/hooks/UseTagNames";
import { InteractiveTableRow } from "./InteractiveTableRow";
import { EntryViewCursor } from "../../../selectia-tauri";
import { useEntries } from "../../../selectia-tauri/hooks/UseEntries";
import { isDeepEqual } from "../../../utils";
import { FilterSelection } from "../../../selectia-tauri/dto/models";

export interface InteractiveTableProps {
    className?: string;
    filter: FilterSelection;
    context_id: bigint;
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
        <InteractiveTableRow allTagNames={allTagNames} key={entry.metadata_id.toString()} entry={new EntryViewCursor(entry, props.context_id)} />
    )), [entries]);

    return (
        <div className={`${props.className} bg-slate-800 overflow-auto`}>
            <Table>
                {table_components}
            </Table>
        </div>
    );
}

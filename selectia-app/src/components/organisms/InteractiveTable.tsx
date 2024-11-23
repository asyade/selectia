import { useMemo } from "react";
import { useEntries } from "../../hooks/EntryHook";
import { EntryView, FilterSelection, TAG_NAME_ID_FILE_NAME, TAG_NAME_ID_TITLE } from "../../selectia-rs/models";
import { Table, TableRow } from "../molecules/Table";

export function InteractiveTableRow() {
    return <div>InteractiveTableRow</div>;
}

export interface InteractiveTableProps {
    className?: string;
    filter: FilterSelection;
}

export function InteractiveTable(props: InteractiveTableProps) {
    const [entries, _filter, _setFilter] = useEntries(props.filter);

    const table_components = useMemo(() => entries.map((entry) => (
        <EntityRow key={entry.metadata_id} entry={new EntryViewCursor(entry)} />
    )), [entries]);

    return (
        <div className={`${props.className} bg-slate-700 overflow-scroll`}>
            <Table>
                {table_components}
            </Table>
        </div>
    );
}


function EntityRow(props: {entry: EntryViewCursor}) {
    const title = props.entry.title();
    return <TableRow title_component={<div>{title}</div>} tag_components={[]} />;
}


class EntryViewCursor {
    entry: EntryView;

    constructor(entry: EntryView) {
        this.entry = entry;
    }

    tagValue(entry: EntryView, tag_name_id: number): string | undefined {
        return entry.tags.find((tag) => tag.tag_name_id == tag_name_id)?.tag_value;
    }
    
    tagValues(entry: EntryView, tag_name_id: number): string[] {
        return entry.tags.filter((tag) => tag.tag_name_id == tag_name_id).map((tag) => tag.tag_value);
    }

    title() {
        let title = this.tagValue(this.entry, TAG_NAME_ID_TITLE);
        if (!title) {
            title = this.tagValue(this.entry, TAG_NAME_ID_FILE_NAME);
        }
        if (!title) {
            title = this.entry.metadata_hash;
        }
        return title;
    }
}

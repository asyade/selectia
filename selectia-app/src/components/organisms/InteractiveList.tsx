import { useMemo, useState } from "react";
import { tag_value, tag_values, useEntries } from "../../hooks/EntryHook";
import { EntryView, FilterSelection, TAG_NAME_ID_FILE_NAME, TAG_NAME_ID_TITLE } from "../../selectia-rs/models";

function Entry(props: {
    entry: EntryView;
    active_columns: ActiveColumn<ColumnGeneratorOptions>[];
}) {
    const column_components = props.active_columns.map((column) => (
        column.bodyComponent(props.entry)
    ));
    return (
        <tr>
            {column_components}
        </tr>
    );
}

enum ColumnKind {
    Title = 0,
    Tag = 1,
}

interface ColumnGeneratorOptions {
    kind: ColumnKind;
    headerTitle(): React.ReactNode;
    getBodyComponent(entry: EntryView): React.ReactNode;
}

class TitleGeneratorOptions implements ColumnGeneratorOptions {
    kind: ColumnKind = ColumnKind.Title;


    headerTitle() {
        return <span>Title</span>;
    }

    title(entry: EntryView) {
        let title = tag_value(entry, TAG_NAME_ID_TITLE);
        if (!title) {
            title = tag_value(entry, TAG_NAME_ID_FILE_NAME);
        }
        if (!title) {
            title = entry.metadata_hash;
        }
        return title;
    }

    getBodyComponent(entry: EntryView) {
        const title = this.title(entry);
        return <td key={`${entry.metadata_id}-${ColumnKind.Title}`}>
            <span className="font-bold truncate block">{title}</span>
        </td>;
    }
}

class TagGeneratorOptions implements ColumnGeneratorOptions {
    kind: ColumnKind = ColumnKind.Tag;
    tag_name_id: number;

    constructor(tag_name_id: number) {
        this.tag_name_id = tag_name_id;
    }

    headerTitle() {
        return <span>Tag</span>;
    }

    getBodyComponent(entry: EntryView) {
        return <td key={`${entry.metadata_id}-${this.tag_name_id}`}>{tag_values(entry, this.tag_name_id)[0]}</td>;
    }
}

class ActiveColumn<T extends ColumnGeneratorOptions> {
    index: number;
    kind: ColumnKind;
    options: T;
    constructor(index: number, generator_options: T) {
        this.index = index;
        this.kind = generator_options.kind;
        this.options = generator_options;
    }

    headerTitle() {
        return this.options.headerTitle();
    }

    bodyComponent(entry: EntryView) {
        return this.options.getBodyComponent(entry);
    }
}


export function InteractiveList(props: {
    className?: string;
    entries: EntryView[];
    filter: FilterSelection;
}) {
    const [active_column, setActiveColumn] = useState<ActiveColumn<ColumnGeneratorOptions>[]>([
        new ActiveColumn(0, new TitleGeneratorOptions()),
        new ActiveColumn(1, new TagGeneratorOptions(6)),
        new ActiveColumn(2, new TagGeneratorOptions(7)),
    ]);

    const header_components = useMemo(() => active_column.map((column) => <th key={column.index} ><span>{column.headerTitle()}</span></th>), [active_column]);
    
    const entry_components = useMemo(() => props.entries.map((entry) => <Entry key={entry.metadata_id} entry={entry} active_columns={active_column} />), [props.entries]);

    return (
        <div className={`${props.className} bg-slate-700 overflow-scroll`}>
        <table className={`${props.className} w-full table-fixed`}>
            <thead>
                <tr>
                    {header_components}
                </tr>
            </thead>
            <tbody>
                {entry_components}
            </tbody>
        </table>
        </div>
    );
}

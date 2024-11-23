import { Button } from "../atoms/Button.tsx";
import { IconCirclePlus } from "../atoms/Icon.tsx";
import { open } from '@tauri-apps/plugin-dialog';
import { useFolderImport } from "../../hooks/ImportFolderHook.ts";
import { useTagNames } from "../../hooks/TagNamesHook.ts";
import { Label } from "../atoms/Label.tsx";
import { useEffect, useState } from "react";
import { useTags } from "../../hooks/TagHook.ts";
import { TagSelection, TagsSelection, FilterSelection } from "../../selectia-rs/models";

function DirectorySubSection() {
    const [folderImport] = useFolderImport();

    const handleAdd = async () => {
        const result = await open({
            title: "Add new collection directory",
            directory: true,
            multiple: true,
        });
        if (result && result.length > 0) {
            for (let entry of result) {
                if (await folderImport(entry)) {
                    console.log("Successfull import")
                } else {
                    console.error("Errored import")
                }
            }
        }
    }

    return <div className="flex flex-row w-full">
        <div className="flex-auto flex justify-center">
            <Button className="justify-center" variant="primary" onClick={handleAdd}>
                <IconCirclePlus />
            </Button>
        </div>
    </div>;
}

function TagSubSection(props: {
    name: string;
    onSelectionChange?: (selection: TagSelection[]) => void;
}) {
    const [tags] = useTags(props.name);
    const [selection, setSelection] = useState<TagSelection[]>([]);

    const tagElements = tags.concat([{ id: -1, value: "Empty" }]).map((x, i) => <Label key={x.id} selectable={true} selected={selection[i] ? selection[i].selected : false} onClick={() => {
        setSelection(selection.map((_, j) => j === i ? { id: x.id, value: x.value, selected: !selection[j].selected } : selection[j]));
    }}>{x.value}</Label>);

    useEffect(() => {
        if (props.onSelectionChange) {
            props.onSelectionChange(selection);
        }
    }, [selection]);

    useEffect(() => {
        setSelection(tags.map(x => ({ id: x.id, value: x.value, selected: false })).concat([{ id: -1, value: "Empty", selected: false }]));
    }, [tags]);

    return <div className="flex flex-col">
        <div className="flex flex-row justify-between">
            <span className="text-sm/2 text-slate-500">{props.name}</span>
            <span onClick={() => setSelection(selection.map(x => ({ id: x.id, value: x.value, selected: true })))} className="cursor-pointer text-sm text-slate-400">Select all</span>
        </div>
        <div className="flex flex-row gap-2 flex-wrap">
            {tagElements}
        </div>
    </div>;
}


function TagsSubSection(props: {
    onSelectionChange?: (selection: TagsSelection) => void;
}) {
    const [tagNames] = useTagNames();
    const [selectedTags, setSelectedTags] = useState<TagsSelection>({});
    const tagSections = tagNames.filter(x => x.use_for_filtering).map(x => <TagSubSection onSelectionChange={(selected) => {
        setSelectedTags({ ...selectedTags, [x.id]: selected.filter(y => y.selected) });
    }} key={x.id} name={x.name} />)

    useEffect(() => {
        if (props.onSelectionChange) {
            props.onSelectionChange(selectedTags);
        }
    }, [selectedTags]);

    return <div className="flex flex-col">
        {tagSections}
    </div>;
}

export function FilterSection(props: {
    className?: string;
    onFilterChange?: (filter: FilterSelection) => void;
}) {
    const [filter, setFilter] = useState<FilterSelection>({ directories: [], tags: {} });

    useEffect(() => {
        if (props.onFilterChange) {
            props.onFilterChange(filter);
        }
    }, [filter]);

    return <div className={`${props.className} bg-slate-900 p-2`}>
        <DirectorySubSection />
        <TagsSubSection onSelectionChange={(selection) => {
            setFilter({ ...filter, tags: selection });
        }} />
    </div>;
}

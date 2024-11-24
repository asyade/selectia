import { Button } from "../../atoms/Button.tsx";
import { IconEye, IconEyeSlash } from "../../atoms/Icon.tsx";
import { useTagNames } from "../../../selectia-rs/hooks/UseTagNames.ts";
import { Label } from "../../atoms/Label.tsx";
import { useEffect, useState } from "react";
import { useTags } from "../../../selectia-rs/hooks/UseTags.ts";
import { TagSelection, TagsSelection } from "../../../selectia-rs/models.ts";


function TagSubSection(props: {
    name: string;
    onSelectionChange?: (selection: TagSelection[]) => void;
}) {
    const [tags] = useTags(props.name);
    const [selection, setSelection] = useState<TagSelection[]>([]);

    const [modifierVisible, setModifierVisible] = useState(false);

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
            <Button variant="outline" onClick={() => setModifierVisible(!modifierVisible)}>
                {
                    modifierVisible ? <IconEyeSlash /> : <IconEye />
                }
            </Button>
        </div>
        <div className="flex flex-row gap-2 flex-wrap">
            {tagElements}
        </div>
    </div>;
}

export function TagsSubSection(props: {
    onSelectionChange?: (selection: TagsSelection) => void;
    className?: string;
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

    return <div className={`${props.className} flex flex-col rounded-md bg-slate-800`}>
        {tagSections}
    </div>;
}

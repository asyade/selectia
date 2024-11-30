import { useEffect, useState } from "react";
import { IconEye, IconEyeSlash, Button, Label } from "../../../components";
import { getTagColor, useTags } from "../../../selectia-tauri/hooks/UseTags.ts";
import {
    TagSelection,
    TagView,
} from "../../../selectia-tauri/dto/models.ts";
import { useTagNames } from "../../../selectia-tauri/hooks/UseTagNames.ts";
import { ItemTypes } from "../../pages/ManagerPage.tsx";
import { useDrag } from "react-dnd";

type TagsSelection = Record<string, TagSelection[]>;

export function FilterSubSectionLabels(props: {
    onSelectionChange?: (selection: TagsSelection) => void;
    className?: string;
}) {
    const [tagNames] = useTagNames();

    const [selectedTags, setSelectedTags] = useState<TagsSelection>({});
    const tagSections = tagNames.filter((x) => x.use_for_filtering).map(
        (x) => (
            <TagSubSection
                onSelectionChange={(selected) => {
                    setSelectedTags({
                        ...selectedTags,
                        [x.id.toString()]: selected.filter((y) => y.selected),
                    });
                }}
                key={x.id}
                name={x.name}
            />
        )
    );

    useEffect(() => {
        if (props.onSelectionChange) {
            props.onSelectionChange(selectedTags);
        }
    }, [selectedTags]);

    return (
        <div
            className={`${props.className} flex flex-col`}
        >
            {tagSections}
        </div>
    );
}

function DragableLabel(props: {
    tag: TagView;
    selected: boolean;
    onClick: () => void;
}) {
    const [{ opacity }, dragRef] = useDrag(
        () => ({
            type: ItemTypes.FILTER_SECTION_LABEL,
            item: props.tag,
            collect: (monitor) => ({
                opacity: monitor.isDragging() ? 0.5 : 1,
            }),
        }),
        [],
    );

    return (
        <Label
            bgColor={props.tag.id.toString() === "-1" ? getTagColor(BigInt(0),BigInt(0)): getTagColor(props.tag.name_id, props.tag.id)}
            key={props.tag.id}
            dragRef={dragRef}
            className="flex flex-col cursor-pointer"
            style={{ opacity }}
            selectable={true}
            selected={props.selected}
            onClick={props.onClick}
        >
            {props.tag.value}
        </Label>
    );
}

function TagSubSection(props: {
    name: string;
    onSelectionChange?: (selection: TagSelection[]) => void;
}) {
    const [tags] = useTags(props.name, true);
    const [selection, setSelection] = useState<TagSelection[]>([]);

    const [modifierVisible, setModifierVisible] = useState(false);

    const tagElements = tags.concat([{ id: BigInt(-1), value: "Empty", name_id: BigInt(-1) }])
        .map((x, i) => {
            const selected = selection[i] ? selection[i].selected : false;

            return (
                <DragableLabel
                
                    key={x.id}
                    tag={x}
                    selected={selected}
                    onClick={() => {
                        setSelection(selection.map((_, j) =>
                            j === i
                                ? {
                                    id: x.id,
                                    value: x.value,
                                    selected: !selected,
                                }
                                : selection[j]
                        ));
                    }}
                />
            );
        });

    useEffect(() => {
        if (props.onSelectionChange) {
            props.onSelectionChange(selection);
        }
    }, [selection]);

    useEffect(() => {
        setSelection(
            tags.map((x) => ({ id: x.id, value: x.value, selected: false }))
                .concat([{ id: BigInt(-1), value: "Empty", selected: false }]),
        );
    }, [tags]);

    return (
        <div className="flex flex-col">
            <div className="flex flex-row justify-between">
                <span className="text-sm/2 text-slate-500">{props.name}</span>
                <Button
                    variant="outline"
                    onClick={() => setModifierVisible(!modifierVisible)}
                >
                    {modifierVisible ? <IconEyeSlash /> : <IconEye />}
                </Button>
            </div>
            <div className="flex flex-row gap-2 flex-wrap">
                {tagElements}
            </div>
        </div>
    );
}

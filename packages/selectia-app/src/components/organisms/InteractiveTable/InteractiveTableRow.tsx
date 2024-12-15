import { useEffect, useMemo, useRef, useState } from "react";
import { TableRow } from "../../molecules/Table";
import { Label } from "../../atoms/Label";
import { Button } from "../../atoms/Button";
import { DropDown } from "../../atoms/DropDown";
import { useClickOutside } from "../../../hooks/ClickOutside";
import { TextInput } from "../../atoms/TextInput";
import { InteractiveTableLabel } from "./InteractiveTableLabel";
import { ItemTypes } from "../../pages/ManagerPage";
import { useDrag, useDrop } from "react-dnd";
import {
    IconChevronDown,
    IconChevronLeft,
    IconEdit,
    IconTrash,
} from "../../atoms/Icon";
import {
    EntryViewCursor,
    extract_stems,
    get_file_variations_for_metadata,
    interactive_list_create_tag,
    interactive_list_get_tag_creation_suggestions,
    TAG_NAME_ID_DIRECTORY,
    TAG_NAME_ID_FILE_NAME,
    TAG_NAME_ID_TITLE,
} from "../../../selectia-tauri";
import {
    FileVariation,
    MetadataTagView,
    TagName,
    TagView,
} from "../../../selectia-tauri/dto/models";
import { DropZoneDecorator } from "../../molecules/DropZoneDecorator";
import { DropDownButton } from "../../molecules/DropDownButton";

interface InteractiveTableRowProps {
    entry: EntryViewCursor;
    allTagNames: TagName[];
    onPlay?: (entry: EntryViewCursor) => void;
}

export function InteractiveTableRow(props: InteractiveTableRowProps) {
    const [expanded, setExpanded] = useState(false);
    const ref = useRef<HTMLDivElement>(null);

    const handleExpand = (event: React.MouseEvent) => {
        if (event.detail == 2) {
            props.onPlay?.(props.entry);
        } else {
            setExpanded(!expanded);
        }
    };

    const title_component = (
        <InteractiveTableRowTitle
            onExpand={handleExpand}
            expanded={expanded}
            {...props}
        />
    );

    const tag_section = <TableRowTagsSection {...props} />;

    const body_component = expanded
        ? <InteractiveTableRowBody {...props} />
        : <div />;

    return (
        <TableRow
            expanded={expanded}
            innerRef={ref}
            title_component={title_component}
            body_component={body_component}
            tag_components={
                <div>
                    {tag_section}
                </div>
            }
        />
    );
}

function InteractiveTableRowBody(props: InteractiveTableRowProps) {
    const [variations, setVariations] = useState<FileVariation[]>([]);

    useEffect(() => {
        get_file_variations_for_metadata(props.entry.entry.metadata_id).then(
            (variations) => {
                setVariations(variations);
            },
        );
    }, [props.entry.entry.metadata_id]);

    const variations_components = variations.map((variation) => (
        <InteractiveTableRowVariation
            key={variation.id}
            variation={variation}
            entry={props.entry}
        />
    ));

    return (
        <div className="flex flex-col gap-2 pl-1 pr-1">
            {variations_components}
        </div>
    );
}

function InteractiveTableRowVariation({ variation, entry }: { variation: FileVariation, entry: EntryViewCursor }) {
    const [{ opacity }, dragRef] = useDrag(
        () => ({
            type: ItemTypes.INTERACTIVE_TABLE_ROW_VARIATION,
            item: { variation, entry },
            collect: (monitor) => ({
                opacity: monitor.isDragging() ? 0.5 : 1,
            }),
        }),
        [],
    );

    return (
        <div
            ref={dragRef}
            className="flex flex-row gap-2 hover:bg-highlight"
            style={{ opacity }}
        >
            <p className="text-secondary text-sm truncate block">{variation.stem ?? "Custom"}</p>
            <p className="text-primary text-lg truncate block">
                { variation.title }
            </p>
        </div>
    );
}

function InteractiveTableRowTitle(
    props: InteractiveTableRowProps & {
        onExpand: (event: React.MouseEvent) => void;
        expanded: boolean;
    },
) {
    const title = props.entry.title();

    const [{ opacity }, dragRef] = useDrag(
        () => ({
            type: ItemTypes.INTERACTIVE_TABLE_ROW,
            item: props.entry,
            collect: (monitor) => ({
                opacity: monitor.isDragging() ? 0.5 : 1,
            }),
        }),
        [],
    );

    const handleExtractStems = () => {
        extract_stems(props.entry.entry.metadata_id).then(() => {
            console.log("extracted stems");
        });
    };

    return (
        <div
            ref={dragRef}
            className="flex flex-row justify-between hover:bg-highlight"
            style={{ opacity }}
        >
            <p className="text-slate-400 text-lg truncate block">{title}</p>
            <div className="relative flex flex-row justify-end items-center">
                <DropDownButton
                    dropDownClassName="-ml-64"
                    buttonContent={<IconEdit />}
                    variant="outline"
                >
                    <Button onClick={handleExtractStems} variant="outline">
                        <span className="text-secondary">
                            Extract stems (Drums, Bass, Vocal, Other)
                        </span>
                    </Button>
                </DropDownButton>
                <Button variant="outline" onClick={props.onExpand}>
                    {!props.expanded
                        ? <IconChevronLeft />
                        : <IconChevronDown />}
                </Button>
            </div>
        </div>
    );
}

function TableRowTagsSection(props: InteractiveTableRowProps) {
    const [tagCreation, setTagCreation] = useState<TagName | null>(null);

    const [{ isOver, canDrop, showTrash }, drop] = useDrop(() => ({
        accept: [
            ItemTypes.FILTER_SECTION_LABEL,
            ItemTypes.INTERACTIVE_TABLE_LABEL,
        ],
        drop: (args, monitor) => {
            const kind = monitor.getItemType();
            if (kind == ItemTypes.FILTER_SECTION_LABEL) {
                const item = args as TagView;
                interactive_list_create_tag(
                    props.entry.context_id,
                    props.entry.entry.metadata_id,
                    item.name_id,
                    item.value,
                ).then(() => {
                    console.log("filter section label");
                });
            } else if (kind == ItemTypes.INTERACTIVE_TABLE_LABEL) {
                const item = args as MetadataTagView;
                interactive_list_create_tag(
                    props.entry.context_id,
                    props.entry.entry.metadata_id,
                    item.tag_name_id,
                    item.tag_value,
                ).then(() => {
                    console.log("interactive table label");
                });
            }
        },
        collect: (monitor) => ({
            isOver: !!monitor.isOver({ shallow: true }),
            canDrop: !!monitor.canDrop(),
            showTrash: monitor.canDrop() &&
                monitor.getItemType() == ItemTypes.INTERACTIVE_TABLE_LABEL &&
                monitor.getItem<MetadataTagView>().metadata_id ==
                    props.entry.entry.metadata_id,
        }),
    }), []);

    const handleAddTag = (selectedTag: TagName) => {
        setTagCreation(selectedTag);
    };

    const static_tag_components = useMemo(() => {
        if (showTrash) {
            return [<TagSectionTrashZone key="tag_section_trash_zone" />];
        } else if (tagCreation) {
            return [
                <IndeterminateTagComponent
                    tagName={tagCreation}
                    entry={props.entry}
                    key={`indeterminate_${tagCreation.id}`}
                    onBlur={() => setTagCreation(null)}
                />,
            ];
        } else {
            return [
                <ButtonAddTag
                    key="button_add_tag"
                    allTagNames={props.allTagNames}
                    onAddTag={handleAddTag}
                />,
            ];
        }
    }, [tagCreation, showTrash]);

    const tag_components = useMemo(
        () =>
            props.entry.entry.tags.filter((tag) => {
                return tag.tag_name_id != TAG_NAME_ID_FILE_NAME &&
                    tag.tag_name_id != TAG_NAME_ID_TITLE &&
                    tag.tag_name_id != TAG_NAME_ID_DIRECTORY;
            }).map((tag) => (
                <InteractiveTableLabel
                    allTagNames={props.allTagNames}
                    key={`${tag.tag_name_id}_${tag.tag_id}`}
                    tag={tag}
                />
            )),
        [props.entry.entry.tags],
    );

    const all_tag_components = useMemo(
        () => tag_components.concat(static_tag_components),
        [static_tag_components, tag_components],
    );

    return (
        <DropZoneDecorator
            dropZoneRef={drop}
            isOver={isOver}
            canDrop={canDrop}
            className="w-full flex-wrap flex flex-row gap-2 p-1"
        >
            {all_tag_components}
        </DropZoneDecorator>
    );
}

function TagSectionTrashZone() {
    const [{ isOver }, drop] = useDrop(() => ({
        accept: [ItemTypes.INTERACTIVE_TABLE_LABEL],
        drop: (_args, _monitor) => {
        },
        collect: (monitor) => ({
            isOver: !!monitor.isOver(),
        }),
    }), []);

    return (
        <DropZoneDecorator dropZoneRef={drop} isOver={isOver} canDrop={true}>
            <Label className="h-11 w-11 flex items-center justify-center">
                <IconTrash />
            </Label>
        </DropZoneDecorator>
    );
}

function ButtonAddTag(
    props: { allTagNames: TagName[]; onAddTag: (selectedTag: TagName) => void },
) {
    const [showDropdown, setShowDropdown] = useState(false);

    const handleClose = (selectedTag: TagName | null) => {
        setShowDropdown(false);
        if (selectedTag) {
            props.onAddTag(selectedTag);
        }
    };

    return (
        <div>
            <div className="relative">
                <Label
                    key="button_add_tag"
                    className="flex h-11 flex-col cursor-pointer items-center justify-center"
                    onClick={() => setShowDropdown(!showDropdown)}
                >
                    <span className="text-slate-400 text-l truncate block">
                        +
                    </span>
                </Label>
                {showDropdown && (
                    <ButtonAddTagDropdown
                        allTagNames={props.allTagNames}
                        onClose={handleClose}
                    />
                )}
            </div>
        </div>
    );
}

function ButtonAddTagDropdown(
    props: {
        allTagNames: TagName[];
        onClose: (selectedTag: TagName | null) => void;
    },
) {
    const drop_down_buttons = props.allTagNames.filter((x) =>
        x.use_for_filtering
    ).map((tag) => (
        <Button
            variant="outline"
            key={tag.id}
            onClick={() => props.onClose(tag)}
        >
            <span className="text-slate-400 text-left w-full">{tag.name}</span>
        </Button>
    ));

    return (
        <DropDown onClose={() => props.onClose(null)}>
            {drop_down_buttons}
        </DropDown>
    );
}

function IndeterminateTagComponent(
    props: { tagName: TagName; entry: EntryViewCursor; onBlur: () => void },
) {
    const ref = useRef<HTMLDivElement>(null);
    const [value, setValue] = useState<string>("");
    const [suggestions, setSuggestions] = useState<string[]>([]);

    useClickOutside(ref, props.onBlur);

    const handleSubmit = () => {
        if (value) {
            interactive_list_create_tag(
                props.entry.context_id,
                props.entry.entry.metadata_id,
                props.tagName.id,
                value,
            ).then(() => {
                props.onBlur();
            });
        } else {
            props.onBlur();
        }
    };

    useEffect(() => {
        if (!value) {
            return;
        }
        interactive_list_get_tag_creation_suggestions(
            props.entry.context_id,
            props.tagName.id,
            value,
        ).then((suggestions) => {
            setSuggestions(suggestions);
        });
    }, [value]);

    return (
        <div ref={ref}>
            <Label className="relative flex flex-col outline-dashed outline-3 outline-yellow-800">
                <span className="leading-3 text-slate-400 text-xs truncate block">
                    {props.tagName.name}
                </span>
                <TextInput
                    className="bg-transparent"
                    suggestedValues={suggestions}
                    autoFocus={true}
                    onSubmit={handleSubmit}
                    onChange={(value) => setValue(value)}
                    value={value}
                />
            </Label>
        </div>
    );
}

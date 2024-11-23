import { useEffect, useMemo, useRef, useState } from "react";
import { TableRow } from "../molecules/Table";
import { Label } from "../atoms/Label";
import { DropDown } from "../molecules/DropDown";
import { Button } from "../atoms/Button";
import { useClickOutside } from "../../hooks/ClickOutside";
import { TextInput } from "../atoms/TextInput";
import { EntryViewCursor, interactive_list_create_tag, interactive_list_get_tag_creation_suggestions, MetadataTagView, TAG_NAME_ID_FILE_NAME, TAG_NAME_ID_TITLE, TagName } from "../../selectia-rs";

export function InteractiveTableRow(props: { entry: EntryViewCursor, allTagNames: TagName[] }) {
    const title = props.entry.title();

    const title_component = <div>
        <p className="text-slate-400 text-lg truncate block">{title}</p>
    </div>;

    const [tagCreation, setTagCreation] = useState<TagName | null>(null);

    const handleAddTag = (selectedTag: TagName) => {
        setTagCreation(selectedTag);
    }

    const static_tag_components = useMemo(() => [
        <ButtonAddTag key="button_add_tag" allTagNames={props.allTagNames} onAddTag={handleAddTag} />
    ].concat(tagCreation ? [
        <IndeterminateTagComponent tagName={tagCreation} entry={props.entry} key={`indeterminate_${tagCreation.id}`} onBlur={() => setTagCreation(null)} />
    ] : []), [tagCreation]);


    const tag_components = useMemo(() => props.entry.entry.tags.filter((tag) => {
        return tag.tag_name_id != TAG_NAME_ID_FILE_NAME &&
            tag.tag_name_id != TAG_NAME_ID_TITLE;
    }).map((tag) => (
        <TagComponent allTagNames={props.allTagNames} key={`${tag.tag_name_id}_${tag.tag_id}`} tag={tag} />
    ))
        , [props.entry.entry.tags]);

    const all_tag_components = useMemo(() => static_tag_components.concat(tag_components), [static_tag_components, tag_components]);

    return <TableRow title_component={title_component} tag_components={all_tag_components} />;
}

function ButtonAddTag(props: { allTagNames: TagName[], onAddTag: (selectedTag: TagName) => void }) {
    const [showDropdown, setShowDropdown] = useState(false);

    const handleClose = (selectedTag: TagName | null) => {
        setShowDropdown(false);
        if (selectedTag) {
            props.onAddTag(selectedTag);
        }
    }

    return <div>
        <div className="relative">
            <Label key="button_add_tag" className="cursor-pointer" onClick={() => setShowDropdown(!showDropdown)} >+</Label>
            {
                showDropdown && (<ButtonAddTagDropdown allTagNames={props.allTagNames} onClose={handleClose} />)
            }
        </div>
    </div>;
}

function ButtonAddTagDropdown(props: { allTagNames: TagName[], onClose: (selectedTag: TagName | null) => void }) {
    const drop_down_buttons = props.allTagNames.map((tag) => (
        <Button variant="outline" key={tag.id} onClick={() => props.onClose(tag)}>
            <span className="text-slate-400 text-left w-full">{tag.name}</span>
        </Button >
    ));

    return (
        <DropDown onClose={() => props.onClose(null)}>
            {drop_down_buttons}
        </DropDown>
    );
}

function TagComponent(props: { allTagNames: TagName[], tag: MetadataTagView }) {
    const tag_name = props.allTagNames.find((tag_name) => tag_name.id == props.tag.tag_name_id)?.name;

    return (
        <Label className="flex flex-col">
            <span className="leading-3 text-slate-400 text-xs truncate block">{tag_name}</span>
            <span className="text-slate-400 text-md truncate block">{props.tag.tag_value}</span>
        </Label>
    );
}

function IndeterminateTagComponent(props: { tagName: TagName, entry: EntryViewCursor, onBlur: () => void }) {
    const ref = useRef<HTMLDivElement>(null);
    const [value, setValue] = useState<string>("");
    const [suggestions, setSuggestions] = useState<string[]>([]);

    useClickOutside(ref, props.onBlur);

    const handleSubmit = () => {
        if (value) {
            interactive_list_create_tag(props.entry.context_id, props.entry.entry.metadata_id, props.tagName.id, value).then(props.onBlur);
        } else {
            props.onBlur();
        }
    }

    useEffect(() => {
        if (!value) {
            return;
        }
        interactive_list_get_tag_creation_suggestions(props.entry.context_id, props.tagName.id, value).then((suggestions) => {
            setSuggestions(suggestions);
        });
    }, [value]);


    return (<div ref={ref}>
        <Label className="relative flex flex-col outline-dashed outline-3 outline-yellow-800">
            <span className="leading-3 text-slate-400 text-xs truncate block">{props.tagName.name}</span>
            <TextInput suggestedValues={suggestions} autoFocus={true} onSubmit={handleSubmit} onChange={(value) => setValue(value)} value={value} />
        </Label>
    </div>);
}

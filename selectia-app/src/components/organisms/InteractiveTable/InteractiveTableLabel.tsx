import { TagName } from "../../../selectia-rs";

import { MetadataTagView } from "../../../selectia-rs";
import { Button } from "../../atoms/Button";
import { Label } from "../../atoms/Label";
import { useDrag } from 'react-dnd'
import { ItemTypes } from "../../pages/ManagerPage";

export function InteractiveTableLabel(props: { allTagNames: TagName[], tag: MetadataTagView }) {
    const tag_name = props.allTagNames.find((tag_name) => tag_name.id == props.tag.tag_name_id)?.name;

    const [{ opacity }, dragRef] = useDrag(
        () => ({
            type: ItemTypes.INTERACTIVE_TABLE_LABEL,
            item: props.tag,
            collect: (monitor) => ({
                opacity: monitor.isDragging() ? 0.5 : 1
            })
        }),
        []
    )

    return (
        <Label innerRef={dragRef} className="flex flex-col cursor-pointer" style={{ opacity }}>
            <span className="leading-3 text-slate-400 text-xs truncate block">{tag_name}</span>
            <span className="text-slate-400 text-md truncate block">{props.tag.tag_value}</span>
        </Label>
    );
}

function TagContextualMenu(props: { tag: MetadataTagView }) {
    return <div>
        <Button variant="outline">-</Button>
    </div>;
}
import { ConnectDropTarget } from "react-dnd";

const DropZoneDecoratorClasses = {
    "isOver": "outline outline outline-2 outline-green-400",
    "canDrop": "outline outline-dashed outline-1 outline-gray-400",
};

export function DropZoneDecorator(
    { isOver, canDrop, dropZoneRef, ...props }:
        & { isOver: boolean; canDrop: boolean; dropZoneRef: ConnectDropTarget }
        & React.ComponentPropsWithoutRef<"div">,
) {
    const className = isOver
        ? DropZoneDecoratorClasses.isOver
        : canDrop
        ? DropZoneDecoratorClasses.canDrop
        : "";

    return <div {...props} className={`${className} ${props.className}`} ref={dropZoneRef} />;
}

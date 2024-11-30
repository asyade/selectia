import { ConnectDragSource } from "react-dnd";

interface LabelProps {
    bgColor?: { r: number; g: number; b: number; opacity: number };
    selectable?: boolean;
    selected?: boolean;
    dragRef?: ConnectDragSource;
}

export function Label({bgColor, selectable, selected, dragRef, ...props}: LabelProps & React.ComponentPropsWithoutRef<"div">) {
    const color = bgColor
        ? { ...bgColor, opacity: selected ? 0.8 : 0.3 }
        : { r: 31, g: 41, b: 55, opacity: selected ? 0.8 : 0.3 };

    const style = {
        ...props.style,
        backgroundColor:
            `rgba(${color.r}, ${color.g}, ${color.b}, ${color.opacity})`,
    };

    const className = `${selected ? "outline-2 outline-dashed" : ""} p-1 rounded text-sm/2 text-white cursor-pointer ${props.className}`;
    return (
        <div {...props} style={style} className={className} ref={dragRef} />
    );
}

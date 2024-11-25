import { LegacyRef } from "react";

interface LabelProps {
    className?: string;
    children: React.ReactNode;
    selectable?: boolean;
    selected?: boolean;
    style?: React.CSSProperties;
    onClick?: () => void;
    innerRef?: LegacyRef<HTMLDivElement>;
}

export function Label(props: LabelProps) {
    return <div onClick={props.onClick} ref={props.innerRef} style={props.style} className={`p-1 rounded text-sm/2 text-white ${props.className} ${props.selectable ? "cursor-pointer" : ""} ${props.selected ? "bg-green-700/50" : "bg-green-800/10"}`}>{props.children}</div>;
}

import { useRef } from "react";
import { useClickOutside } from "../../hooks/ClickOutside";

export function DropDown(props: {
    className?: string;
    children: React.ReactNode[] | React.ReactNode;
    onClose: () => void;
}) {
    const ref = useRef<HTMLDivElement>(null);
    useClickOutside(ref, props.onClose);

    return (
        <div ref={ref} className={`${props.className} z-10 bg-slate-800 rounded flex flex-col absolute shadow-lg border border-slate-700`}>
            {props.children}
        </div>
    );
}

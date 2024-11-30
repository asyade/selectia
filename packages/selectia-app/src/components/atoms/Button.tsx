export type ButtonVariant = "primary" | "outline" | "ghost";

export interface ButtonProps {
    variant?: ButtonVariant;
    children: React.ReactNode;
    onClick?: () => void;
    className?: string;
}

const baseClass = "flex items-center justify-center rounded";
const classNames = {
    "outline": `${baseClass} p-1 hover:bg-slate-700`,
    "primary": `${baseClass} p-1 hover:bg-blue-700 bg-blue-500`,
    "ghost": `${baseClass} hover:bg-slate-700`,
};

export function Button(props: ButtonProps) {
    return <button onMouseDown={(e) => e.preventDefault()} className={`${props.className} ${classNames[props.variant ?? "primary"]}`} onClick={() => props.onClick?.()}>{props.children}</button>;
}

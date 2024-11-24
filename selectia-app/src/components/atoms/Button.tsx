export type ButtonVariant = "primary" | "outline";

export interface ButtonProps {
    variant?: ButtonVariant;
    children: React.ReactNode;
    onClick?: () => void;
    className?: string;
}

export function Button(props: ButtonProps) {
    if (props.variant === "outline") {
        return <button className={`${props.className} p-2 flex items-center justify-center rounded hover:bg-slate-700`} onClick={() => props.onClick?.()}>{props.children}</button>;
    } else {
        return <button className={`${props.className} p-2 flex items-center justify-center rounded hover:bg-blue-700 bg-blue-500`} onClick={() => props.onClick?.()}>{props.children}</button>;
    }
}
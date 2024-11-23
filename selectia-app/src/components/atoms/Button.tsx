interface ButtonProps {
    variant?: "primary" | "outline";
    children: React.ReactNode;
    onClick?: () => void;
    className?: string;
}

export function Button(props: ButtonProps) {
    if (props.variant === "outline") {
        return <button className={`p-2 flex items-center justify-center rounded hover:bg-slate-700 ${props.className}`} onClick={() => props.onClick?.()}>{props.children}</button>;
    } else {
        return <button className={`p-2 flex items-center justify-center rounded hover:bg-blue-700 bg-blue-500 ${props.className}`} onClick={() => props.onClick?.()}>{props.children}</button>;
    }
}
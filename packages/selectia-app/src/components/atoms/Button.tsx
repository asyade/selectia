export type ButtonVariant = "primary" | "outline" | "ghost";

export interface ButtonProps {
    variant?: ButtonVariant;
}

const baseClass = "flex items-center justify-center rounded";
const classNames = {
    "outline": `${baseClass} p-1 hover:bg-slate-700`,
    "primary": `${baseClass} p-1 hover:bg-blue-700 bg-blue-500`,
    "ghost": `${baseClass} hover:bg-slate-700`,
};

export function Button({ variant, ...props }: ButtonProps & React.ComponentPropsWithoutRef<"button">) {
    const className = `${props.className} ${classNames[variant ?? "primary"]}`;

    // Prevent the button from being dragged when clicking on it (that cause issue when used in header actions but maybe scoped to that in future)
    const onMouseDown = props.onMouseDown ?? ((e: React.MouseEvent<HTMLButtonElement>) => {
        e.preventDefault();
    });
    return <button onMouseDown={onMouseDown} className={className} {...props} />;
}

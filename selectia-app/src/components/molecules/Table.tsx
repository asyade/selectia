export function Table(props: {children: React.ReactNode[]}) {
    return (
        <div className="bg-slate-900 p-2">
            <div className="flex flex-col">
                {props.children}
            </div>
        </div>
    );
}

export interface TableRowProps {
    title_component: React.ReactNode;
    tag_components: React.ReactNode[] | React.ReactNode;
    onClick?: () => void;
    className?: string;
    innerRef?: React.RefObject<HTMLDivElement>;
}

export function TableRow(props: TableRowProps) {
    return (
        <div onClick={() => props.onClick?.()} ref={props.innerRef} className={`flex flex-col ${props.className}`}>
            <div className="p-1 selectable">
                {props.title_component}
            </div>
            <div className={`flex flex-row gap-2`}>
                {props.tag_components}
            </div>
        </div>
    );
}
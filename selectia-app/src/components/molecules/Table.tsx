export function Table(props: {children: React.ReactNode[]}) {
    return (
        <div className="flex flex-col">
            {props.children}
        </div>
    );
}

export interface TableRowProps {
    title_component: React.ReactNode;
    tag_components: React.ReactNode[];
}

export function TableRow(props: TableRowProps) {
    return (
        <div className="flex flex-col">
            <div className="p-1">
                {props.title_component}
            </div>
            <div className="flex flex-row gap-2 pl-1 pr-1">
                {props.tag_components}
            </div>
        </div>
    );
}
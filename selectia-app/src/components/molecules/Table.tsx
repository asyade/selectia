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
            {props.title_component}
            <div className="flex flex-row">
                {props.tag_components}
            </div>
        </div>
    );
}
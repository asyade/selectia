export function Table(props: {children: React.ReactNode[]}) {
    return (
        <div className="p-2">
            <div className="flex flex-col">
                {props.children}
            </div>
        </div>
    );
}

export interface TableRowProps {
    expanded?: boolean;
    title_component: React.ReactNode;
    tag_components: React.ReactNode[] | React.ReactNode;
    body_component?: React.ReactNode;
    innerRef?: React.RefObject<HTMLDivElement>;
}

export function TableRow({expanded, title_component, tag_components, innerRef, body_component, ...props}: TableRowProps & React.ComponentPropsWithoutRef<"div">) {

    const className = `${props.className} flex flex-col rounded-md ${expanded ? "border-primary" : ""}`;

    return (
        <div {...props} ref={innerRef} className={className}>
            <div className="p-1 selectable">
                {title_component}
            </div>
            {expanded && body_component}
            <div className={`flex flex-row gap-2`}>
                {tag_components}
            </div>
        </div>
    );
}
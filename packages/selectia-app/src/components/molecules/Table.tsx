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
    title_component: React.ReactNode;
    tag_components: React.ReactNode[] | React.ReactNode;
    innerRef?: React.RefObject<HTMLDivElement>;
}

export function TableRow({title_component, tag_components, innerRef, ...props}: TableRowProps & React.ComponentPropsWithoutRef<"div">) {
    return (
        <div {...props} ref={innerRef} className={`flex flex-col ${props.className}`}>
            <div className="p-1 selectable">
                {title_component}
            </div>
            <div className={`flex flex-row gap-2`}>
                {tag_components}
            </div>
        </div>
    );
}
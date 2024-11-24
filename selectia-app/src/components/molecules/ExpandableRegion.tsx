import { useState } from "react";
import { Button } from "../atoms/Button";
import { IconChevronDown, IconChevronUp } from "../atoms/Icon";

export function ExpandableRegion(props: {
    className?: string;
    children: React.ReactNode;
    header: React.ReactNode;
}) {
    const [expanded, setExpanded] = useState(false);

    return <div className={`${props.className} flex flex-col`}>
        <div className="flex flex-row item-between">
            <div className="grow">
                {props.header}
            </div>
            <Button variant="outline" onClick={() => setExpanded(!expanded)}>
                {
                    expanded ? <IconChevronUp /> : <IconChevronDown />
                }
            </Button>
        </div>
        {expanded && props.children}
    </div>;
}
import { IPaneviewPanelProps } from "dockview-react";
import { IconChevronRight } from "../..";

export function PaneViewHeader(
    props: IPaneviewPanelProps & { actionComponents?: React.ReactNode },
) {
    const style = props.api.isExpanded ? "rotate-90" : "";

    return (
        <div
            className="flex flex-row justify-between gap-2 cursor-pointer items-center"
            onClick={() => props.api.setExpanded(!props.api.isExpanded)}
        >
            <div className="pl-1 flex flex-row gap-2 items-center">
                <IconChevronRight
                    color="rgb(209 213 219)"
                    className={`${style} w-2.5 h-2.5`}
                />
                <p className="text-sm text-gray-300">{props.title}</p>
            </div>
            <div className="flex flex-row gap-2">
                {props.actionComponents}
            </div>
        </div>
    );
}

import { IDockviewPanelProps } from "dockview-react";

export function DockViewHeader(
    props: IDockviewPanelProps & { actionComponents?: React.ReactNode },
) {
    return (
        <div
            className="flex flex-row justify-between gap-2 cursor-pointer items-center h-full w-full pl-3"
        >
            <p className="text-sm text-gray-300">{props.api.title}</p>
            <div className="flex flex-row gap-2">
                {props.actionComponents}
            </div>
        </div>
    );
}

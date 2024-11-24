import { useState } from "react";
import { DropDownButton } from "../molecules/DropDownButton";
import { IconTaskManager } from "../atoms/Icon";
import { TaskManager } from "./TaskManager";

export function Statusbar(props: {
    className?: string;
}) {
    const [status, _setStatus] = useState("Idle");

    return <div className={`${props.className} p-2 bg-slate-900`}>
        <div className="flex flex-row justify-between w-full items-center">
            <span className="text-slate-400">{status}</span>
            <DropDownButton buttonContent={<IconTaskManager />} variant="outline" className="relative" dropDownClassName="-right-full -top-full -mt-10 mr-10">
                <TaskManager />
            </DropDownButton>
        </div>
    </div>;
}
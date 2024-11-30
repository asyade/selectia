import { useState } from "react";
import { DropDownButton } from "../molecules/DropDownButton";
import { IconTaskManager } from "../atoms/Icon";
import { TaskManager } from "./TaskManager";
import { useWorkerQueueTasks } from "../../selectia-tauri/hooks/UseWorkerQueue";

export function Statusbar(props: {
    className?: string;
}) {
    const [status, _setStatus] = useState("Idle");

    const [tasks] = useWorkerQueueTasks();

    return <div className={`${props.className} p-2 bg-primary`}>
        <div className="flex flex-row justify-between w-full items-center">
            <span className="text-slate-400">{status}</span>
            <DropDownButton buttonContent={<IconTaskManager />} variant="outline" className="relative" dropDownClassName="-right-full -top-64 mr-10">
                <TaskManager tasks={tasks} />
            </DropDownButton>
        </div>
    </div>;
}
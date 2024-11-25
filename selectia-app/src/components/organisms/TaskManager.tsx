import { useMemo } from "react";
import { WorkerQueueTask } from "../../selectia-rs";

export function TaskManager(props: {
    tasks: WorkerQueueTask[];
}) {


    const task_elements = useMemo(() => props.tasks.map(task => (
        <div key={task.id} className="flex flex-row justify-between">
            <span>{task.id}</span>
            <span>{task.status}</span>
        </div>
    )), [props.tasks]);

    return (
        <div className="bg-slate-800 text-white p-4 rounded-md h-64 w-64 overflow-y-scroll">
            {task_elements.length > 0 ? task_elements : <span className="text-slate-400 text-center w-full">No tasks</span>}
        </div>
    );
}

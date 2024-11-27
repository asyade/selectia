import { useEffect, useState } from "react";
import {
    WorkerQueueTaskCreatedEvent,
    WorkerQueueTaskUpdatedEvent,
} from "../dto/events";
import { WorkerQueueTask } from "../dto/models";
import { get_worker_queue_task, get_worker_queue_tasks } from "../index";
import { useEvent } from "./UseEvent";

export function useWorkerQueueTasks(): [WorkerQueueTask[]] {
    const [tasks, setTasks] = useState<WorkerQueueTask[]>([]);

    useEvent<WorkerQueueTaskCreatedEvent>("WorkerQueueTaskCreated", (event) => {
        get_worker_queue_task(event.task.id).then((task) => {
            setTasks((prev) => [...prev, task]);
        });
    });

    useEvent<WorkerQueueTaskUpdatedEvent>("WorkerQueueTaskUpdated", (event) => {
        if (event.task) {
            setTasks((prev) =>
                prev.map((t) =>
                    t.id === event.id ? event.task as WorkerQueueTask : t
                )
            );
        } else {
            setTasks((prev) => prev.filter((t) => t.id !== event.id));
        }
    });

    useEffect(() => {
        get_worker_queue_tasks().then(setTasks);
    }, []);

    return [tasks];
}

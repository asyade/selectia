import { useState, useEffect } from "react";
import { WorkerQueueTask, WorkerQueueTaskCreatedEvent, WorkerQueueTaskUpdatedEvent } from "../models";
import { get_worker_queue_task, get_worker_queue_tasks } from "../index";
import { listen } from "@tauri-apps/api/event";

export function useWorkerQueueTasks(): [WorkerQueueTask[]] {
    const [tasks, setTasks] = useState<WorkerQueueTask[]>([]);

    useEffect(() => {
        const unlisten = listen("worker-queue-task-created", (event) => {
            const payload = event.payload as WorkerQueueTaskCreatedEvent;
            get_worker_queue_task(payload.id).then(task => {
                setTasks(prev => [...prev, task]);
            });
        });

        const unlisten2 = listen("worker-queue-task-updated", (event) => {
            const payload = event.payload as WorkerQueueTaskUpdatedEvent;
            if (payload.task) {
                setTasks(prev => prev.map(t => t.id === payload.id ? payload.task as WorkerQueueTask : t));
            } else {
                setTasks(prev => prev.filter(t => t.id !== payload.id));
            }
        });

        get_worker_queue_tasks().then(setTasks);
        return () => {
            unlisten.then(unlisten => unlisten());
            unlisten2.then(unlisten2 => unlisten2());
        };
    }, []);

    return [tasks]
}
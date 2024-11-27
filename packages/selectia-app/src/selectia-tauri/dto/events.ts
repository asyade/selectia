// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { DeckFileView } from "./models";
import type { WorkerQueueTask } from "./models";

export type AudioDeckCreatedEvent = { id: number, };

export type AudioDeckUpdatedEvent = { id: number, file: DeckFileView | null, };

export type Events = { "AudioDeckCreated": AudioDeckCreatedEvent } | { "AudioDeckUpdated": AudioDeckUpdatedEvent } | { "WorkerQueueTaskCreated": WorkerQueueTaskCreatedEvent } | { "WorkerQueueTaskUpdated": WorkerQueueTaskUpdatedEvent };

export type WorkerQueueTaskCreatedEvent = { task: WorkerQueueTask, };

export type WorkerQueueTaskUpdatedEvent = { id: bigint, task: WorkerQueueTask | null, };

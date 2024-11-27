// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { DeckFileView } from "./models";
import type { EntryView } from "./models";
import type { WorkerQueueTask } from "./models";

export type AudioDeckCreatedEvent = { id: number, };

export type AudioDeckUpdatedEvent = { id: number, file: DeckFileView | null, };

export type EntryChangedEvent = { entry: EntryView, };

export type EntryListChangedEvent = Record<string, never>;

export type Events = { "type": "AudioDeckCreated" } & AudioDeckCreatedEvent | { "type": "AudioDeckUpdated" } & AudioDeckUpdatedEvent | { "type": "WorkerQueueTaskCreated" } & WorkerQueueTaskCreatedEvent | { "type": "WorkerQueueTaskUpdated" } & WorkerQueueTaskUpdatedEvent | { "type": "TagListChanged" } & TagListChangedEvent | { "type": "EntryChanged" } & EntryChangedEvent | { "type": "EntryListChanged" } & EntryListChangedEvent;

export type TagListChangedEvent = Record<string, never>;

export type WorkerQueueTaskCreatedEvent = { task: WorkerQueueTask, };

export type WorkerQueueTaskUpdatedEvent = { id: bigint, task: WorkerQueueTask | null, };
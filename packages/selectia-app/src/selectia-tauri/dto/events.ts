// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { DeckFileMetadataSnapshot } from "./models";
import type { DeckFilePayloadSnapshot } from "./models";
import type { DeckFileStatus } from "./models";
import type { EntryView } from "./models";
import type { WorkerQueueTask } from "./models";

export type AudioDeckCreatedEvent = { id: number, };

export type AudioDeckFileMetadataUpdatedEvent = { id: number, metadata: DeckFileMetadataSnapshot, };

export type AudioDeckFilePayloadUpdatedEvent = { id: number, payload: DeckFilePayloadSnapshot, };

export type AudioDeckFileStatusUpdatedEvent = { id: number, status: DeckFileStatus, };

export type EntryChangedEvent = { entry: EntryView, };

export type EntryListChangedEvent = Record<string, never>;

export type Events = { "type": "AudioDeckFileMetadataUpdated" } & AudioDeckFileMetadataUpdatedEvent | { "type": "AudioDeckFilePayloadUpdated" } & AudioDeckFilePayloadUpdatedEvent | { "type": "AudioDeckFileStatusUpdated" } & AudioDeckFileStatusUpdatedEvent | { "type": "AudioDeckCreated" } & AudioDeckCreatedEvent | { "type": "WorkerQueueTaskCreated" } & WorkerQueueTaskCreatedEvent | { "type": "WorkerQueueTaskUpdated" } & WorkerQueueTaskUpdatedEvent | { "type": "TagListChanged" } & TagListChangedEvent | { "type": "EntryChanged" } & EntryChangedEvent | { "type": "EntryListChanged" } & EntryListChangedEvent;

export type TagListChangedEvent = Record<string, never>;

export type WorkerQueueTaskCreatedEvent = { task: WorkerQueueTask, };

export type WorkerQueueTaskUpdatedEvent = { id: bigint, task: WorkerQueueTask | null, };

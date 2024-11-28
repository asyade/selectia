// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

export type AppError = { message: string, id: number, };

export type ContextId = bigint;

export type DeckFileMetadataSnapshot = { title: string, };

export type DeckFilePayloadSnapshot = { duration: number, sample_rate: number, channels_count: number, samples_count: number, };

export type DeckFileStatus = { "kind": "Loading", progress: number, } | { "kind": "Playing", offset: number, } | { "kind": "Paused", offset: number, };

export type DeckFileView = { metadata: DeckFileMetadataSnapshot, payload: DeckFilePayloadSnapshot, status: DeckFileStatus, };

export type DeckView = { file: DeckFileView | null, id: number, };

export type EntryView = { metadata_id: bigint, metadata_hash: string, tags: Array<MetadataTagView>, };

export type FilterSelection = { directories: Array<string>, tags: { [key in number]?: Array<TagSelection> }, };

export type MetadataTagView = { tag_id: bigint, metadata_tag_id: bigint, tag_name_id: bigint, tag_value: string, metadata_id: bigint, };

export type Models = { "DeckFileMetadataSnapshot": DeckFileMetadataSnapshot } | { "DeckFilePayloadSnapshot": DeckFilePayloadSnapshot } | { "DeckFileStatus": DeckFileStatus } | { "AppError": AppError } | { "ContextId": ContextId } | { "WorkerQueueTask": WorkerQueueTask } | { "TaskStatus": TaskStatus } | { "DeckView": DeckView } | { "DeckFileView": DeckFileView } | { "TagSelection": TagSelection } | { "FilterSelection": FilterSelection } | { "EntryView": EntryView } | { "MetadataTagView": MetadataTagView } | { "TagName": TagName } | { "TagView": TagView };

export type TagName = { id: bigint, name: string, use_for_filtering: boolean, };

export type TagSelection = { id: bigint, value: string, selected: boolean, };

export type TagView = { id: bigint, value: string, name_id: bigint, };

export type TaskStatus = "Queued" | "Processing" | "Done";

export type WorkerQueueTask = { id: bigint, status: TaskStatus, };

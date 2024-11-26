
export const TAG_NAME_ID_FILE_NAME_EMBEDDING = 1;
export const TAG_NAME_ID_DIRECTORY = 2;
export const TAG_NAME_ID_FILE_NAME = 3;
export const TAG_NAME_ID_TITLE = 4;
export const TAG_NAME_ID_ARTIST = 5;
export const TAG_NAME_ID_ALBUM = 6;
export const TAG_NAME_ID_GENRE = 7;

export interface TagSelection {
    id: number;
    value: string;
    selected: boolean;
}
export interface TagsSelection {
    [key: number]: TagSelection[];
}
export interface FilterSelection {
    directories: string[];
    tags: TagsSelection;
}

export interface EntryView {
    metadata_id: number,
    metadata_hash: string,
    tags: MetadataTagView[],
}

export interface MetadataTagView {
    tag_id: number,
    metadata_tag_id: number,
    tag_name_id: number,
    tag_value: string,
    metadata_id: number,
}

export interface TagName {
    id: number;
    name: string;
    use_for_filtering: boolean;
}

export interface TagView {
    id: number;
    value: string;
    name_id: number;
}

export interface WorkerQueueTask {
    id: number;
    status: string;
}

export type WorkerQueueTaskCreatedEvent = {
    task: WorkerQueueTask;
}

export type WorkerQueueTaskUpdatedEvent = {
    id: number;
    task?: WorkerQueueTask;
}


export type ContextId = string;
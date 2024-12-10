
export const TAG_NAME_ID_FILE_NAME_EMBEDDING = BigInt(1);
export const TAG_NAME_ID_DIRECTORY = BigInt(2);
export const TAG_NAME_ID_FILE_NAME = BigInt(3);
export const TAG_NAME_ID_TITLE = BigInt(4);
export const TAG_NAME_ID_ARTIST = BigInt(5);
export const TAG_NAME_ID_ALBUM = BigInt(6);
export const TAG_NAME_ID_GENRE = BigInt(7);

import { invoke } from "@tauri-apps/api/core";
import { ContextId, FilterSelection, EntryView, TagView, TagName, WorkerQueueTask, DeckView, DeckFileStatus, FileVariation } from "./dto/models";

export async function interactive_list_create_context(): Promise<bigint> {
    return await invoke("interactive_list_create_context").then((x: any) => x as bigint);
}

export async function interactive_list_delete_context(contextId: ContextId) {
    return await invoke("interactive_list_delete_context", { contextId });
}

export async function interactive_list_get_tag_creation_suggestions(contextId: ContextId, tagNameId: bigint, input: string): Promise<string[]> {
    return await invoke("interactive_list_get_tag_creation_suggestions", { contextId, tagNameId, input }).then((x: any) => x as string[]);
}

export async function interactive_list_create_tag(contextId: ContextId, metadataId: bigint, nameId: bigint, value: string) {
    return await invoke("interactive_list_create_tag", { contextId, metadataId, nameId, value });
}

export async function get_interactive_list_context_entries(contextId: ContextId, filter: FilterSelection): Promise<EntryView[]> {
    return await invoke("get_interactive_list_context_entries", { contextId, filter }).then((x: any) => x as EntryView[]);
}

export async function get_tags_by_name(tagName: string): Promise<TagView[]> {
    return await invoke("get_tags_by_name", { tagName }).then((x: any) => x as TagView[]);
}

export async function get_tag_names(): Promise<TagName[]> {
    return await invoke("get_tag_names").then((x: any) => x as TagName[]);
}

export async function import_folder(directory: string): Promise<boolean> {
    return await invoke("import_folder", { directory }).then((x: any) => x as boolean);
}

export async function get_worker_queue_tasks(): Promise<WorkerQueueTask[]> {
    return await invoke("get_worker_queue_tasks").then((x: any) => x as WorkerQueueTask[]);
}

export async function get_worker_queue_task(taskId: bigint): Promise<WorkerQueueTask> {
    return await invoke("get_worker_queue_task", { taskId }).then((x: any) => x as WorkerQueueTask);
}

export async function create_audio_deck(): Promise<bigint> {
    return await invoke("create_audio_deck").then((x: any) => x as bigint);
}

export async function get_audio_decks(): Promise<DeckView[]> {
    return await invoke("get_audio_decks").then((x: any) => x as DeckView[]);
}

export async function load_audio_track_from_metadata(deckId: bigint, metadataId: bigint) {
    return await invoke("load_audio_track_from_metadata", { deckId, metadataId });
}

export async function load_audio_track_from_variation(deckId: bigint, fileVariationId: bigint) {
    return await invoke("load_audio_track_from_variation", { deckId, fileVariationId });
}

export async function set_deck_file_status(deckId: bigint, status: DeckFileStatus) {
    return await invoke("set_deck_file_status", { deckId, status });
}

export async function extract_stems(metadataId: bigint) {
    return await invoke("extract_stems", { metadataId });
}

export async function get_file_variations_for_metadata(metadataId: bigint): Promise<FileVariation[]> {
    return await invoke("get_file_variations_for_metadata", { metadataId }).then((x: any) => x as FileVariation[]);
}

export interface EntryVariationCursor {
    entry: EntryView;
    variation: FileVariation;
}

export class EntryViewCursor {
    entry: EntryView;
    context_id: ContextId;

    constructor(entry: EntryView, context_id: ContextId) {
        this.entry = entry;
        this.context_id = context_id;
    }

    uid(): string {
        return this.entry.metadata_id.toString();
    }

    tagValue(entry: EntryView, tag_name_id: bigint): string | undefined {
        return entry.tags.find((tag) => tag.tag_name_id == tag_name_id)?.tag_value;
    }

    tagValues(entry: EntryView, tag_name_id: bigint): string[] {
        return entry.tags.filter((tag) => tag.tag_name_id == tag_name_id).map((tag) => tag.tag_value);
    }

    title() {
        let title = this.tagValue(this.entry, TAG_NAME_ID_TITLE);
        if (!title) {
            title = this.tagValue(this.entry, TAG_NAME_ID_FILE_NAME);
        }
        if (!title) {
            title = this.entry.metadata_hash;
        }
        return title;
    }
}


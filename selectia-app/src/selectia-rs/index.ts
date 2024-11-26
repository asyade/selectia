import { invoke } from "@tauri-apps/api/core";
import { ContextId, EntryView, FilterSelection, TAG_NAME_ID_FILE_NAME, TAG_NAME_ID_TITLE, TagName, TagView, WorkerQueueTask } from "./models";

export * from "./models";

export async function interactive_list_create_context(): Promise<ContextId> {
    return await invoke("interactive_list_create_context").then(x => x as ContextId);
}

export async function interactive_list_delete_context(contextId: ContextId) {
    return await invoke("interactive_list_delete_context", { contextId });
}

export async function interactive_list_get_tag_creation_suggestions(contextId: ContextId, tagNameId: number, input: string): Promise<string[]> {
    return await invoke("interactive_list_get_tag_creation_suggestions", { contextId, tagNameId, input }).then(x => x as string[]);
}

export async function interactive_list_create_tag(contextId: ContextId, metadataId: number, nameId: number, value: string) {
    return await invoke("interactive_list_create_tag", { contextId, metadataId, nameId, value });
}

export async function get_interactive_list_context_entries(contextId: ContextId, filter: FilterSelection): Promise<EntryView[]> {
    return await invoke("get_interactive_list_context_entries", { contextId, filter }).then(x => x as EntryView[]);
}

export async function get_tags_by_name(tagName: string): Promise<TagView[]> {
    return await invoke("get_tags_by_name", { tagName }).then(x => x as TagView[]);
}

export async function get_tag_names(): Promise<TagName[]> {
    return await invoke("get_tag_names").then(x => x as TagName[]);
}

export async function import_folder(directory: string): Promise<boolean> {
    return await invoke("import_folder", { directory }).then(x => x as boolean);
}

export async function get_worker_queue_tasks(): Promise<WorkerQueueTask[]> {
    return await invoke("get_worker_queue_tasks").then(x => x as WorkerQueueTask[]);
}

export async function get_worker_queue_task(taskId: number): Promise<WorkerQueueTask> {
    return await invoke("get_worker_queue_task", { taskId }).then(x => x as WorkerQueueTask);
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

    tagValue(entry: EntryView, tag_name_id: number): string | undefined {
        return entry.tags.find((tag) => tag.tag_name_id == tag_name_id)?.tag_value;
    }

    tagValues(entry: EntryView, tag_name_id: number): string[] {
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


import { useEffect, useState } from "react";
import { TagView } from "../dto/models";
import { get_tags_by_name } from "../index";
import { useEvent } from "./UseEvent";
import { TagListChangedEvent } from "../dto/events";

export function useTags(name: string, auto_update: boolean = false) {
    const [tags, setTags] = useState<TagView[]>([]);
    
    useEffect(() => {
        get_tags_by_name(name).then(setTags);
    }, [name]);

    if (auto_update) {
        useEvent<TagListChangedEvent>("TagListChanged", () => {
            get_tags_by_name(name).then(setTags);
        });
    }

    return [tags];
}

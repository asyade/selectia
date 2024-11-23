import { useEffect, useState } from "react";
import { TagView } from "../selectia-rs/models";
import { get_tags_by_name } from "../selectia-rs";

export function useTags(name: string) {
    const [tags, setTags] = useState<TagView[]>([]);
    useEffect(() => {
        get_tags_by_name(name).then(setTags);
    }, [name]);

    return [tags];
}
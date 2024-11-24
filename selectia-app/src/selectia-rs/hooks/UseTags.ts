import { useEffect, useState } from "react";
import { TagView } from "../models";
import { get_tags_by_name } from "..";

export function useTags(name: string) {
    const [tags, setTags] = useState<TagView[]>([]);
    useEffect(() => {
        get_tags_by_name(name).then(setTags);
    }, [name]);

    return [tags];
}
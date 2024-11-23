import { useEffect, useState } from "react";
import { invoke } from '@tauri-apps/api/core';

interface TagView {
    id: number;
    value: string;
}

export function useTags(name: string) {
    const [tags, setTags] = useState<TagView[]>([]);
    useEffect(() => {
        invoke("get_tags_by_name", { tagName: name }).then(x => setTags(x as TagView[]));
    }, []);

    return [tags];
}
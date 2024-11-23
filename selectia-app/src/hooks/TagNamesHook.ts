import { useEffect, useState } from "react";
import { invoke } from '@tauri-apps/api/core';

interface TagName {
    id: number;
    name: string;
    use_for_filtering: boolean;
}

export function useTagNames() {
    const [tagNames, setTagNames] = useState<TagName[]>([]);
 
    useEffect(() => {
        invoke("get_tag_names").then(x => setTagNames(x as TagName[]));
    }, []);

    return [tagNames];
}
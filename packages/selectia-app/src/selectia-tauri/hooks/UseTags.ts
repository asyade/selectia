import { useEffect, useState } from "react";
import { TagView } from "../dto/models";
import { get_tags_by_name } from "../index";
import { listen } from "@tauri-apps/api/event";

export function useTags(name: string, auto_update: boolean = false) {
    const [tags, setTags] = useState<TagView[]>([]);
    useEffect(() => {
        get_tags_by_name(name).then(setTags);
    }, [name]);

    useEffect(() => {
        if (!auto_update) {
            return;
        }
        const unlisten = listen("tag_list_changed", () => {
            console.log("tag_list_changed");
            get_tags_by_name(name).then(setTags);
        });
        return () => {
            unlisten.then(fn => fn());
        };
    }, []);


    return [tags];
}

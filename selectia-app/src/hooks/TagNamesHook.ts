import { useEffect, useState } from "react";
import { TagName } from "../selectia-rs/models";
import { get_tag_names } from "../selectia-rs";


export function useTagNames() {
    const [tagNames, setTagNames] = useState<TagName[]>([]);
 
    useEffect(() => {
        get_tag_names().then(setTagNames);
    }, []);

    return [tagNames];
}
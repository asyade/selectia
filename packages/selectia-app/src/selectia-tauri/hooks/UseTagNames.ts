import { useEffect, useState } from "react";
import { TagName } from "../dto/models";
import { get_tag_names } from "../index";

export function useTagNames() {
    const [tagNames, setTagNames] = useState<TagName[]>([]);
 
    useEffect(() => {
        get_tag_names().then(setTagNames);
    }, []);

    return [tagNames];
}
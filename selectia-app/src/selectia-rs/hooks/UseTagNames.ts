import { useEffect, useState } from "react";
import { TagName } from "../models";
import { get_tag_names } from "..";

export function useTagNames() {
    const [tagNames, setTagNames] = useState<TagName[]>([]);
 
    useEffect(() => {
        get_tag_names().then(setTagNames);
    }, []);

    return [tagNames];
}
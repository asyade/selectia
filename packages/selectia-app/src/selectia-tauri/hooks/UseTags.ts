import { useEffect, useState } from "react";
import { TagView } from "../dto/models";
import { get_tags_by_name } from "../index";
import { useEvent } from "./UseEvent";
import { TagListChangedEvent } from "../dto/events";

export const TAG_COLORS = [
    [   // Red
        {r: 239,g: 68,b: 68},
        {r: 220,g: 38,b: 38},
        {r: 185,g: 28,b: 28},
        {r: 153,g: 27,b: 27},
        {r: 69,g: 10,b: 10},
    ],
    [   // Orange
        {r: 249,g: 115,b: 22},
        {r: 234,g: 88,b: 12},
        {r: 194,g: 65,b: 12},
        {r: 154,g: 52,b: 18},
        {r: 67,g: 20,b: 7},
    ],
    [   // Lime
        {r: 132,g: 204,b: 22},
        {r: 101,g: 163,b: 13},
        {r: 77,g: 124,b: 15},
        {r: 63,g: 98,b: 18},
        {r: 26,g: 39,b: 11},
    ],
    [   // Cyan
        {r: 6,g: 182,b: 212},
        {r: 8,g: 145,b: 178},
        {r: 14,g: 116,b: 144},
        {r: 21,g: 94,b: 117},
        {r: 8,g: 51,b: 68},
    ],
    [   // Purple
        {r: 168,g: 85,b: 247},
        {r: 147,g: 51,b: 234},
        {r: 126,g: 34,b: 206},
        {r: 107,g: 33,b: 168},
        {r: 59,g: 7,b: 100},
    ],
    [   // Pink
        {r: 236,g: 72,b: 153},
        {r: 219,g: 39,b: 119},
        {r: 190,g: 24,b: 93},
        {r: 157,g: 23,b: 77},
        {r: 80,g: 7,b: 36},
    ],
    [   // Blue
        {r: 59,g: 130,b: 246},
        {r: 37,g: 99,b: 235},
        {r: 29,g: 78,b: 216},
        {r: 30,g: 64,b: 175},
        {r: 23,g: 37,b: 84},
    ],
    [   // Yellow
        {r: 234,g: 179,b: 8},
        {r: 202,g: 138,b: 4},
        {r: 161,g: 98,b: 7},
        {r: 133,g: 77,b: 14},
        {r: 113,g: 63,b: 18},
        {r: 66,g: 32,b: 6},
    ],
    [   // Indigo
        {r: 99,g: 102,b: 241},
        {r: 79,g: 70,b: 229},
        {r: 67,g: 56,b: 202},
        {r: 55,g: 48,b: 163},
        {r: 30,g: 27,b: 75},
    ]
]

export function getTagColor(tagNameId: bigint, tagId: bigint): {r: number, g: number, b: number, opacity: number} {
    const tagNameIdNum = Number(tagNameId);
    const tagIdNum = Number(tagId);
    const colorList = TAG_COLORS[tagNameIdNum % TAG_COLORS.length];
    const color = colorList[tagIdNum % colorList.length];
    return {r: color.r, g: color.g, b: color.b, opacity: 1};
}

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

import { Button } from "../../atoms/Button.tsx";
import { IconCirclePlus, IconEyeSlash, IconTrash } from "../../atoms/Icon.tsx";
import { open } from '@tauri-apps/plugin-dialog';
import { useFolderImport } from "../../../selectia-rs/hooks/UseImportFolder.ts";
import { useEffect, useState } from "react";
import { TagsSubSection } from "./FilterSubSectionLabels.tsx";
import { FilterSelection } from "../../../selectia-rs/models.ts";
import { ExpandableRegion } from "../../molecules/ExpandableRegion.tsx";

function DirectorySubSection() {
    return <div className="flex flex-row w-full bg-slate-800">
        <div className="flex-auto flex justify-center">
        </div>
    </div>;
}

export function FilterSection(props: {
    className?: string;
    onFilterChange?: (filter: FilterSelection) => void;
}) {
    const [filter, setFilter] = useState<FilterSelection>({ directories: [], tags: {} });
    
    useEffect(() => {
        if (props.onFilterChange) {
            props.onFilterChange(filter);
        }
    }, [filter]);

    const [folderImport] = useFolderImport();

    const handleAdd = async () => {
        const result = await open({
            title: "Add new collection directory",
            directory: true,
            multiple: true,
        });
        if (result && result.length > 0) {
            for (let entry of result) {
                if (await folderImport(entry)) {
                    console.log("Successfull import")
                } else {
                    console.error("Errored import")
                }
            }
        }
    }

    const directoryHeader = (
        <div className="flex flex-row justify-between w-full">
            <span className="text-sm/2 text-slate-500">
                Directories
            </span>
            <Button onClick={() => handleAdd()} variant="outline">
                <IconCirclePlus />
            </Button>
        </div>
    );

    const tagHeader = (
        <div className="flex flex-row justify-between w-full">
            <span className="text-sm/2 text-slate-500">
                Tags
            </span>
            <Button variant="outline">
                <IconEyeSlash />
            </Button>
        </div>
    );

    return <div className={`${props.className} bg-slate-900 p-2 overflow-scroll`}>
        <ExpandableRegion expanded={true} header={directoryHeader}>
            <DirectorySubSection />
        </ExpandableRegion>
        <ExpandableRegion expanded={true} header={tagHeader}>
            <TagsSubSection className="p-2" onSelectionChange={(selection) => {
                setFilter({ ...filter, tags: selection });
            }} />
        </ExpandableRegion>
        </div>;
}


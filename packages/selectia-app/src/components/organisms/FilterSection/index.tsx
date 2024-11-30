import { Button } from "../../atoms/Button.tsx";
import { IconCirclePlus } from "../../atoms/Icon.tsx";
import { open } from "@tauri-apps/plugin-dialog";
import { useFolderImport } from "../../../selectia-tauri/hooks/UseImportFolder.ts";
import { useEffect, useState } from "react";

import { FilterSubSectionLabels } from "./FilterSubSectionLabels.tsx";
import { FilterSelection } from "../../../selectia-tauri/dto/models.ts";
import { FilterSubSectionDirectories } from "./FilterSubSectionDirectories.tsx";
import {
    IPaneviewPanelProps,
    PaneviewApi,
    PaneviewReact,
    PaneviewReadyEvent,
} from "dockview-react";
import { PaneViewHeader } from "../..";

export * from "./FilterSubSectionDirectories";
export * from "./FilterSubSectionLabels";

export function FilterSection(props: {
    className?: string;
    onFilterChange?: (filter: FilterSelection) => void;
}) {
    const [filter, setFilter] = useState<FilterSelection>({
        directories: [],
        tags: {},
    });

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
                    console.log("Successfull import");
                } else {
                    console.error("Errored import");
                }
            }
        }
    };

    const headerComponents = {
        "tags": PaneViewHeader,
        "addDirectory": (props: IPaneviewPanelProps) => {
            return PaneViewHeader({
                actionComponents: (
                    <Button variant="ghost" onClick={handleAdd}>
                        <IconCirclePlus />
                    </Button>
                ),
                ...props,
            });
        },
    };

    const components = {
        "directories": () => <FilterSubSectionDirectories />,
        "tags": () => (
            <FilterSubSectionLabels
                className="p-2"
                onSelectionChange={(selection) => {
                    setFilter({ ...filter, tags: selection });
                }}
            />
        ),
    };

    const onReady = (event: PaneviewReadyEvent) => {
        const api: PaneviewApi = event.api;
        api.addPanel({
            id: "directories",
            headerComponent: "addDirectory",
            component: "directories",
            title: "Directories",
        });
        api.addPanel({
            id: "tags",
            headerComponent: "tags",
            component: "tags",
            title: "Tags",
        });
    };

    return (
        <PaneviewReact
            onReady={onReady}
            components={components}
            headerComponents={headerComponents}
        />
    );
}

import { useState } from "react";
import { FilterSection } from "..";
import {
    ISplitviewPanelProps,
    SplitviewApi,
    SplitviewReact,
    SplitviewReadyEvent,
} from "dockview-react";
import { FilterSelection } from "../../selectia-tauri/dto/models";
import { InteractiveTable } from "./InteractiveTable";
import { emit } from "@tauri-apps/api/event";
import { useEvent } from "../../selectia-tauri/hooks/UseEvent";

export function Explorer(props: { contextId: string; className?: string }) {
    const components = {
        "filter_section": () => {
            return (
                <div className="w-full h-full overflow-auto">
                    <FilterSection
                        onFilterChange={(filter) => {
                            emit("interactive_list_filter_change", filter);
                        }}
                    />
                </div>
            );
        },
        "interactive_table": (props: ISplitviewPanelProps) => {
            const [filter, setFilter] = useState<FilterSelection>({
                directories: [],
                tags: {},
            });

            useEvent<FilterSelection>(
                "interactive_list_filter_change",
                (filter) => {
                    setFilter(filter);
                },
            );
            return (
                <div className="w-full h-full overflow-auto">
                    <InteractiveTable
                        filter={filter}
                        context_id={props.params.contextId}
                    />
                </div>
            );
        },
    };

    const onReady = (event: SplitviewReadyEvent) => {
        const api: SplitviewApi = event.api;
        api.addPanel({
            id: "filter_section",
            component: "filter_section",
            params: {
                contextId: props.contextId,
            },
            minimumSize: 30,
        });
        api.addPanel({
            id: "interactive_table",
            component: "interactive_table",
            params: {
                contextId: props.contextId,
            },
            minimumSize: 70,
        });
    };

    return (
        <SplitviewReact
            className={props.className}
            onReady={onReady}
            components={components}
        />
    );
}

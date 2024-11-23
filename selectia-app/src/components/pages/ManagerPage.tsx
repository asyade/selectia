import {FilterSection} from "../organisms/FilterSection";
import { Statusbar } from "../organisms/StatusBar";
import { useCallback, useEffect, useState } from "react";
import { InteractiveTable } from "../organisms/InteractiveTable";
import { FilterSelection } from "../../selectia-rs/models";
import { interactive_list_create_context, interactive_list_delete_context } from "../../selectia-rs";

export function ManagerPage() {
    // const [filter, setFilter] = useState<FilterSelection>({ directories: [], tags: {} });
    const [filter, setFilter] = useState<FilterSelection>({ directories: [], tags: {} });

    const [contextId, setContextId] = useState<string | null>(null);

    const deleteContext = useCallback(() => {
        if (contextId) {
            interactive_list_delete_context(contextId);
        }
    }, [contextId]);

    useEffect(() => {
        interactive_list_create_context().then(setContextId);
        return () => {
            if (contextId) {
                deleteContext();
            }
        }
    }, []);

    return (
        <div className="flex flex-col h-screen w-screen overflow-scroll">
            {/* <ActionBar className="flex-none  w-full" /> */}
            <div className="basis-full flex w-full overflow-scroll shadow-inner">
                <FilterSection className="flex-auto w-1/4" onFilterChange={(filter) => {
                        setFilter(filter);
                    }}
                />
                {
                    contextId && (
                        <InteractiveTable context_id={contextId} filter={filter} className="flex-auto w-3/4 flex-grow"/>
                    )
                }
            </div>
            <Statusbar className="flex-none w-full flex" />
        </div>
    );
}

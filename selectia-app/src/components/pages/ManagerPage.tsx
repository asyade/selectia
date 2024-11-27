import { FilterSection } from "../organisms/FilterSection/FilterSection";
import { Statusbar } from "../organisms/StatusBar";
import { useCallback, useEffect, useState } from "react";
import { InteractiveTable } from "../organisms/InteractiveTable/InteractiveTable";
import { FilterSelection } from "../../selectia-rs/models";
import { EntryViewCursor, interactive_list_create_context, interactive_list_delete_context } from "../../selectia-rs";

import { DndProvider } from 'react-dnd'
import { HTML5Backend } from 'react-dnd-html5-backend'
import { Player } from "../organisms/Player/Player";

export const ItemTypes = {
    INTERACTIVE_TABLE_ROW: "interactive_table_row",
    INTERACTIVE_TABLE_LABEL: "interactive_table_label",
    FILTER_SECTION_LABEL: "filter_section_label",
}

export function ManagerPage() {
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
        <DndProvider backend={HTML5Backend}>

            <div className="flex flex-grow overflow-auto w-full">

                {/* <ActionBar className="flex-none  w-full" /> */}
                <FilterSection className="flex-auto w-1/4" onFilterChange={(filter) => {
                    setFilter(filter);
                }}
                />
                {
                    contextId && (
                        <InteractiveTable
                            context_id={contextId}
                            filter={filter}
                            className="flex-auto w-3/4 flex-grow"
                        />
                    )
                }
            <Player />
            </div>
        </DndProvider>

    );
}

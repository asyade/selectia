import {FilterSection} from "../organisms/FilterSection";
import {InteractiveList} from "../organisms/InteractiveList";
import { ActionBar } from "../organisms/ActionBar";
import { Statusbar } from "../organisms/StatusBar";
import { useEntries } from "../../hooks/EntryHook";
import { useState } from "react";
import { FilterSelection } from "../../hooks/EntryHook";

export function ManagerPage() {
    // const [filter, setFilter] = useState<FilterSelection>({ directories: [], tags: {} });
    const [entries, filter, setFilter] = useEntries({ directories: [], tags: {} });

    return (
        <div className="flex flex-col h-screen w-screen overflow-scroll">
            {/* <ActionBar className="flex-none  w-full" /> */}
            <div className="basis-full flex w-full overflow-scroll shadow-inner">
                <FilterSection className="flex-auto w-1/4" onFilterChange={(filter) => {
                        setFilter(filter);
                    }}
                />
                <InteractiveList entries={entries} className="flex-auto w-3/4 flex-grow" filter={filter} />
            </div>
            <Statusbar className="flex-none w-full flex" />
        </div>
    );
}

import {FilterSection} from "../organisms/FilterSection";
import { Statusbar } from "../organisms/StatusBar";
import { useState } from "react";
import { InteractiveTable } from "../organisms/InteractiveTable";
import { FilterSelection } from "../../selectia-rs/models";

export function ManagerPage() {
    // const [filter, setFilter] = useState<FilterSelection>({ directories: [], tags: {} });
    const [filter, setFilter] = useState<FilterSelection>({ directories: [], tags: {} });

    return (
        <div className="flex flex-col h-screen w-screen overflow-scroll">
            {/* <ActionBar className="flex-none  w-full" /> */}
            <div className="basis-full flex w-full overflow-scroll shadow-inner">
                <FilterSection className="flex-auto w-1/4" onFilterChange={(filter) => {
                        setFilter(filter);
                    }}
                />
                <InteractiveTable filter={filter} className="flex-auto w-3/4 flex-grow"/>
            </div>
            <Statusbar className="flex-none w-full flex" />
        </div>
    );
}

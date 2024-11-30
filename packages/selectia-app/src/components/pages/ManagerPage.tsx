import { interactive_list_create_context } from "../../selectia-tauri";

import {
    DockviewApi,
    DockviewReact,
    DockviewReadyEvent,
    IDockviewPanelHeaderProps,
    IDockviewPanelProps,
} from "dockview-react";

import { Player } from "../organisms/Player/Player";
import { Explorer } from "../organisms/Explorer";
import { Button, DockViewHeader, DropDownButton, DropZoneDecorator, IconCirclePlus } from "..";
import { useDrop } from "react-dnd";

export const ItemTypes = {
    INTERACTIVE_TABLE_ROW: "interactive_table_row",
    INTERACTIVE_TABLE_LABEL: "interactive_table_label",
    FILTER_SECTION_LABEL: "filter_section_label",
};

export function ManagerPage() {
    const onReady = (event: DockviewReadyEvent) => {
        const api: DockviewApi = event.api;
        interactive_list_create_context().then((contextId) => {
            api.addPanel({
                id: "explorer",
                component: "explorer",
                tabComponent: "explorer",
                title: "Library",
                params: {
                    contextId: contextId,
                },
            });

            api.addPanel({
                id: "player",
                component: "player",
                tabComponent: "player",
                title: "Deck 0 - (empty)",
            });
        });
    };

    const headerComponents = {
        "explorer": (props: IDockviewPanelHeaderProps) => {
            return <DockViewHeader {...props} />;
        },
        "player": (props: IDockviewPanelHeaderProps) => {
            return <DockViewHeader {...props} />;
        },
    };

    const components = {
        "explorer": (props: IDockviewPanelProps) => {
            return <Explorer contextId={props.params.contextId} />;
        },
        "player": (props: IDockviewPanelProps) => {
            return <Player />;
        },
    };

    return (
        <DockviewReact
            className="dockview-theme-dracula dockview-theme-dracula-custom bg-secondary"
            onReady={onReady}
            components={components}
            tabComponents={headerComponents}
            leftHeaderActionsComponent={TabHeaderActions}
        />
    );
}

function TabHeaderActions() {
    const [{ isOver, canDrop }, drop] = useDrop(() => ({
        accept: [ItemTypes.INTERACTIVE_TABLE_ROW],
        drop: (_args, _monitor) => {
        },
        collect: (monitor) => ({
            canDrop: !!monitor.canDrop(),
            isOver: !!monitor.isOver(),
        }),
    }), []);

    return (
        <DropZoneDecorator
            dropZoneRef={drop}
            className="flex flex-row gap-2 items-center justify-left m-1 p-1"
            isOver={isOver}
            canDrop={canDrop}
        >
            <DropDownButton
                variant="outline"
                buttonContent={<IconCirclePlus />}
                dropDownClassName="flex flex-col"
            >
                <Button variant="outline">
                    <span className="text-sm text-secondary w-full text-left">
                        New library
                    </span>
                </Button>
                <Button variant="outline">
                    <span className="text-sm text-secondary w-full text-left">
                        New Deck
                    </span>
                </Button>
            </DropDownButton>
        </DropZoneDecorator>
    );
}

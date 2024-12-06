import {
    create_audio_deck,
    EntryViewCursor,
    get_audio_decks,
    interactive_list_create_context,
    load_audio_track,
} from "../../selectia-tauri";

import {
    DockviewApi,
    DockviewReact,
    DockviewReadyEvent,
    IDockviewHeaderActionsProps,
    IDockviewPanelHeaderProps,
    IDockviewPanelProps,
} from "dockview-react";

import { Explorer } from "../organisms/Explorer";
import {
    Button,
    DockViewHeader,
    DropDownButton,
    DropZoneDecorator,
    IconCirclePlus,
    IconXmark,
    TrackControls,
    Player,
    PlayerProps,
} from "..";
import { useDrop } from "react-dnd";
import { EntryView } from "../../selectia-tauri/dto/models";
import { useDeckMetadata, useDeckStatus } from "../../selectia-tauri/hooks/UseAudioPlayer";
import { useEffect } from "react";
import { Console } from "../organisms/Console";

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

            get_audio_decks().then((decks) => {
                decks.forEach((deck) => {
                    api.addPanel<PlayerProps>({
                        id: `player-deck-${deck.id}`,
                        component: "player-deck",
                        tabComponent: "player-deck",
                        params: {
                            deckId: BigInt(deck.id),
                            status: deck.file?.status ?? null,
                            metadata: deck.file?.metadata ?? null,
                            payload: deck.file?.payload ?? null,
                        },
                    });
                });
            });
        });
    };

    const headerComponents = {
        "explorer": (props: IDockviewPanelHeaderProps) => {
            return (
                <DockViewHeader {...props}>
                </DockViewHeader>
            );
        },
        "player-deck": (props: IDockviewPanelHeaderProps<PlayerProps>) => {
            const [metadata] = useDeckMetadata(props.params.deckId, props.params.metadata);
            const [status, setStatus] = useDeckStatus(props.params.deckId, props.params.status);

            return (
                <DockViewHeader
                    {...props}
                    actionComponents={
                        <TrackControls
                            metadata={metadata}
                            status={status}
                            setStatus={setStatus}
                        />
                    }
                />
            );
        },
    };

    const components = {
        "console": (props: IDockviewPanelProps) => {
            return <Console />;
        },
        "explorer": (props: IDockviewPanelProps) => {
            return <Explorer contextId={props.params.contextId} />;
        },
        "player-deck": (props: IDockviewPanelProps<PlayerProps>) => {
            return <Player {...props.params} />;
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

function TabHeaderActions(props: IDockviewHeaderActionsProps) {
    const [{ isOver, canDrop }, drop] = useDrop(() => ({
        accept: [ItemTypes.INTERACTIVE_TABLE_ROW],
        drop: (args: EntryViewCursor, _monitor) => {
            create_audio_deck().then((deck_id) => {
                props.containerApi.addPanel<PlayerProps>({
                    id: `player-deck-${deck_id}`,
                    component: "player-deck",
                    tabComponent: "player-deck",
                    params: {
                        deckId: deck_id,
                        status: null,
                        metadata: null,
                        payload: null,
                    },
                    position: {
                        referenceGroup: props.group.id
                    }
                });
                // Wait 100ms to ensure that the panel got the first status update
                // Will not be required once progress is implemented as multiple status updates will be sent
                setTimeout(() => {
                    load_audio_track(deck_id, args.entry.metadata_id).then(() => {
                    });
                }, 100);
            });
        },
        collect: (monitor) => ({
            canDrop: !!monitor.canDrop(),
            isOver: !!monitor.isOver(),
        }),
    }), []);

    const createDeck = (entry?: EntryView) => {
        create_audio_deck().then((deck_id) => {
            if (entry?.metadata_id) {
                load_audio_track(deck_id, entry.metadata_id);
            }
            props.containerApi.addPanel<PlayerProps>({
                id: `player-deck-${deck_id}`,
                component: "player-deck",
                tabComponent: "player-deck",
                params: {
                    deckId: deck_id,
                    status: null,
                    metadata: null,
                    payload: null,
                },
                position: {
                    referenceGroup: props.group.id
                }
            });
        });
    };

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
                <Button variant="outline" onClick={() => createDeck()}>
                    <span className="text-sm text-secondary w-full text-left">
                        New Deck
                    </span>
                </Button>
            </DropDownButton>
        </DropZoneDecorator>
    );
}

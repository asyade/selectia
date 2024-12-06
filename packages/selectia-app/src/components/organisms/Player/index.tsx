import { useEffect, useMemo, useRef, useState } from "react";
import {
    DeckFileMetadataSnapshot,
    DeckFilePayloadSnapshot,
    DeckFileStatus,
} from "../../../selectia-tauri/dto/models";
import { useDeck } from "../../../selectia-tauri/hooks/UseAudioPlayer";
import { DropZoneDecorator } from "../../molecules/DropZoneDecorator";
import { useDrop } from "react-dnd";
import { ItemTypes } from "../../pages/ManagerPage";
import { EntryViewCursor, load_audio_track } from "../../../selectia-tauri";
import { TrackControls } from "./TrackControls";
import { TrackView } from "./TrackView";

export { TrackControls };

export interface PlayerProps {
    deckId: bigint;
    status: DeckFileStatus | null;
    metadata: DeckFileMetadataSnapshot | null;
    payload: DeckFilePayloadSnapshot | null;
}

export function Player(props: PlayerProps) {

    const [{ isOver, canDrop }, drop] = useDrop(() => ({
        accept: [ItemTypes.INTERACTIVE_TABLE_ROW],
        drop: (args: EntryViewCursor, _monitor) => {
            load_audio_track(props.deckId, args.entry.metadata_id)
        },
        collect: (monitor) => ({
            canDrop: !!monitor.canDrop(),
            isOver: !!monitor.isOver(),
        }),
    }), []);

    const trackViewMemo = (
        <TrackView
            deckId={props.deckId}
            payload={props.payload}
            status={props.status}
        />
    );
    return (
        <div className="h-full w-full p-1">
            <DropZoneDecorator
                dropZoneRef={drop}
                className="flex flex-row gap-2 items-center justify-left h-full"
                isOver={isOver}
                canDrop={canDrop}
            >
                {trackViewMemo}
            </DropZoneDecorator>
        </div>
    );
}

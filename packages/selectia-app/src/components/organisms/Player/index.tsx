import {
    DeckFileMetadataSnapshot,
    DeckFilePayloadSnapshot,
    DeckFileStatus,
} from "../../../selectia-tauri/dto/models";
import {
    useDeckPayload,
    useDeckStatus,
} from "../../../selectia-tauri/hooks/UseAudioPlayer";
import { DropZoneDecorator } from "../../molecules/DropZoneDecorator";
import { useDrop } from "react-dnd";
import { ItemTypes } from "../../pages/ManagerPage";
import {
    EntryVariationCursor,
    EntryViewCursor,
    load_audio_track_from_metadata,
    load_audio_track_from_variation,
} from "../../../selectia-tauri";
import { TrackControls } from "./TrackControls";
import { TrackView } from "./TrackView";
import { TrackDetails } from "./TrackDetails";

export { TrackControls };

export interface PlayerProps {
    deckId: bigint;
    status: DeckFileStatus | null;
    metadata: DeckFileMetadataSnapshot | null;
    payload: DeckFilePayloadSnapshot | null;
}

export function Player(props: PlayerProps) {
    const [payload] = useDeckPayload(props.deckId, props.payload);
    const [status, _setStatus] = useDeckStatus(props.deckId, props.status);

    const [{ isOver, canDrop }, drop] = useDrop(() => ({
        accept: [
            ItemTypes.INTERACTIVE_TABLE_ROW,
            ItemTypes.INTERACTIVE_TABLE_ROW_VARIATION,
        ],
        drop: (args: EntryViewCursor | EntryVariationCursor, _monitor) => {
            const kind = _monitor.getItemType();
            if (kind === ItemTypes.INTERACTIVE_TABLE_ROW) {
                const entry = args as EntryViewCursor;
                load_audio_track_from_metadata(
                    props.deckId,
                    entry.entry.metadata_id,
                );
            } else {
                const entry = args as EntryVariationCursor;
                load_audio_track_from_variation(
                    props.deckId,
                    entry.variation.id,
                );
            }
        },
        collect: (monitor) => ({
            canDrop: !!monitor.canDrop(),
            isOver: !!monitor.isOver(),
        }),
    }), []);

    return (
        <div className="h-full w-full p-1">
            <DropZoneDecorator
                dropZoneRef={drop}
                className="flex flex-col gap-2 items-center justify-left h-full"
                isOver={isOver}
                canDrop={canDrop}
            >
                <TrackView
                    deckId={props.deckId}
                    payload={payload}
                    status={status}
                />
                <TrackDetails
                    status={status}
                    payload={payload}
                    deckId={props.deckId}
                />
            </DropZoneDecorator>
        </div>
    );
}

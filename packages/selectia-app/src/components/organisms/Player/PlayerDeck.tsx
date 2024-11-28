import {
    DeckFileMetadataSnapshot,
    DeckFilePayloadSnapshot,
    DeckFileStatus,
} from "../../../selectia-tauri/dto/models";
import { useDeck } from "../../../selectia-tauri/hooks/UseAudioPlayer";
import { IconPause } from "../../atoms/Icon";
import { IconPlay } from "../../atoms/Icon";

export function PlayerDeck(
    props: {
        deckId: number;
    },
) {
    const [metadata, payload, status] = useDeck(props.deckId);
    if (!metadata) {
        return <div>Loading...</div>;
    }
    return (
        <div className="bg-slate-800">
            <div className="flex flex-col gap-2 relative">
                <TrackView payload={payload} status={status} />
                <div className="flex justify-between items-center">
                    <div className="grow overflow-hidden">
                        <p className="text-white text-xs truncate">
                            {metadata.title}
                        </p>
                    </div>
                    <div className="shrink-0">
                        <TrackControls />
                    </div>
                </div>
            </div>
        </div>
    );
}

function TrackView(props: { payload: DeckFilePayloadSnapshot | null, status: DeckFileStatus | null }) {
    if (props.payload === null || props.status === null) {
        return (
            <div className="w-full h-16">
                <div className="bg-red-500 h-full" style={{ width: `10%` }}>
                </div>
            </div>
        );
    }

    const progress = trackProgress(props.payload, props.status);

    return (
        <div className="w-full h-16">
            <div
                className="bg-red-500 h-full"
                style={{ width: `${progress}%` }}
            >
            </div>
        </div>
    );
}

function trackProgress(payload: DeckFilePayloadSnapshot, status: DeckFileStatus) {
    if (status.kind === "Playing") {
        return status.offset / payload.samples_count * 100;
    } else if (status.kind === "Paused") {
        return status.offset / payload.samples_count * 100;
    } else {
        return 0;
    }
}

function TrackControls() {
    return (
        <div>
            <button>
                <IconPlay />
            </button>
            <button>
                <IconPause />
            </button>
        </div>
    );
}

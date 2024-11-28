import { useEffect, useMemo, useState } from "react";
import {
    DeckFileMetadataSnapshot,
    DeckFilePayloadSnapshot,
    DeckFileStatus,
} from "../../../selectia-tauri/dto/models";
import { useDeck } from "../../../selectia-tauri/hooks/UseAudioPlayer";
import { IconPause } from "../../atoms/Icon";
import { IconPlay } from "../../atoms/Icon";
import { Spinner } from "../../atoms/Spinner";
import { Button } from "../../atoms/Button";

export function PlayerDeck(
    props: {
        deckId: number;
    },
) {
    const [metadata, payload, status, setStatus] = useDeck(props.deckId);
    
    const [statusKind, setStatusKind] = useState<DeckFileStatus["kind"] | null>(null);
    
    const trackViewMemo = useMemo(() => {
        return <TrackView payload={payload} status={status} setStatus={setStatus} />;
    }, [payload, status]);


    const trackControlsMemo = useMemo(() => {
        return <TrackControls status={status} setStatus={setStatus} />;
    }, [statusKind]);

    useEffect(() => {
        if (status?.kind !== statusKind) {
            setStatusKind(status?.kind ?? null);
        }
    }, [status]);

    if (!metadata) {
        return <div>Loading...</div>;
    }
    return (
        <div className="bg-slate-800">
            <div className="flex flex-col gap-2 relative">
                {trackViewMemo}
                <div className="flex justify-between items-center">
                    <div className="grow overflow-hidden">
                        <p className="text-white text-xs truncate">
                            {metadata.title}
                        </p>
                    </div>
                    <div className="shrink-0">
                        {trackControlsMemo}
                    </div>
                </div>
            </div>
        </div>
    );
}

function TrackView(
    props: {
        payload: DeckFilePayloadSnapshot | null;
        status: DeckFileStatus | null;
        setStatus: (status: DeckFileStatus) => void;
    },
) {
    if (props.payload === null || props.status === null) {
        return (
            <div className="w-full h-16">
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

function trackProgress(
    payload: DeckFilePayloadSnapshot,
    status: DeckFileStatus,
) {
    if (status.kind === "Playing") {
        return status.offset / payload.samples_count * 100;
    } else if (status.kind === "Paused") {
        return status.offset / payload.samples_count * 100;
    } else {
        return 0;
    }
}

function TrackControls({ status, setStatus }: { status: DeckFileStatus | null, setStatus: (status: DeckFileStatus) => void }) {
    if (status && status.kind === "Playing") {
        return (
            <div>
                <Button onClick={() => setStatus({ kind: "Paused", offset: status.offset })}>
                    <IconPause />
                </Button>
            </div>
        );
    } else if (status && status.kind === "Paused") {
        return (
            <div>
                <Button onClick={() => setStatus({ kind: "Playing", offset: status.offset })}>
                    <IconPlay />
                </Button>
            </div>
        );
    } else {
        return <Spinner />;
    }
}

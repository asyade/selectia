import { useMemo } from "react";
import { useDeck } from "../../../selectia-tauri/hooks/UseAudioPlayer";
import { DeckFileView } from "../../../selectia-tauri/dto/models";
import { IconPause } from "../../atoms/Icon";
import { IconPlay } from "../../atoms/Icon";

export function PlayerDeck(props: { deckId: number }) {

    const deckId = useMemo(() => props.deckId, [props.deckId]);

    const [file] = useDeck(deckId);

    if (!file) {
        return <div>Loading...</div>;
    }

    return (
    <div className="bg-slate-800">
        <div className="flex flex-col gap-2 relative">
            <TrackView file={file} />
            <div className="flex justify-between items-center">
                <div className="grow overflow-hidden">
                    <p className="text-white text-xs truncate">{file.title}</p>
                </div>
                <div className="shrink-0">
                    <TrackControls />
                </div>
            </div>
        </div>
    </div>
    );
}

function TrackControls() {
    return <div>
        <button><IconPlay /></button>
        <button><IconPause /></button>
    </div>;
}

function TrackView(props: { file: DeckFileView }) {
    return (
        <div className="w-full h-16">
            <div className="bg-red-500 h-full" style={{ width: `10%` }}></div>
        </div>
    );
}

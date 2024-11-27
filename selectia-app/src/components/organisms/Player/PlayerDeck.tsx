import { useMemo } from "react";
import { useDeck } from "../../../selectia-rs/hooks/UseAudioPlayer";

export function PlayerDeck(props: { deckId: number }) {

    const deckId = useMemo(() => props.deckId, [props.deckId]);

    const [file] = useDeck(deckId);

    return (
        <div className="bg-slate-800">
            {file ? <span>{file.title}</span> : <span>Loading...</span>}
        </div> 
    );
}

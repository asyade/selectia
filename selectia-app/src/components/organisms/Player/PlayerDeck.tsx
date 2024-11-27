import { useDeck } from "../../../selectia-rs/hooks/UseAudioPlayer";

export function PlayerDeck(props: { deckId: number }) {
    const [file] = useDeck(props.deckId);
    console.log("@@@@@@@@", props.deckId)
    return (
        <div className="bg-slate-800">
            {file ? <span>{file.title}</span> : <span>Loading...</span>}
        </div> 
    );
}

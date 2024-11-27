import { useDrop } from "react-dnd";
import { useAudioPlayer } from "../../../selectia-rs/hooks/UseAudioPlayer";
import { PlayerDeck } from "./PlayerDeck";
import { ItemTypes } from "../../pages/ManagerPage";
import { IconCirclePlus } from "../../atoms/Icon";
import { create_audio_deck, EntryView, load_audio_track } from "../../../selectia-rs";
import { useMemo } from "react";

export function Player() {
    const [decks] = useAudioPlayer();

    const [{ isOver, canDrop }, drop] = useDrop(() => ({
        accept: [ItemTypes.INTERACTIVE_TABLE_ROW],
        drop: (args: { entry: EntryView }, _monitor) => {
            create_audio_deck().then(deck_id => load_audio_track(deck_id, args.entry.metadata_id));
        },
        collect: (monitor) => ({
            isOver: !!monitor.isOver(),
            canDrop: !!monitor.canDrop(),
        }),
    }), []);

    const deckElements = useMemo(() => {
        console.log("decks", decks);
        return decks.map((deck) => <PlayerDeck key={deck.id} deckId={deck.id} />);
    }, [decks]);

    return (
        <div className="p-1 flex flex-col bg-slate-800 h-200 fixed bottom-12 left-0 right-0 z-50 gap-1">
            {canDrop && (
                <div ref={drop} className={`bg-slate-700 w-full p-4 flex items-center justify-center outline outline-dashed ${isOver ? "outline-2 outline-green-400" : ""}`}>
                    <IconCirclePlus />
                </div>
            )}
            <div className="flex flex-col gap-1">
                {deckElements}
            </div>
        </div>
    );
}
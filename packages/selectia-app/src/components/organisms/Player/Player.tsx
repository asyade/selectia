import { useDrop } from "react-dnd/dist/index.js";
import { useAudioPlayer } from "../../../selectia-tauri/hooks/UseAudioPlayer";
import { PlayerDeck } from "./PlayerDeck";
import { ItemTypes } from "../../pages/ManagerPage";
import { IconCirclePlus } from "../../atoms/Icon";
import { useMemo } from "react";
import { EntryView } from "../../../selectia-tauri/dto/models";
import { create_audio_deck, load_audio_track } from "../../../selectia-tauri";

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
        return decks.map((deck) => <PlayerDeck key={deck.id} deckId={deck.id} status={deck.file?.status ?? null} metadata={deck.file?.metadata ?? null} payload={deck.file?.payload ?? null} />);
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

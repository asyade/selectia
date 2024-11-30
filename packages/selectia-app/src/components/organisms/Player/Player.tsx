import { useDrop } from "react-dnd/dist/index.js";
import { useAudioPlayer } from "../../../selectia-tauri/hooks/UseAudioPlayer";
import { PlayerDeck } from "./PlayerDeck";
import { ItemTypes } from "../../pages/ManagerPage";
import { IconCirclePlus } from "../../atoms/Icon";
import { useMemo } from "react";
import { EntryView } from "../../../selectia-tauri/dto/models";
import { create_audio_deck, load_audio_track } from "../../../selectia-tauri";

export function Player(props: {
    className?: string;
}) {
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
        <div className={`flex flex-col h-200 gap-1 min-h-full ${props.className}`}>
            {decks.length === 0 && (
                <div className="flex-grow flex items-center justify-center">
                    <div className="text-slate-400">Drop tracks here</div>
                </div>
            )}
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

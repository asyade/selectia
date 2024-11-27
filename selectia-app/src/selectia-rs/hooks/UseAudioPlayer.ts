import { useEffect, useState } from "react";
import { AudioDeckUpdatedEvent, DeckFileView, DeckView } from "../models";
import { get_audio_decks } from "..";
import { listen } from "@tauri-apps/api/event";



export function useAudioPlayer(): [DeckView[]] {
    const [decks, setDecks] = useState<DeckView[]>([]);

    useEffect(() => {
        get_audio_decks().then(setDecks);
    }, []);

    useEffect(() => {
        const unlisten = listen("audio-deck-created", (event) => {
            console.log("audio-deck-created", event);
            const id = event.payload as number;
            setDecks(prev => [...prev, { id, file: null }]);
        });

        return () => {
            unlisten.then(unlisten => unlisten());
        };
    }, []);
    return [
        decks,
    ];
}

export function useDeck(deckId: number): [DeckFileView | null] {
    const [file, setFile] = useState<DeckFileView | null>(null);
    useEffect(() => {
        const unlisten = listen("audio-deck-updated", (event) => {
            const payload = event.payload as AudioDeckUpdatedEvent;
            if (payload.id === deckId) {
                setFile(payload.file);
            }
        });

        return () => {
            unlisten.then(unlisten => unlisten());
        };
    }, []);

    return [
        file,
    ];
}
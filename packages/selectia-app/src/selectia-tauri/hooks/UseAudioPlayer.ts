import { useEffect, useState } from "react";
import { DeckFileView, DeckView } from "../dto/models";
import {
    AudioDeckCreatedEvent,
    AudioDeckUpdatedEvent,
} from "../dto/events";
import { get_audio_decks } from "../index";
import { useEvent } from "./UseEvent";

export function useAudioPlayer(): [DeckView[]] {
    const [decks, setDecks] = useState<DeckView[]>([]);

    useEffect(() => {
        get_audio_decks().then(setDecks);
    }, []);

    useEvent<AudioDeckCreatedEvent>("AudioDeckCreated", (event) => {
        setDecks((prev) => [...prev, { id: event.id, file: null }]);
    });

    return [
        decks,
    ];
}

export function useDeck(deckId: number): [DeckFileView | null] {
    const [file, setFile] = useState<DeckFileView | null>(null);
    useEvent<AudioDeckUpdatedEvent>("AudioDeckUpdated", (event) => {
        if (event.id === deckId) {
            setFile(event.file);
        }
    });
    return [
        file,
    ];
}

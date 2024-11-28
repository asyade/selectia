import { useCallback, useEffect, useState } from "react";
import { DeckFileMetadataSnapshot, DeckFilePayloadSnapshot, DeckFileStatus, DeckView } from "../dto/models";
import {
    AudioDeckCreatedEvent,
    AudioDeckFileMetadataUpdatedEvent,
    AudioDeckFilePayloadUpdatedEvent,
    AudioDeckFileStatusUpdatedEvent,
} from "../dto/events";
import { get_audio_decks, set_deck_file_status } from "../index";
import { useEvent, useIdentifiedEvent } from "./UseEvent";

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



export function useDeck(deckId: number): [DeckFileMetadataSnapshot | null, DeckFilePayloadSnapshot | null, DeckFileStatus | null, (status: DeckFileStatus) => void] {
    const [metadata, setMetadata] = useState<DeckFileMetadataSnapshot | null>(null);
    const [payload, setPayload] = useState<DeckFilePayloadSnapshot | null>(null);
    const [status, setStatus] = useState<DeckFileStatus | null>(null);

    const setStatusDetached = useCallback((status: DeckFileStatus) => {
        set_deck_file_status(deckId, status)
            .then(() => {
                console.log("status set");
            })
            .catch((error) => {
                console.error(error);
            });
    }, [deckId, setStatus]);

    useIdentifiedEvent<AudioDeckFileMetadataUpdatedEvent>(`AudioDeckFileMetadataUpdated`, deckId, (event) => {
        setMetadata(event.metadata);
        setPayload(null);
        setStatus(null);
    });

    useIdentifiedEvent<AudioDeckFilePayloadUpdatedEvent>(`AudioDeckFilePayloadUpdated`, deckId, (event) => {
        setPayload(event.payload);
    });

    useIdentifiedEvent<AudioDeckFileStatusUpdatedEvent>(`AudioDeckFileStatusUpdated`, deckId, (event) => {
        setStatus(event.status);
    });

    return [
        metadata,
        payload,
        status,
        setStatusDetached,
    ];
}

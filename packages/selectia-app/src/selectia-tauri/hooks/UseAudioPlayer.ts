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

export function useDeckStatus(deckId: bigint, initialStatus: DeckFileStatus | null = null): [DeckFileStatus | null, (status: DeckFileStatus) => void] {
    const [status, setStatus] = useState<DeckFileStatus | null>(initialStatus);
 
    useIdentifiedEvent<AudioDeckFileStatusUpdatedEvent>(`AudioDeckFileStatusUpdated`, deckId, (event) => {
        setStatus(event.status);
    });
    
    const setStatusDetached = useCallback((status: DeckFileStatus) => {
        set_deck_file_status(deckId, status)
            .then(() => {
                console.log("status set");
            })
            .catch((error) => {
                console.error(error);
            });
    }, [deckId, setStatus]);

    return [status, setStatusDetached];
}

export function useDeckMetadata(deckId: bigint, initialMetadata: DeckFileMetadataSnapshot | null = null): [DeckFileMetadataSnapshot | null] {
    const [metadata, setMetadata] = useState<DeckFileMetadataSnapshot | null>(initialMetadata);
 
    useIdentifiedEvent<AudioDeckFileMetadataUpdatedEvent>(`AudioDeckFileMetadataUpdated`, deckId, (event) => {
        setMetadata(event.metadata);
    });
 
    return [metadata];
}

export function useDeckPayload(deckId: bigint, initialPayload: DeckFilePayloadSnapshot | null = null): [DeckFilePayloadSnapshot | null] {
    const [payload, setPayload] = useState<DeckFilePayloadSnapshot | null>(initialPayload);
 
    useIdentifiedEvent<AudioDeckFilePayloadUpdatedEvent>(`AudioDeckFilePayloadUpdated`, deckId, (event) => {
        setPayload(event.payload);
    });
 
    return [payload];
}

export function useDeck(
    deckId: bigint,
    initialStatus: DeckFileStatus | null = null,
    initialMetadata: DeckFileMetadataSnapshot | null = null,
    initialPayload: DeckFilePayloadSnapshot | null = null,
): [DeckFileMetadataSnapshot | null, DeckFilePayloadSnapshot | null, DeckFileStatus | null, (status: DeckFileStatus) => void] {
    const [metadata, setMetadata] = useState<DeckFileMetadataSnapshot | null>(initialMetadata);
    const [payload, setPayload] = useState<DeckFilePayloadSnapshot | null>(initialPayload);
    const [status, setStatus] = useState<DeckFileStatus | null>(initialStatus);

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

    useEffect(() => {

    }, [status]);

    return [
        metadata,
        payload,
        status,
        setStatusDetached,
    ];
}

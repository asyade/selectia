import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

export function useIdentifiedEvent<T>(
    name: string,
    id: number,
    callback: (event: T) => void,
): void {
    return useEvent(`${name}:${id}`, callback);
}

export function useEvent<T>(
    name: string,
    callback: (event: T) => void,
): void {
    useEffect(() => {
        const unlisten = listen(name, (event_payload) => {
            try {
                const payload = event_payload.payload as T;
                callback(payload);
            } catch (e) {
                console.error(
                    "Failed to parse event payload, event type probably incorrect",
                    e,
                );
            }
        });
        return () => {
            unlisten.then((unlisten) => unlisten());
        };
    }, []);
}
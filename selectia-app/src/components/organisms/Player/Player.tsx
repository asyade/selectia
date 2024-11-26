import { PlayerDeck } from "./PlayerDeck";

export function Player() {
    return (
        <div className="flex flex-col bg-slate-800 h-200 fixed bottom-12 left-0 right-0 z-50">
            <PlayerDeck />
        </div>
    )
}

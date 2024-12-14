import { DeckFileStatus } from "../../../selectia-tauri/dto/models";
import { DeckFilePayloadSnapshot } from "../../../selectia-tauri/dto/models";
import { Button } from "../../atoms/Button";
import { IconPlay } from "../../atoms/Icon";
import { Label } from "../../atoms/Label";
import Knob from "../../molecules/Knob";

export function TrackDetails(props: {
    deckId: bigint;
    payload: DeckFilePayloadSnapshot | null;
    status: DeckFileStatus | null;
}) {
    return (
        <div className="w-full flex flex-row gap-2 justify-between flex-shrink">
            <div className="flex flex-col gap-2">
                <p className="text-lg text-primary">Gost in the shell</p>
                <div className="flex flex-row gap-2">
                    <Label>
                        Unknown album
                    </Label>
                    <Label>
                        Unknown artist
                    </Label>
                </div>
            </div>
            <div className="flex flex-col gap-2">
                <TrackAdvance
                    status={props.status}
                    payload={props.payload}
                />
                <div className="flex flex-row gap-2">
                    <Knob />
                    <Knob />
                </div>
            </div>
        </div>
    );
}

function TrackAdvance(props: {
    status: DeckFileStatus | null;
    payload: DeckFilePayloadSnapshot | null;
}) {
    const totalDuration = props.payload?.duration ?? 0.0;
    const channels = props.payload?.channels_count ?? 1;
    const sampleRate = props.payload?.sample_rate ?? 1.0;
    const currentTime =
        (props.status &&
                (props.status.kind === "Playing" ||
                    props.status.kind === "Paused"))
            ? props.status.offset / sampleRate / channels
            : 0.0;

    const timeRemaining = totalDuration - currentTime;

    return (
        <div className="flex flex-row gap-2 justify-end">
            <span className="text-secondary">
                -{formatTime(timeRemaining)}
            </span>
            <span className="text-secondary">
                {formatTime(currentTime)}
            </span>
        </div>
    );
}

function formatTime(time: number) {
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function RangeSlider(props: {
    min: number;
    max: number;
    value: number;
    onChange: (value: number) => void;
}) {
    return (
        <div>
        </div>
    );
}

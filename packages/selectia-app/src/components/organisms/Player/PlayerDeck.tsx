import { useEffect, useMemo, useRef, useState } from "react";
import {
    DeckFileMetadataSnapshot,
    DeckFilePayloadSnapshot,
    DeckFilePreview,
    DeckFileStatus,
} from "../../../selectia-tauri/dto/models";
import { useDeck } from "../../../selectia-tauri/hooks/UseAudioPlayer";
import { IconPause } from "../../atoms/Icon";
import { IconPlay } from "../../atoms/Icon";
import { Spinner } from "../../atoms/Spinner";
import { Button } from "../../atoms/Button";

export function PlayerDeck(
    props: {
        deckId: number;
    },
) {
    const [metadata, payload, status, setStatus] = useDeck(props.deckId);

    const [statusKind, setStatusKind] = useState<DeckFileStatus["kind"] | null>(
        null,
    );

    const trackViewMemo = useMemo(() => {
        return (
            <TrackView
                payload={payload}
                status={status}
                setStatus={setStatus}
            />
        );
    }, [payload, status]);

    useEffect(() => {
        if (status?.kind !== statusKind) {
            setStatusKind(status?.kind ?? null);
        }
    }, [status]);

    if (!metadata) {
        return <div>Loading...</div>;
    }
    return (
        <div className="bg-slate-800">
            <div className="flex flex-col gap-2 relative">
                {trackViewMemo}
                <div className="flex justify-between items-center">
                    <div className="grow overflow-hidden">
                        <p className="text-white text-xs truncate">
                            {metadata.title}
                        </p>
                    </div>
                    <div className="shrink-0">
                        <TrackControls status={status} setStatus={setStatus} />
                    </div>
                </div>
            </div>
        </div>
    );
}

function TrackView(
    props: {
        payload: DeckFilePayloadSnapshot | null;
        status: DeckFileStatus | null;
        setStatus: (status: DeckFileStatus) => void;
    },
) {
    const trackBarRef = useRef<HTMLDivElement>(null);

    const [waveformSize, setWaveformSize] = useState({ width: 800, height: 200 });

    const progress = (props.payload && props.status) ? trackProgress(props.payload, props.status) : 0;


    const handleTrackBarClick = (e: React.MouseEvent<HTMLDivElement>) => {
        const rect = e.currentTarget.getBoundingClientRect();
        const offsetX = e.pageX - rect.left;

        if (props.payload) {
            const sampleOffset = Math.floor(
                offsetX / rect.width *
                    props.payload.samples_count,
            );
            console.log(sampleOffset);
            props.setStatus({ kind: "Playing", offset: sampleOffset });
        }
    };

    const handleResize = () => {
        const { width, height } = trackBarRef.current?.getBoundingClientRect() ?? { width: 800, height: 200 };
        if (width !== waveformSize.width || height !== waveformSize.height) {
            console.log(width, height);
            setWaveformSize({ width, height });
        }
    };

    const resizeObserver = new ResizeObserver((entries) => {
        handleResize();
    });

    if (trackBarRef.current) {
        resizeObserver.observe(trackBarRef.current);
    }

    return (
        <div ref={trackBarRef} className="w-full h-16 relative" onClick={(e) => handleTrackBarClick(e)}>
            <div
                className="bg-red-500 h-full"
                style={{ width: `${progress}%` }}
            >
            </div>
            <div className="absolute top-0 left-0">
                <AudioWaveform audioBuffer={props.payload?.preview ?? null} width={waveformSize.width} height={waveformSize.height} />
            </div>
        </div>
    );
}

function trackProgress(
    payload: DeckFilePayloadSnapshot,
    status: DeckFileStatus,
) {
    if (status.kind === "Playing") {
        return status.offset / payload.samples_count * 100;
    } else if (status.kind === "Paused") {
        return status.offset / payload.samples_count * 100;
    } else {
        return 0;
    }
}

function TrackControls(
    { status, setStatus }: {
        status: DeckFileStatus | null;
        setStatus: (status: DeckFileStatus) => void;
    },
) {
    if (status && status.kind === "Playing") {
        return (
            <div>
                <Button
                    onClick={() =>
                        setStatus({ kind: "Paused", offset: status.offset })}
                >
                    <IconPause />
                </Button>
            </div>
        );
    } else if (status && status.kind === "Paused") {
        return (
            <div>
                <Button
                    onClick={() =>
                        setStatus({ kind: "Playing", offset: status.offset })}
                >
                    <IconPlay />
                </Button>
            </div>
        );
    } else {
        return <Spinner />;
    }
}

function AudioWaveform(
    { audioBuffer, width, height }: {
        audioBuffer: DeckFilePreview | null;
        width: number;
        height: number;
    },
) {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        if (!audioBuffer) {
            console.error("No audio buffer");
            return;
        }
        const canvas: HTMLCanvasElement | null = canvasRef.current;
        if (!canvas) {
            console.error("No canvas");
            return;
        }

        const ctx = canvas.getContext("2d");

        if (!ctx) {
            console.error("No context");
            return;
        }
        // Clear the canvas
        ctx.clearRect(0, 0, width, height);

        const normalizedSamples = audioBuffer.samples.map((sample) =>
            sample * height / 2
        );
        ctx.beginPath();
        ctx.moveTo(0, height / 2); // Start in the middle

        normalizedSamples.forEach((sample, index) => {
            const x = (index / normalizedSamples.length) * width;
            const y = (height / 2) - sample;
            ctx.lineTo(x, y);
        });

        ctx.strokeStyle = "blue";
        ctx.lineWidth = 1;
        ctx.stroke();
    }, [audioBuffer, width, height]);

    return <canvas ref={canvasRef} width={width} height={height} />;
}

export default AudioWaveform;

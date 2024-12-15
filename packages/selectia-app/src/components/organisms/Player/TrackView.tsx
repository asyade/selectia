import { useEffect, useMemo, useRef, useState } from "react";
import { DeckFilePayloadSnapshot, DeckFilePreview, DeckFileStatus } from "../../../selectia-tauri/dto/models";
import { Button } from "../../atoms/Button";
import { IconZoomIn, IconZoomOut } from "../../atoms/Icon";
import { set_deck_file_status } from "../../../selectia-tauri";


export function TrackView(
    props: {
        deckId: bigint;
        payload: DeckFilePayloadSnapshot | null;
        status: DeckFileStatus | null;
    },
) {
    const trackBarRef = useRef<HTMLDivElement>(null);

    const [waveformSize, setWaveformSize] = useState<
        { width: number; height: number } | null
    >(null);

    const progress = (props.payload && props.status)
        ? trackProgress(props.payload, props.status)
        : 0;

    const handleTrackBarClick = (e: React.MouseEvent<HTMLDivElement>) => {
        const rect = e.currentTarget.getBoundingClientRect();
        const offsetX = e.pageX - rect.left;

        if (props.payload) {
            const sampleOffset = Math.floor(
                offsetX / rect.width *
                    props.payload.samples_count,
            );
            console.log(sampleOffset);
            set_deck_file_status(props.deckId, { kind: "Playing", offset: sampleOffset });
        }
    };

    const handleResize = () => {
        if (!trackBarRef.current) {
            return;
        }
        const { width, height } = trackBarRef.current.getBoundingClientRect();
        if (
            !waveformSize ||
            (width !== waveformSize.width || height !== waveformSize.height)
        ) {
            console.log(width, height);
            setWaveformSize({ width, height });
        }
    };

    const resizeObserver = new ResizeObserver((_entries) => {
        handleResize();
    });

    if (trackBarRef.current) {
        resizeObserver.observe(trackBarRef.current);
    }


    const samplesViewMemo = useMemo(() => (
        waveformSize && props.status?.kind !== "Loading" && (
            <SamplesView
                audioBuffer={props.payload?.preview ?? null}
                width={waveformSize.width}
                height={waveformSize.height}
                bpm={120.0}
                grid_offset={10}
            />
        ) || <div></div>
    ), [props.payload, waveformSize]);

    return (
        <div
            ref={trackBarRef}
            className="w-full relative flex-grow"
            onClick={(e) => handleTrackBarClick(e)}
        >
            <div className="absolute top-0 left-0">
                {samplesViewMemo}
            </div>
            

            {props.payload == null && (
                <div className="h-full flex items-center justify-center">
                    <span className="text-primary text-xs">No track loaded</span>
                </div>
            )}

            <div
                className="bg-slate-700 h-full"
                style={{ width: `${progress}%` }}
            >
            </div>
            <ViewControls />
        </div>
    );
}

function ViewControls(_props: {}) {
    return (
        <div className="absolute top-0 left-0 flex flex-row items-start justify-center">
            <Button onClick={() => {}}>
                <IconZoomIn />
            </Button>
            <Button onClick={() => {}}>
                <IconZoomOut />
            </Button>
        </div>
    )
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



 interface SamplesViewProps  {
    audioBuffer: DeckFilePreview | null;
    width: number;
    height: number;
    bpm: number;
    grid_offset: number;
}

 function SamplesView(
    { audioBuffer, width, height }: SamplesViewProps,
) {
    console.log("Redrawing samples view");
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
        ctx.beginPath();
        ctx.moveTo(0, height / 2); // Start in the middle

        const samplesPerPixel = Math.min(1, audioBuffer.samples.length / width);

        for (let i = 0; i < audioBuffer.samples.length; i += samplesPerPixel) {
            const sample = audioBuffer.samples[i];
            const normalized = sample * height / 2;
            const x = (i / audioBuffer.samples.length) * width;
            const y = (height / 2) - normalized;
            ctx.lineTo(x, y);
        }
        ctx.strokeStyle = "#7c3aed";
        ctx.lineWidth = 1;
        ctx.stroke();

    }, [audioBuffer, width, height]);

    return <canvas ref={canvasRef} width={width} height={height} />;
}

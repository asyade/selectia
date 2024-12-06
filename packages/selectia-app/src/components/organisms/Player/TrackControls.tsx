import { DeckFileStatus, DeckFileMetadataSnapshot } from "../../../selectia-tauri/dto/models";
import { Button } from "../../atoms/Button";
import { IconPause, IconPlay } from "../../atoms/Icon";
import { Spinner } from "../../atoms/Spinner";

export interface TrackControlsProps {
    status: DeckFileStatus | null;
    metadata: DeckFileMetadataSnapshot | null;
    setStatus: (status: DeckFileStatus) => void;
}

export function TrackControls(props: TrackControlsProps) {
    if (props.status?.kind == "Loading") {
        return <Spinner />;
    } else {
        return (
            <div>
                <PlayButton {...props} />
            </div>
        )
    }
}

function PlayButton({status, setStatus}: TrackControlsProps) {
    if (status && status.kind === "Playing") {
        return (
            <div>
                <Button
                    variant="outline"
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
                    variant="outline"
                    onClick={() =>
                        setStatus({ kind: "Playing", offset: status.offset })}
                >
                    <IconPlay />
                </Button>
            </div>
        );
    } else {
        return <></>
    }
}

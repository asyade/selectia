import { Button } from "../atoms/Button";
import {
    IconBack,
    IconGear,
    IconMinus,
    IconWindowMaximize,
    IconXmark,
} from "../atoms/Icon";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TextInput } from "../atoms/TextInput";

const appWindow = getCurrentWindow();

export interface ToolBarProps {
    currentPage: "manager" | "settings";
    onSettings: () => void;
}

export function ToolBar(props: ToolBarProps) {
    const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
        if (e.defaultPrevented) {
            return;
        }
        if (e.detail === 2) {
            appWindow.toggleMaximize();
        } else {
            appWindow.startDragging();
        }
    };

    return (
        <div
            onMouseDown={(e) => handleMouseDown(e)}
            className="flex flex-row justify-between items-center w-full p-2 bg-primary"
        >
            <ToolBarControls {...props} />
            <CommandBar className="flex-grow" />
            <WindowControls
                onMinimize={() => appWindow.minimize()}
                onMaximize={() => appWindow.toggleMaximize()}
                onClose={() => appWindow.close()}
            />
        </div>
    );
}

function ToolBarControls(props: ToolBarProps) {
    return (
        <div className="flex flex-row items-center justify-center">
            <Button variant="outline" onClick={props.onSettings}>
                {props.currentPage === "manager" && <IconGear />}
                {props.currentPage === "settings" && <IconBack />}
            </Button>
        </div>
    );
}

function CommandBar(props: {
    className?: string;
}) {
    return (
        <div className={`${props.className} max-w-96`}>
            <TextInput className="p-1 bg-slate-800" placeholder="Search..." />
        </div>
    );
}

function WindowControls(props: {
    onMinimize: () => void;
    onMaximize: () => void;
    onClose: () => void;
}) {
    return (
        <div className="flex flex-row justify-center items-center">
            <Button variant="outline" onClick={props.onMinimize}>
                <IconMinus />
            </Button>
            <Button variant="outline" onClick={props.onMaximize}>
                <IconWindowMaximize />
            </Button>
            <Button variant="outline" onClick={props.onClose}>
                <IconXmark />
            </Button>
        </div>
    );
}

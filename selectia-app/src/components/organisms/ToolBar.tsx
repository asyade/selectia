import { Button } from "../atoms/Button";
import { IconBack, IconGear, IconLogo, IconMinus, IconWindowMaximize, IconXmark } from "../atoms/Icon";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { TextInput } from "../atoms/TextInput";

const appWindow = getCurrentWindow();

export function ToolBar(props: {
    currentPage: "manager" | "settings";
    onSettings: () => void;
}) {

    const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
        if (e.defaultPrevented) {
            return;
        }
        if (e.detail === 2) {
            appWindow.toggleMaximize();
        } else {
            appWindow.startDragging();
        }
    }

    return (
        <div onMouseDown={(e) => handleMouseDown(e)} className="flex flex-row justify-between items-center w-full bg-slate-900 p-4">
            <div className="w-24 flex flex-row items-center gap-2">
                
                <Button variant="outline" onClick={props.onSettings}>
                    {props.currentPage === "manager" && <IconGear />}
                    {props.currentPage === "settings" && <IconBack />}
                </Button>
            </div>
            <CommandBar className="flex-grow" />
            <WindowControls
                onMinimize={() => appWindow.minimize()}
                onMaximize={() => appWindow.toggleMaximize()}
                onClose={() => appWindow.close()}
            />
        </div>
    );
}

function CommandBar(props: {
    className?: string;
}) {
    return <div className={`${props.className} max-w-96`}>
        <TextInput className="p-1 bg-slate-800" placeholder="Search..." />
    </div>;
}

function WindowControls(props: {
    onMinimize: () => void;
    onMaximize: () => void;
    onClose: () => void;
}) {
    return (
        <div className="flex flex-row justify-center items-center bg-slate-900">
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
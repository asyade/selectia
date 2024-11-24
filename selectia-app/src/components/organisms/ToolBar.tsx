import { Button } from "../atoms/Button";
import { IconCircleXMark, IconLogo, IconMinus, IconSquare, IconWindowMaximize, IconXmark } from "../atoms/Icon";
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

export function ToolBar() {

    const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
        if (e.detail === 2) {
            appWindow.toggleMaximize();
        } else {
            appWindow.startDragging();
        }
    }

    const handleMinimize = () => {
        console.log("Minimize");
        appWindow.minimize();
    }

    const handleMaximize = () => {
        appWindow.toggleMaximize();
    }

    const handleClose = () => {
        appWindow.close();
    }

    return (
        <div className="flex flex-row justify-between items-center w-full bg-slate-900 pl-3 pr-3 fixed top-0 left-0 right-0 h-12">
            <div className="grow" onMouseDown={(e) => handleMouseDown(e)}>
                <IconLogo />
            </div>
            <WindowControls onMinimize={handleMinimize} onMaximize={handleMaximize} onClose={handleClose} />
        </div>
    );
}

function WindowControls(props: {
    onMinimize: () => void;
    onMaximize: () => void;
    onClose: () => void;
}) {
    return (
        <div className="flex flex-row justify-center items-center bg-slate-900">
            <Button variant="outline" onClick={props.onMinimize}>
                <IconMinus/>
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
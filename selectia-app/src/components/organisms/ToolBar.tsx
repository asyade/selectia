import { Button } from "../atoms/Button";
import { IconLogo, IconMinus, IconTaskManager, IconWindowMaximize, IconXmark } from "../atoms/Icon";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { TaskManager } from "./TaskManager";
import { DropDownButton } from "../molecules/DropDownButton";

const appWindow = getCurrentWindow();

export function ToolBar() {

    const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
        if (e.detail === 2) {
            appWindow.toggleMaximize();
        } else {
            appWindow.startDragging();
        }
    }

    return (
        <div className="flex flex-row justify-between items-center w-full bg-slate-900 pl-3 pr-3 fixed top-0 left-0 right-0 h-12">
            <div className="grow" onMouseDown={(e) => handleMouseDown(e)}>
                <IconLogo />
            </div>
            <DropDownButton buttonContent={<IconTaskManager />} variant="outline">
                <TaskManager />
            </DropDownButton>
            <WindowControls
                onMinimize={() => appWindow.minimize()}
                onMaximize={() => appWindow.toggleMaximize()}
                onClose={() => appWindow.close()}
            />
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
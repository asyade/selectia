import { useState } from "react";

export function Statusbar(props: {
    className?: string;
}) {
    const [status, _setStatus] = useState("Idle");

    return <div className={`${props.className} p-2 bg-slate-800`}>
        <div className="flex flex-row">
            <span className="text-slate-400">{status}</span>
        </div>
    </div>;
}
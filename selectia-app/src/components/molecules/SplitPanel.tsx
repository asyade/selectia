export function SplitPanel(props: {
    className?: string;
    left: React.ReactNode;
    right: React.ReactNode;
}) {
    return <div className={`${props.className} flex flex-row w-full h-full max-h-full bg-slate-900`}>
        <div className="flex-none">
            {props.left}
        </div>
        <div className="flex-grow overflow-scroll">
            {props.right}
        </div>
    </div>;
}
import { useMemo, useState } from "react";

export function DropZone(props: {children: React.ReactNode, onDrop: (files: FileList) => void}) {
    const [isDragging, setIsDragging] = useState(false);
    const [files, setFiles] = useState([]);

    const handleDragEnter = (event: React.DragEvent<HTMLDivElement>) => {
        event.preventDefault();
        event.stopPropagation();
        // setIsDragging(true);
    };

    const handleDragLeave = (event: React.DragEvent<HTMLDivElement>) => {
        event.preventDefault();
        event.stopPropagation();
        // setIsDragging(false);
    };

    const handleDragOver = (event: React.DragEvent<HTMLDivElement>) => {
        event.preventDefault();
        event.stopPropagation();
    };

    const handleDrop = (event: React.DragEvent<HTMLDivElement>) => {
        event.preventDefault();
        event.stopPropagation();
        const droppedFiles = Array.from(event.dataTransfer.files);
        // setFiles((prevFiles) => [...prevFiles, ...droppedFiles]);
    };

    const dragbox = useMemo(() => {
        return (
            isDragging && ( 
                <div className="opacity-50 w-full h-full bg-slate-500/50 backdrop-blur-sm absolute left-0 top-0 flex flex-col items-center justify-center">
                    <span>Drop files here</span>
                </div>
            )
        );
    }, [isDragging]);
    
    const children = useMemo(() => {
        return (
            <div className="w-full h-full relative">{props.children}</div>
        );
    }, [props.children]);

    return (
        <div
            onDragEnter={handleDragEnter}
            onDragLeave={handleDragLeave}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
            className="w-full h-full relative"
        >
            {/* {dragbox} */}
            {children}
        </div>
    );
}
import { useMemo, useState } from "react";
import { TagView } from "../../../selectia-tauri/dto/models";
import { useTags } from "../../../selectia-tauri/hooks/UseTags";
import { IconFolderOpenOutline, IconFolderOutline, } from "../../../components";

export function FilterSubSectionDirectories(props: {
    className?: string;
}) {
    const [tags] = useTags("directory");

    const tagAsDirectoryCursor = new TagAsDirectoryCursor(tags);


    const treeItems = useMemo(() => tagAsDirectoryCursor.root.children.map(x => {
        const id = x.path();
        return <TreeItem key={id} node={x} />
    }), [tags]);
    console.log(treeItems);
    return <div className={`${props.className} flex flex-wrap flex-col`}>
        {treeItems}
    </div>;
}

function TreeItem(props: {
    node: DirectoryCursorNode;
}) {
    const [open, setOpen] = useState(false);
    const [selected, setSelected] = useState(false);

    const children = props.node.children.map(x => <TreeItem key={x.path()} node={x} />);

    const handleClick = (e: React.MouseEvent<HTMLDivElement>) => {
        if (e.ctrlKey || e.metaKey || e.detail === 2) {
            setSelected(!selected);
        } else {
            setOpen(!open);
        }
    }

    return (
        <>
            <div onClick={(e) => handleClick(e)} key={props.node.tag?.id} className={`${selected ? "box-border outline outline-2 outline-dashed outline-slate-500" : ""} flex flex-row justify-start items-center cursor-pointer hover:bg-slate-700`}>
                <div className="p-1">
                    {open && children.length > 0 ? <IconFolderOpenOutline /> : <IconFolderOutline />}
                </div>
                <span className="text-sm/2 text-slate-500 truncate ">{props.node.title}</span>
            </div>
            {children.length > 0 && open && <div className="pl-2">
                {children}
            </div>}
        </>
    );
}

class TagAsDirectoryCursor {
    root: DirectoryCursorNode;
    constructor(tags: TagView[]) {
        this.root = new DirectoryCursorNode("Root", []);
        for (let tag of tags) {
            const normalizedPath = tag.value.replace(/\\/g, "/");
            let splited = normalizedPath.split("/");
            let current = this.root;
            for (let i = 0; i < splited.length; i++) {
                let next = current.children.find(x => x.title === splited[i]);
                if (!next) {
                    next = new DirectoryCursorNode(splited[i], []);
                    current.children.push(next);
                }
                current = next;
                next.tag = tag;
            }
        }
        // this.normalize();
    }

    normalize() {
        while (this.root.children.length === 1) {
            this.root.prefix = (this.root.prefix ?? "") + this.root.title + "/";
            this.root = this.root.children[0];

        }
    }
}

class DirectoryCursorNode {
    constructor(
        public title: string,
        public children: DirectoryCursorNode[],
        public tag?: TagView,
        public prefix?: string,
    ) {
    }

    path(): string {
        return (this.prefix ?? "") + this.title;
    }

}
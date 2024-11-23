import { Button } from "../atoms/Button.tsx";
import { IconCirclePlus } from "../atoms/Icon.tsx";

import { open } from '@tauri-apps/plugin-dialog';

export function ActionBar(props: {
    className?: string;
}) {

    const handleAdd = async () => {
        const result = await open({
            title: "Add new item",
            directory: true,
            multiple: true,
        });
        console.log(result);
    }
   
   return <div className={`${props.className} p-2 bg-slate-800`}>
        <div className="flex flex-row">

        </div>
    </div>;
}
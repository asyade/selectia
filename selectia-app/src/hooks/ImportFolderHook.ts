import { invoke } from '@tauri-apps/api/core';

export function useFolderImport() {
    const importFolder = async (directory: String): Promise<boolean> => {
        return invoke("import_folder", { directory })
    }

    return [importFolder]
}
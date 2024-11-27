import { import_folder } from "../index";

export function useFolderImport() {
    const importFolder = async (directory: string): Promise<boolean> => {
        return import_folder(directory);
    }

    return [importFolder]
}
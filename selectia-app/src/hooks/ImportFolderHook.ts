import { import_folder } from "../selectia-rs";

export function useFolderImport() {
    const importFolder = async (directory: string): Promise<boolean> => {
        return import_folder(directory);
    }

    return [importFolder]
}
import { TextInput } from "..";

export function SettingsPage() {
    return <div className="flex-grow bg-secondary">
        <SettingsSection title="Audio">
            <Setting>
                <SettingTitle>Volume</SettingTitle>
                <SettingDescription>
                    Régle le volume de la musique.
                </SettingDescription>
                <SettingInput>

                </SettingInput>
            </Setting>
        </SettingsSection>
        <SettingsSection title="Fichiers">
            <Setting>
                <SettingTitle>Base de données</SettingTitle>
                <SettingDescription>
                    Sélectionne le dossier ou est stockée votre base de données (contiens tous vos paramètres, collections, tags, etc.)
                </SettingDescription>
                <SettingInput>
                    <TextInput
                        value={""}
                        onChange={() => {}}
                    />
                </SettingInput>
            </Setting>
        </SettingsSection>
    </div>;
}

function SettingsSection(props: {
    title: string;
    children: React.ReactNode;
}) {
    return <div className="flex flex-col gap-2">
        <h2>{props.title}</h2>
        {props.children}
    </div>;
}

function Setting(props: {
    children: React.ReactNode;
}) {
    return <div className="flex flex-col gap-2">
        {props.children}
    </div>;
}


function SettingTitle(props: {
    children?: React.ReactNode;
}) {
    return <h3 className="text-lg font-semibold text-gray-300">{props.children}</h3>;
}

function SettingDescription(props: {
    children?: React.ReactNode;
}) {
    return <p className="text-sm text-gray-400">{props.children}</p>;
}

function SettingInput(props: {
    children?: React.ReactNode;
}) {
    return <div className="flex flex-col gap-2">{props.children}</div>;
}

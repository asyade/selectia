import { useMemo, useState } from "react";
import { Button } from "./Button";

export function TextInput(props: {
    className?: string,
    onChange?: (value: string) => void,
    autoFocus?: boolean,
    onSubmit?: () => void,
    value?: string,
    placeholder?: string,
    suggestedValues?: string[],
}) {
    const [selectedSuggestion, setSelectedSuggestion] = useState<number>(0);

    const selectionDelta = (delta: number) => {
        if (!props.suggestedValues) {
            return;
        }

        const updated = selectedSuggestion + delta;
        if (updated < 0) {
            setSelectedSuggestion(props.suggestedValues.length - 1);
        } else if (updated >= props.suggestedValues.length) {
            setSelectedSuggestion(0);
        } else {
            setSelectedSuggestion(updated);
        }
    }

    const suggestionElement = useMemo(() => {
        if (!props.suggestedValues || props.suggestedValues.length === 0) {
            return null;
        }
        return (
            <TextInputSuggestions
                selectedSuggestion={selectedSuggestion}
                suggestions={props.suggestedValues}
                onSelect={(suggestion) => props.onChange?.(suggestion)}
            />
        );
    }, [props.suggestedValues, selectedSuggestion]);

    return (
        <div className="p-0 m-0">
            <input
                placeholder={props.placeholder}
                autoFocus={props.autoFocus}
                className={`
                    ${props.className} 
                    transition duration-300 ease text-sm w-full placeholder:text-slate-300 text-white text-sm
                    border-none rounded-md focus:outline-none focus:border-none hover:border-none shadow-none focus:shadow-none
                    `}
                onKeyDown={(e) => {
                    if (e.key === "ArrowUp") {
                        selectionDelta(-1);
                        e.preventDefault();
                    } else if (e.key === "ArrowDown") {
                        selectionDelta(1);
                        e.preventDefault();
                    } else if (e.key === "Escape") {
                    } else if (e.key === "Enter") {
                        if (
                            props.suggestedValues &&
                            props.suggestedValues.length > 0 &&
                            props.suggestedValues[selectedSuggestion] !== props.value
                        ) {
                            props.onChange?.(props.suggestedValues[selectedSuggestion]);
                        } else {
                            props.onSubmit?.();
                        }
                    }
                }}
                onChange={(e) => props.onChange?.(e.target.value)}
                value={props.value}
            />
            {
                suggestionElement && suggestionElement
            }
        </div>
    );
}

function TextInputSuggestions(props: {
    suggestions: string[],
    onSelect: (suggestion: string) => void,
    selectedSuggestion: number,
}) {
    return <div className="absolute bg-slate-800 p-2 rounded flex flex-col absolute shadow-lg border border-slate-700">
        {
            props.suggestions.map((suggestion, index) => (
                <Button className={`${index === props.selectedSuggestion ? "bg-slate-700" : ""}`} variant="outline" onClick={() => props.onSelect(suggestion)}>
                    <span className="text-slate-400 text-left w-full">{suggestion}</span>
                </Button>
            ))
        }
    </div>;
}

import { useState } from "react";
import { Button, ButtonVariant } from "../atoms/Button";
import { DropDown } from "../atoms/DropDown";

export function DropDownButton(props: {
    buttonContent: React.ReactNode;
    children: React.ReactNode[] | React.ReactNode;
    variant?: ButtonVariant;
    dropDownClassName?: string;
    className?: string;
}) {
    const [showDropDown, setShowDropDown] = useState(false);

    return (
        <div className={`${props.className}`}>
            <Button variant={props.variant} onClick={() => setShowDropDown(!showDropDown)}>{props.buttonContent}</Button>
            {showDropDown && (
                <DropDown className={props.dropDownClassName} onClose={() => setShowDropDown(false)}>
                    {props.children}
                </DropDown>
            )}
        </div>
    );
}

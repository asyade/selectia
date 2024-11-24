import { useState } from "react";
import { Button, ButtonVariant } from "../atoms/Button";
import { DropDown } from "../atoms/DropDown";

export function DropDownButton(props: {
    buttonContent: React.ReactNode;
    children: React.ReactNode[] | React.ReactNode;
    variant?: ButtonVariant;
}) {
    const [showDropDown, setShowDropDown] = useState(false);

    return (
        <div>
            <Button variant={props.variant} onClick={() => setShowDropDown(!showDropDown)}>{props.buttonContent}</Button>
            {showDropDown && (
                <DropDown onClose={() => setShowDropDown(false)}>
                    {props.children}
                </DropDown>
            )}
        </div>
    );
}

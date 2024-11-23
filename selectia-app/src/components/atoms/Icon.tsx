import { FaCirclePlus } from "react-icons/fa6";

interface IconProps {
    color?: string;
}

export function IconCirclePlus(props: IconProps) {
    return <FaCirclePlus color={props.color ? props.color : "#F8FAFC"} />;
}
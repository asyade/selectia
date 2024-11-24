import { FaChevronDown, FaChevronUp, FaCirclePlus, FaCircleXmark, FaEye, FaEyeSlash, FaMinus, FaPen, FaSquare, FaWindowMaximize, FaXmark } from "react-icons/fa6";

interface IconProps {
    color?: string;
}

export function IconCirclePlus(props: IconProps) {
    return <FaCirclePlus color={props.color ? props.color : "#4b5563"} />;
}

export function IconEye(props: IconProps) {
    return <FaEye color={props.color ? props.color : "#4b5563"} />;
}

export function IconEyeSlash(props: IconProps) {
    return <FaEyeSlash color={props.color ? props.color : "#4b5563"} />;
}

export function IconChevronDown(props: IconProps) {
    return <FaChevronDown color={props.color ? props.color : "#4b5563"} />;
}

export function IconChevronUp(props: IconProps) {
    return <FaChevronUp color={props.color ? props.color : "#4b5563"} />;
}

export function IconEdit(props: IconProps) {
    return <FaPen color={props.color ? props.color : "#4b5563"} />;
}

export function IconLogo(props: IconProps) {
    return <img src="/src/assets/logo.png" alt="Selectia" className="w-6 h-6 mr-2" />;
}


export function IconSquare(props: IconProps) {
    return <FaSquare color={props.color ? props.color : "#4b5563"} />;
}

export function IconCircleXMark(props: IconProps) {
    return <FaCircleXmark color={props.color ? props.color : "#4b5563"} />;
}

export function IconXmark(props: IconProps) {
    return <FaXmark color={props.color ? props.color : "#4b5563"} />;
}


export function IconWindowMaximize(props: IconProps) {
    return <FaWindowMaximize color={props.color ? props.color : "#4b5563"} />;
}

export function IconMinus(props: IconProps) {
    return <FaMinus color={props.color ? props.color : "#4b5563"} />;
}

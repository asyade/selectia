
import { MdAdd, MdFolder, MdFolderOpen, MdOutlineFolder, MdOutlineFolderOpen } from "react-icons/md";
import { FaArrowLeft, FaTasks, FaTrash } from "react-icons/fa";
import { FaChevronDown, FaChevronUp, FaCircleXmark, FaEye, FaEyeSlash, FaGear, FaMinus, FaPen, FaSquare, FaWindowMaximize, FaXmark } from "react-icons/fa6";

interface IconProps {
    color?: string;
    width?: number;
    height?: number;
}

export function IconCirclePlus(props: IconProps) {
    return <MdAdd width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconEye(props: IconProps) {
    return <FaEye width={props.width} height={props.height}     color={props.color ? props.color : "#4b5563"} />;
}

export function IconEyeSlash(props: IconProps) {
    return <FaEyeSlash width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconChevronDown(props: IconProps) {
    return <FaChevronDown width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconChevronUp(props: IconProps) {
    return <FaChevronUp width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconEdit(props: IconProps) {
    return <FaPen width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconLogo(_props: IconProps) {
    return <img src="/src/assets/logo.png" alt="Selectia" className="w-6 h-6 mr-2"></img>;
}


export function IconSquare(props: IconProps) {
    return <FaSquare width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconCircleXMark(props: IconProps) {
    return <FaCircleXmark width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconXmark(props: IconProps) {
    return <FaXmark width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}


export function IconWindowMaximize(props: IconProps) {
    return <FaWindowMaximize width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconMinus(props: IconProps) {
    return <FaMinus width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconTaskManager(props: IconProps) {
    return <FaTasks width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconTrash(props: IconProps) {
    return <FaTrash width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconFolder(props: IconProps) {
    return <MdFolder width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconFolderOpen(props: IconProps) {
    return <MdFolderOpen width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconFolderOpenOutline(props: IconProps) {
    return <MdOutlineFolderOpen width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconFolderOutline(props: IconProps) {
    return <MdOutlineFolder width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconGear(props: IconProps) {
    return <FaGear width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}

export function IconBack(props: IconProps) {
    return <FaArrowLeft width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} />;
}
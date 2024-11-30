
import { MdAdd, MdFolder, MdFolderOpen, MdOutlineFolder, MdOutlineFolderOpen } from "react-icons/md";
import { FaArrowLeft, FaChevronRight, FaPause, FaPlay, FaTasks, FaTrash } from "react-icons/fa";
import { FaChevronDown, FaChevronUp, FaCircleXmark, FaEye, FaEyeSlash, FaGear, FaMinus, FaPen, FaSquare, FaWindowMaximize, FaXmark } from "react-icons/fa6";

interface IconProps {
    color?: string;
    width?: number;
    height?: number;
    className?: string;
}

export function IconCirclePlus(props: IconProps) {
    return <MdAdd width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconEye(props: IconProps) {
    return <FaEye width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconEyeSlash(props: IconProps) {
    return <FaEyeSlash width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconChevronDown(props: IconProps) {
    return <FaChevronDown width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconChevronUp(props: IconProps) {
    return <FaChevronUp width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconChevronRight(props: IconProps) {
    return <FaChevronRight width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconEdit(props: IconProps) {
    return <FaPen width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconLogo(_props: IconProps) {
    return <img src="/src/assets/logo.png" alt="Selectia" className="w-6 h-6 mr-2" />;
}


export function IconSquare(props: IconProps) {
    return <FaSquare width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconCircleXMark(props: IconProps) {
    return <FaCircleXmark width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconXmark(props: IconProps) {
    return <FaXmark width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}


export function IconWindowMaximize(props: IconProps) {
    return <FaWindowMaximize width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconMinus(props: IconProps) {
    return <FaMinus width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconTaskManager(props: IconProps) {
    return <FaTasks width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconTrash(props: IconProps) {
    return <FaTrash width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconFolder(props: IconProps) {
    return <MdFolder width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconFolderOpen(props: IconProps) {
    return <MdFolderOpen width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconFolderOpenOutline(props: IconProps) {
    return <MdOutlineFolderOpen width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconFolderOutline(props: IconProps) {
    return <MdOutlineFolder width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconGear(props: IconProps) {
    return <FaGear width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconBack(props: IconProps) {
    return <FaArrowLeft width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconPlay(props: IconProps) {
    return <FaPlay width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

export function IconPause(props: IconProps) {
    return <FaPause width={props.width} height={props.height} color={props.color ? props.color : "#4b5563"} className={props.className} />;
}

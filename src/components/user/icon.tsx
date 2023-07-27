import { JSX } from "preact/jsx-runtime";
import { Job } from "./profile.tsx";

export interface IconProps {
  job: Job;
  height:
    | string
    | number
    | JSX.SignalLike<string | number | undefined>
    | undefined;
  width:
    | string
    | number
    | JSX.SignalLike<string | number | undefined>
    | undefined;
  style?:
    | string
    | JSX.CSSProperties
    | JSX.SignalLike<string | JSX.CSSProperties | undefined>
    | undefined;
}

function ref_img(job: Job) {
  if (job === "citizen") return "/img/citizen.webp";
  else if (job === "wolf") return "/img/wolf.webp";
  else if (job === "seer") return "/img/seer.webp";
  else if (job === "hunter") return "/img/hunter.webp";
}

export default function Icon(props: IconProps) {
  return (
    <img
      style={props.style}
      height={props.height}
      width={props.width}
      src={ref_img(props.job)}
    />
  );
}

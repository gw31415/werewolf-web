import { Job } from "./profile.tsx";

export interface IconProps {
  job?: Job | undefined;
  size: string | number;
}

function ref_img(job: Job) {
  if (job === "citizen") return "/img/citizen.webp";
  else if (job === "wolf") return "/img/wolf.webp";
  else if (job === "seer") return "/img/seer.webp";
  else if (job === "hunter") return "/img/hunter.webp";
}

export default function Icon(props: IconProps) {
  return (
    props.job
      ? (
        <img
          style={{
            display: "block",
            borderRadius: ".25rem",
            width: props.size,
            height: props.size,
          }}
          height="128"
          width="128"
          src={ref_img(props.job)}
        />
      )
      : (
        <div
          style={{
            backgroundColor: "black",
            borderRadius: ".25rem",
            lineHeight: 1,
            textAlign: "center",
            verticalAlign: "bottom",
            width: props.size,
            height: props.size,
            fontSize: props.size,
          }}
        >
          ?
        </div>
      )
  );
}

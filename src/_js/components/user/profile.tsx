import { JSX } from "preact/jsx-runtime";
import Icon from "./icon.tsx";

export type Job = "citizen" | "wolf" | "seer" | "hunter";

function display(job: Job) {
  if (job === "citizen") return "市民";
  else if (job === "wolf") return "人狼";
  else if (job === "seer") return "占い師";
  else if (job === "hunter") return "狩人";
}

export interface ProfileProps {
  name: string;
  job?: Job | undefined;
  style?:
    | string
    | JSX.CSSProperties
    | JSX.SignalLike<string | JSX.CSSProperties | undefined>
    | undefined;
}

export default function Profile(props: ProfileProps) {
  return (
    <div style={props.style}>
      <div
        style={{
          height: "5rem",
          display: "flex",
          verticalAlign: "middle",
          alignItems: "center",
        }}
      >
        <Icon
          size="3.5rem"
          job={props.job}
        />
        <div style={{ paddingLeft: ".6rem" }}>
          <h1>{props.name}</h1>
          <span
            style={{
              paddingLeft: ".25em",
              visibility: props.job ? "visible" : "hidden",
            }}
          >
            役職：{props.job ? display(props.job) : ""}
          </span>
        </div>
      </div>
    </div>
  );
}

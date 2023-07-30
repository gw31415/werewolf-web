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
  job: Job;
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
          style={{
            display: "block",
            backgroundColor: "white",
            borderRadius: ".25rem",
            width: "3.5rem",
            height: "3.5rem",
          }}
          job="citizen"
        />
        <div style={{ paddingLeft: ".5rem" }}>
          <h1>{props.name}</h1>
          <span style={{ paddingLeft: ".25em" }}>
            役職：{display(props.job)}
          </span>
        </div>
      </div>
    </div>
  );
}

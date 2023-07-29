import { JSX } from "preact/jsx-runtime";
import { css } from "https://esm.sh/@emotion/css@11.11.0";

export interface MembersProps {
  members: string[];
  online: string[];
  master: string;
  style?:
    | string
    | JSX.CSSProperties
    | JSX.SignalLike<string | JSX.CSSProperties | undefined>
    | undefined;
}

const tdStyle = css({
  textAlign: "center",
  paddingRight: ".5rem",
  verticalAlign: "top",
});

export default function Members(props: MembersProps) {
  return (
    <div style={props.style}>
      <div>{props.master}</div>
      <table>
        {props.members.sort().map((m) => {
          const isOnline = props.online.includes(m);
          return (
            <tr>
              <td
                className={tdStyle}
                style={{
                  color: isOnline ? "#4CAF50" : "#F44336",
                }}
              >
                <span
                  class="material-symbols-outlined"
                  style={{ fontSize: ".7rem" }}
                >
                  {isOnline ? "signal_cellular_4_bar" : "signal_cellular_off"}
                </span>
              </td>
              <td>{m}</td>
            </tr>
          );
        })}
      </table>
    </div>
  );
}

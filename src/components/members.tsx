import { JSX } from "preact/jsx-runtime";

export interface MembersProps {
  members: string[];
  online: string[];
  style?:
    | string
    | JSX.CSSProperties
    | JSX.SignalLike<string | JSX.CSSProperties | undefined>
    | undefined;
}

export default function Members(props: MembersProps) {
  return (
    <div style={props.style}>
      <table>
        {props.members.map((m) => {
          const isOnline = props.online.includes(m);
          return (
            <tr>
              <td
                style={{
                  color: isOnline ? "#4CAF50" : "#F44336",
                  textAlign: "center",
                  paddingRight: ".5rem",
                  verticalAlign: "top",
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

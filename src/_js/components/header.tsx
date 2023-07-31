import { GameComponentProps } from "../ws.ts";
import Members from "./members.tsx";
import Profile from "./user/profile.tsx";

export default function Header(props: GameComponentProps) {
  return (
    <div style={{ display: "flex", justifyContent: "space-around" }}>
      {props.logined
        ? (
          // 認証済み
          <>
            <Profile
              style={{ marginTop: ".5rem" }}
              name={props.name}
              job={props.state.day
                ? props.state.day.role[props.name]
                : (props.state.night
                  ? props.state.night.role[props.name]
                  : undefined)}
            />
            <Members
              online={props.online}
              members={props.members}
              master={props.master}
              style={{
                fontSize: ".8rem",
                width: "6.5rem",
                minHeight: "5rem",
                padding: ".5em",
                borderRight: 1,
                borderLeft: 1,
                borderStyle: "solid",
                borderImage:
                  "linear-gradient(to bottom, #181818, #eee, #181818) 1 100%",
              }}
            />
          </>
        )
        : (
          <div
            style={{ height: "5rem", alignItems: "center", display: "flex" }}
          >
            <h1>Werewolf Online</h1>
          </div>
        )}
    </div>
  );
}

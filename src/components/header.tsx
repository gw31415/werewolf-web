import { AuthData } from "../app.tsx";
import Members from "./members.tsx";
import Profile from "./user/profile.tsx";

export interface HeaderProps {
  members: string[];
  online: string[];
  auths: AuthData | null;
}

export default function Header(props: HeaderProps) {
  return (
    <div style={{ display: "flex", justifyContent: "space-around" }}>
      {props.auths !== null
        ? (
          // 認証済み
          <>
            <Profile
              style={{ marginTop: ".5rem" }}
              name={props.auths.name}
              job="citizen"
            />
            <Members
              online={props.online}
              members={props.members}
              master={props.auths.master}
              style={{
                fontSize: ".8rem",
                width: "8rem",
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

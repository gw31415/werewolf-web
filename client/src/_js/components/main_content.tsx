import { GameComponentProps } from "../ws.ts";
import EnteringForm from "./entering_form.tsx";

export default function MainContent(props: GameComponentProps) {
  return (
    <div style={{ display: "flex", justifyContent: "space-around", flex: 1 }}>
      {props.logined
        ? (
          // 認証済み
          <></>
        )
        : (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
            }}
          >
            <div>
              <EnteringForm />
            </div>
          </div>
        )}
    </div>
  );
}

import { useId } from "preact/hooks";
import { NameSet } from "../app.tsx";
import { css } from "https://esm.sh/@emotion/css@11.11.0";

// deno-lint-ignore no-explicit-any
export type Sender = (request: any) => void;

export interface MainContentProps {
  members: string[];
  online: string[];
  auths: NameSet | null;
  sender: Sender;
}

const Form = (props: { sender: Sender }) => {
  const nameInput = useId();
  const roomInput = useId();
  return (
    <div>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          props.sender({
            connect: {
              signup: {
                name: (document.getElementById(nameInput)! as HTMLInputElement).value,
                master: (document.getElementById(roomInput)! as HTMLInputElement).value,
              },
            },
          });
        }}
        className={css({
          "--brand": "gold",

          width: "20rem",
          maxWidth: "100vw",
          height: "8rem",

          ">div": {
            display: "flex",
            height: "50%",
          },

          button: {
            backgroundColor: "transparent",
            cursor: "pointer",
            outline: "none",
            appearance: "none",
            border: "solid 1px",
            borderLeft: "none",
            borderRight: "none",
            borderColor: "white",
            width: "50%",
            padding: ".2rem",
            fontSize: "1.2rem",
            ":hover, :focus": {
              padding: ".5rem",
            },
            opacity: 0,
            visibility: "hidden",
            transition: "opacity .5s, visibility 0s .5s",
          },
          ":valid button": {
            visibility: "unset",
            opacity: 1,
            transition: "opacity .5s, padding .2s, visibility 0s 0s",
          },
        })}
      >
        <div
          className={css(
            {
              ">div": {
                padding: ".5rem 2rem",
                position: "relative",
                label: {
                  position: "absolute",
                  padding: "0 .5rem",
                  top: "50%",
                  left: "50%",
                  transform: "translateY(-50%) translateX(-50%)",
                  pointerEvents: "none",
                  transition: "transform .3s",
                  opacity: .5,
                },
                '[type="text"]': {
                  appearance: "none",
                  color: "currentcolor",
                  backgroundColor: "transparent",
                  border: 0,
                  boxShadow: "none",
                  outline: "0 !important",
                  height: "100%",
                  width: "100%",
                  textAlign: "center",
                  ":focus+label, :valid+label": {
                    top: 0,
                    left: 0,
                    transform: "scale(.75)",
                  },
                  ":focus+label": {
                    color: "var(--brand)",
                    opacity: 1,
                  },
                },
              },
            },
          )}
        >
          <div>
            <input
              type="text"
              id={nameInput}
              required
              maxLength={5}
            />
            <label for={nameInput}>名前</label>
          </div>
          <div>
            <input
              type="text"
              id={roomInput}
              required
              maxLength={5}
              pattern="\S+"
            />
            <label for={roomInput}>部屋</label>
          </div>
        </div>
        <div
          style={{
            alignItems: "center",
            verticalAlign: "middle",
            justifyContent: "center",
          }}
        >
          <button
            type="submit"
            onTouchEnd={(e) => {
              e.currentTarget.style.padding = ".2em";
            }}
            onTouchStart={(e) => {
              e.currentTarget.style.padding = ".5em";
            }}
          >
            入室する
          </button>
        </div>
      </form>
    </div>
  );
};

export default function MainContent(props: MainContentProps) {
  return (
    <div style={{ display: "flex", justifyContent: "space-around", flex: 1 }}>
      {props.auths !== null
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
              <Form sender={props.sender} />
            </div>
          </div>
        )}
    </div>
  );
}

import { ContainerNode, render } from "preact";
import { useEffect, useState } from "preact/hooks";
import Header from "./components/header.tsx";
import MainContent from "./components/main_content.tsx";
import { css } from "https://esm.sh/@emotion/css@11.11.0";

export interface NameSet {
  name: string;
  master: string;
}

function App() {
  const [members, setMembers] = useState([]);
  const [online, setOnline] = useState([]);
  const [_state, setState] = useState(null);
  const [auths, setAuths] = useState<NameSet | null>(null);
  let ws: WebSocket | null = null;

  useEffect(() => {
    ws = new WebSocket("ws://localhost:3232/ws");
    ws.addEventListener("open", () => {
      const token = localStorage.getItem("token");
      if (token) {
        ws!.send(JSON.stringify({
          connect: { token: token },
        }));
      }
    });
    ws.addEventListener("message", (m) => {
      const res = JSON.parse(m.data);
      if (res.success) {
        if (res.success.members) {
          setMembers(res.success.members);
        } else if (res.success.online) {
          setOnline(res.success.online);
        } else if (res.success.state) {
          setState(res.success.state);
        } else if (res.success.authenticationSuccess) {
          localStorage.setItem(
            "token",
            res.success.authenticationSuccess.token,
          );
          setAuths({
            master: res.success.authenticationSuccess.master,
            name: res.success.authenticationSuccess.name,
          });
        }
      } else if (res.error) {
        if (res.error.session === "authenticationFailed") {
          localStorage.removeItem("token");
          setAuths(null);
        } else {
          console.error(res.error);
        }
      }
    });
  }, []);
  return (
    <>
      <Header
        members={members}
        auths={auths}
        online={online}
      />
      <MainContent
        members={members}
        auths={auths}
        online={online}
        sender={(req) => ws!.send(JSON.stringify(req))}
      />
    </>
  );
}

const appRoot = document.getElementById("app")!;
appRoot.innerText = "";

render(
  <App />,
  appRoot as ContainerNode,
);

appRoot.className = css({
  display: "flex",
  flexDirection: "column",
  padding: "1rem",
  height: "100dvh",
  "*::selection": {
    backgroundColor: "black",
  },
  fontFamily: "Bellefair, 'Zen Old Mincho', Hannari, serif",
  "@media screen and (max-width: 640px)": {
    padding: ".5rem",
  },
});

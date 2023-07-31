import { useEffect, useState } from "preact/hooks";
import Header from "./components/header.tsx";
import MainContent from "./components/main_content.tsx";
import { addErrorListener, GameComponentProps, ws } from "./ws.ts";

export function App() {
  const [loaded, setLoaded] = useState(false);
  const [members, setMembers] = useState([]);
  const [online, setOnline] = useState([]);
  const [state, setState] = useState({ "waiting": { "config": undefined } });
  const [logined, setLogin] = useState(false);
  const [name, setName] = useState("");
  const [master, setMaster] = useState("");
  const props: GameComponentProps = {
    members,
    online,
    logined,
    state,
    name,
    master,
  };

  useEffect(() => {
    ws().addEventListener("message", (m) => {
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
          setName(res.success.authenticationSuccess.name);
          setMaster(res.success.authenticationSuccess.master);
          setLogin(true);
        }
      }
    });

    addErrorListener((e) => {
      if (e.session === "authenticationFailed") {
        localStorage.removeItem("token");
        setLogin(false);
      } else {
        console.error(e);
      }
    });

    ws().addEventListener("message", () => setLoaded(true), { once: true });
    setLoaded(true);
  }, []);
  return (
    loaded
      ? (
        <>
          <Header {...props} />
          <MainContent {...props} />
        </>
      )
      : <></>
  );
}

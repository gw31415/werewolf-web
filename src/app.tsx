import { ContainerNode, render } from "preact";
import Members from "./components/members.tsx";
import Profile from "./components/user/profile.tsx";

function App() {
  return (
    <>
      <div style={{ display: "flex", justifyContent: "space-around" }}>
        <Profile
          style={{ marginTop: ".5rem" }}
          name="ワトソン"
          job="citizen"
        />
        <Members
          online={["元太", "ワトソン"]}
          members={[
            "ボブ",
            "アリス",
            "チャーリー",
            "元太",
            "ワトソン",
            "メリー",
          ]}
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
      </div>
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

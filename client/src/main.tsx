import { ContainerNode, render as preactRender } from "preact";
import { css } from "https://esm.sh/@emotion/css@11.11.0";
import { App } from "./_js/app.tsx";

export function render(
  elem: HTMLDivElement,
  callback?: () => void | undefined,
) {
  elem.innerText = "";

  preactRender(
    <App />,
    elem as ContainerNode,
  );

  elem.className = css({
    display: "flex",
    flexDirection: "column",
    padding: "1rem",
    height: ["100vh", "100dvh"],
    "*::selection": {
      backgroundColor: "black",
    },
    "*,*::before,*::after": {
      boxSizing: "border-box",
    },
    fontFamily: "Bellefair, 'Zen Old Mincho', Hannari, serif",
    "@media screen and (max-width: 640px)": {
      padding: ".5rem",
    },
  });

  if (callback) callback();
}

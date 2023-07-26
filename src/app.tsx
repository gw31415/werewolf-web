import { useState } from "preact/hooks";
import { ContainerNode, render } from "preact";

function Counter() {
  const [value, setValue] = useState(0);
  const increment = () => {
    setValue(value + 1);
  };

  return (
    <div>
      <p>count: {value}</p>
      <button onClick={increment}>Increment</button>
    </div>
  );
}

render(
  <div>
    <Counter />
  </div>,
  document.getElementById("app") as ContainerNode,
);

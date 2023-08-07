// deno-lint-ignore-file no-explicit-any

const WS_URL = {
  /// 開発用
  dev: `${
    location.protocol.startsWith("https") ? "wss" : "ws"
  }://${location.hostname}:3232/ws`,
  /// 本番用
  release: `${
    location.protocol.startsWith("https") ? "wss" : "ws"
  }://${location.host}/ws`,
};

let _ws: WebSocket | null = null;

/** サーバから受信するデータのProps */
export interface GameComponentProps {
  logined: boolean;
  members: string[];
  online: string[];
  name: string;
  master: string;
  state: any;
}

/** 初期化済みWebSocketを返す */
export function ws(): WebSocket {
  if (!_ws) {
    _ws = new WebSocket(WS_URL.dev);
    _ws.addEventListener("open", () => {
      const token = localStorage.getItem("token");
      if (token) {
        _ws!.send(JSON.stringify({
          connect: { token: token },
        }));
      }
    });
  }
  return _ws;
}

/** 構造体を送信する */
export function sender(req: any) {
  return ws().send(JSON.stringify(req));
}

/** エラーを受けとった時の処理を追加する */
export function addErrorListener(listener: (err: any) => void) {
  ws().addEventListener("message", (m) => {
    const res = JSON.parse(m.data);
    if (res.error) listener(res.error);
  });
}

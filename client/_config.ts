import lume from "lume/mod.ts";
import esbuild from "lume/plugins/esbuild.ts";
import minify_html from "lume/plugins/minify_html.ts";
import sass from "lume/plugins/sass.ts";
import imagick from "lume/plugins/imagick.ts";

const site = lume({
  src: "./src",
  dest: "../dist/site",
});

const define = Deno.env.get("TARGET") !== "RELEASE"
  ? {
    // 開発用
    "WS_URL.dev": "WS_URL.dev",
  }
  : {
    // 本番用
    "WS_URL.dev": "WS_URL.release",
  };

site.use(esbuild({
  extensions: [".ts", ".js", ".tsx", ".jsx"],
  options: {
    define,
  },
}));
site.use(minify_html());
site.use(sass());
site.use(imagick());

export default site;

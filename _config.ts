import lume from "lume/mod.ts";
import esbuild from "lume/plugins/esbuild.ts";
import minify_html from "lume/plugins/minify_html.ts";
import sass from "lume/plugins/sass.ts";
import imagick from "lume/plugins/imagick.ts";

const site = lume({
  src: "./src",
});

site.use(esbuild({
  extensions: [".ts", ".js", ".tsx", ".jsx"],
  options: {
    bundle: true,
    format: "esm",
    minify: true,
    keepNames: false,
    platform: "browser",
    target: "esnext",
    treeShaking: true,
  },
}));
site.use(minify_html());
site.use(sass());
site.use(imagick());


export default site;

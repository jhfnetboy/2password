import {
  Resource,
  invoke
} from "./chunk-L5MUIIQ4.js";
import "./chunk-G3PMV62Z.js";

// node_modules/.pnpm/@tauri-apps+api@2.8.0/node_modules/@tauri-apps/api/image.js
var Image = class _Image extends Resource {
  /**
   * Creates an Image from a resource ID. For internal use only.
   *
   * @ignore
   */
  constructor(rid) {
    super(rid);
  }
  /** Creates a new Image using RGBA data, in row-major order from top to bottom, and with specified width and height. */
  static async new(rgba, width, height) {
    return invoke("plugin:image|new", {
      rgba: transformImage(rgba),
      width,
      height
    }).then((rid) => new _Image(rid));
  }
  /**
   * Creates a new image using the provided bytes by inferring the file format.
   * If the format is known, prefer [@link Image.fromPngBytes] or [@link Image.fromIcoBytes].
   *
   * Only `ico` and `png` are supported (based on activated feature flag).
   *
   * Note that you need the `image-ico` or `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   */
  static async fromBytes(bytes) {
    return invoke("plugin:image|from_bytes", {
      bytes: transformImage(bytes)
    }).then((rid) => new _Image(rid));
  }
  /**
   * Creates a new image using the provided path.
   *
   * Only `ico` and `png` are supported (based on activated feature flag).
   *
   * Note that you need the `image-ico` or `image-png` Cargo features to use this API.
   * To enable it, change your Cargo.toml file:
   * ```toml
   * [dependencies]
   * tauri = { version = "...", features = ["...", "image-png"] }
   * ```
   */
  static async fromPath(path) {
    return invoke("plugin:image|from_path", { path }).then((rid) => new _Image(rid));
  }
  /** Returns the RGBA data for this image, in row-major order from top to bottom.  */
  async rgba() {
    return invoke("plugin:image|rgba", {
      rid: this.rid
    }).then((buffer) => new Uint8Array(buffer));
  }
  /** Returns the size of this image.  */
  async size() {
    return invoke("plugin:image|size", { rid: this.rid });
  }
};
function transformImage(image) {
  const ret = image == null ? null : typeof image === "string" ? image : image instanceof Image ? image.rid : image;
  return ret;
}

// node_modules/.pnpm/@tauri-apps+plugin-clipboard-manager@2.3.0/node_modules/@tauri-apps/plugin-clipboard-manager/dist-js/index.js
async function writeText(text, opts) {
  await invoke("plugin:clipboard-manager|write_text", {
    label: opts == null ? void 0 : opts.label,
    text
  });
}
async function readText() {
  return await invoke("plugin:clipboard-manager|read_text");
}
async function writeImage(image) {
  await invoke("plugin:clipboard-manager|write_image", {
    image: transformImage(image)
  });
}
async function readImage() {
  return await invoke("plugin:clipboard-manager|read_image").then((rid) => new Image(rid));
}
async function writeHtml(html, altText) {
  await invoke("plugin:clipboard-manager|write_html", {
    html,
    altText
  });
}
async function clear() {
  await invoke("plugin:clipboard-manager|clear");
}
export {
  clear,
  readImage,
  readText,
  writeHtml,
  writeImage,
  writeText
};
//# sourceMappingURL=@tauri-apps_plugin-clipboard-manager.js.map

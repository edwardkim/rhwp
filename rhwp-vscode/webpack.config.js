// @ts-check
const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");

/** @type {import('webpack').Configuration} Extension Host 번들 */
const extensionConfig = {
  target: "node",
  mode: "none",
  entry: "./src/extension.ts",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "extension.js",
    libraryTarget: "commonjs2",
  },
  externals: {
    vscode: "commonjs vscode",
  },
  resolve: {
    extensions: [".ts", ".js"],
    alias: {
      "@rhwp-wasm": path.resolve(__dirname, "..", "pkg"),
    },
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: [/node_modules/, /src\/webview/],
        use: {
          loader: "ts-loader",
          options: { configFile: "tsconfig.json" },
        },
      },
      {
        test: /\.wasm$/,
        type: "javascript/auto",
        loader: "null-loader",
      },
    ],
  },
  performance: { hints: false },
  devtool: "nosources-source-map",
};

/** @type {import('webpack').Configuration} Webview 번들 */
const webviewConfig = {
  target: "web",
  mode: "none",
  entry: "./src/webview/viewer.ts",
  output: {
    path: path.resolve(__dirname, "dist", "webview"),
    filename: "viewer.js",
    clean: true,
  },
  resolve: {
    extensions: [".ts", ".js"],
    extensionAlias: {
      ".js": [".js", ".ts"],
    },
    fallback: {
      fs: false,
      path: false,
    },
    alias: {
      "@rhwp-wasm": path.resolve(__dirname, "..", "pkg"),
      "@/view/canvaskit-wasm-url$": path.resolve(
        __dirname,
        "src",
        "webview",
        "canvaskit-wasm-url.ts"
      ),
      "canvaskit-wasm$": path.resolve(
        __dirname,
        "node_modules",
        "canvaskit-wasm",
        "bin",
        "canvaskit.js"
      ),
      "@noble/hashes": path.resolve(__dirname, "node_modules", "@noble", "hashes"),
      "@": path.resolve(__dirname, "..", "rhwp-studio", "src"),
    },
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: /node_modules/,
        use: {
          loader: "ts-loader",
          options: { configFile: "tsconfig.webview.json" },
        },
      },
      {
        test: /\.wasm$/,
        resourceQuery: /url/,
        type: "asset/resource",
        generator: {
          filename: "assets/[name].[contenthash][ext]",
        },
      },
      {
        test: /\.wasm$/,
        resourceQuery: { not: [/url/] },
        type: "javascript/auto",
        loader: "null-loader",
      },
    ],
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        {
          from: path.resolve(__dirname, "..", "pkg", "rhwp_bg.wasm"),
          to: path.resolve(__dirname, "dist", "media", "rhwp_bg.wasm"),
        },
        {
          from: path.resolve(__dirname, "node_modules", "canvaskit-wasm", "LICENSE"),
          to: path.resolve(__dirname, "dist", "licenses", "canvaskit-wasm-LICENSE.txt"),
        },
        {
          from: path.resolve(__dirname, "node_modules", "@noble", "hashes", "LICENSE"),
          to: path.resolve(__dirname, "dist", "licenses", "noble-hashes-LICENSE.txt"),
        },
        // 필수 오픈소스 폰트만 번들 (크기 최적화)
        ...[
          "NotoSerifKR-Regular.woff2", "NotoSerifKR-Bold.woff2",
          "NotoSansKR-Regular.woff2", "NotoSansKR-Bold.woff2", "NotoSansKR-ExtraLight.woff2",
          "Pretendard-Regular.woff2", "Pretendard-Bold.woff2",
          "D2Coding-Regular.woff2",
          "NanumGothic-Regular.woff2", "NanumMyeongjo-Regular.woff2",
          "GowunBatang-Regular.woff2", "GowunDodum-Regular.woff2",
        ].map(f => ({
          from: path.resolve(__dirname, "..", "assets", "fonts", f),
          to: path.resolve(__dirname, "dist", "media", "fonts", f),
        })),
      ],
    }),
  ],
  performance: { hints: false },
  devtool: "nosources-source-map",
};

module.exports = [extensionConfig, webviewConfig];

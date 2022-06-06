const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin(),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, ".")
        }),
        // Have this example work in Edge which doesn't ship `TextEncoder` or
        // `TextDecoder` at this time.
        new webpack.ProvidePlugin({
          TextDecoder: ['text-encoding', 'TextDecoder'],
          TextEncoder: ['text-encoding', 'TextEncoder']
        })
    ],
    experiments: {
      asyncWebAssembly: true,
    },
    module: {
        rules: [
          {
            test: /\.bin/,
            type: 'asset/inline',
            generator: {
              dataUrl: content => {
                return content;
              }
            }
          }
        ],
    },
    mode: 'production',
    performance: {
      maxEntrypointSize: 2000000,
      maxAssetSize: 2000000,
    }
};


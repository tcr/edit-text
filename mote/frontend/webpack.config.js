var webpack = require('webpack');

module.exports = {
    plugins: [
        new webpack.ProvidePlugin({
            $: 'jquery',
            jQuery: 'jquery',
            'window.jQuery': 'jquery',
        }),
    ],
    module: {
        loaders: [
            { test: /\.css$/, loader: "style-loader!css-loader" },
            {
                test   : /\.(ttf|eot|svg|woff2?)(\?[a-z0-9]+)?$/,
                loader : 'file-loader'
            },
            {
              test: /\.tsx?$/,
              loader: 'ts-loader',
              exclude: /node_modules/
            }
        ]
    },
};
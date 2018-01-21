var webpack = require('webpack');

module.exports = {
    devtool: 'source-map',
    plugins: [
        new webpack.ProvidePlugin({
            $: 'jquery',
            jQuery: 'jquery',
            'window.jQuery': 'jquery',
        }),
    ],
    resolve: { 
        extensions: ['.ts', '.tsx', '.js', '.jsx']
    },
    module: {
        rules: [{
            test: /\.scss$/,
            use: [{
                loader: "style-loader" // creates style nodes from JS strings 
            }, {
                loader: "css-loader" // translates CSS into CommonJS 
            }, {
                loader: "sass-loader" // compiles Sass to CSS 
            }]
        },
        { 
            test: /\.css$/, use: [{loader: "style-loader" }, { loader: "css-loader" }]
        },
        {
            test   : /\.(ttf|eot|svg|woff2?)(\?[a-z0-9]+)?$/,
            use : [{loader: 'file-loader' }],
        },
        {
            test: /\.tsx?$/,
            use: [{loader: 'ts-loader'}],
            exclude: /node_modules/
        }]
    },
};
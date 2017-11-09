'use strict';

// Require index.html so it gets copied to dist
require('../css/index.css');
require('./index.html');
var graphviz = require('viz.js');

var Elm = require('./Main.elm');
var mountNode = document.getElementById('main');

// .embed() can take an optional second argument. This would be an object
// describing the data we need to start a program, i.e. a userID or some token
var app = Elm.Main.embed(mountNode, {/* REPLACE WITH FLAGS */} );

var DOT_RAW = "```dot\\n([\\s\\S]*?\\n)```";
var DOT_RE = new RegExp(DOT_RAW, 'im')

// Do final rendering of artifacts using javascript libraries
app.ports.renderArtifacts.subscribe(function(textMap) {
    var out = new Array();
    console.log("JS: rendering svg")
    console.log(`JS: DOT_RAW=${DOT_RAW}`)

    var replace_dot = function(match, dot) {
        try {
            var svg = graphviz(dot);
            return `<svg>${svg}</svg>`;
        }
        catch (e) {
            return "```\nGRAPHVIZ RENDERING ERROR:\n" + e.message + "```";
        }
    }
    for (var [art_id, text] of textMap) {
        let rendered = text.replace(DOT_RE, replace_dot)
        out.push([art_id, rendered]);
    }

    app.ports.artifactsRendered.send(out)
});

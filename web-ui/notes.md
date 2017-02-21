
css files:
- node_modules/basscss/css/basscss-cp.css
- node_modules/basscss-addons/modules/btn/index.css
- node_modules/basscss-addons/modules/colors/index.css


[
  { "id":10
  , "name":"req-name"
  , "path":"path"
  , "text": "text"
  , "partof": ["req-partof-1"]
  , "parts": ["req-part-1"]
  , "loc": { "path": "path", "row": 10, "col": 10 }
  , "completed": 0.0
  , "tested": 0.0
  }
]

change:
    var _user$project$Main$initialModel = F2(
to:
var ARTIFACTS_JSON = '[ { "id":10 , "name":"req-name" , "path":"path" , "text": "text" , "partof": ["req-partof-1"] , "parts": ["req-part-1"] , "loc": { "path": "path", "row": 10, "col": 10 } , "completed": 0.0 , "tested": 0.0 } ]';
var _user$project$Main$initialModel = F2(


and change:
    artifacts: _user$project$Artifacts_Commands$artifactsFromStrUnsafe("[]"),

to:
	artifacts: _user$project$Artifacts_Commands$artifactsFromStrUnsafe(ARTIFACTS_JSON),


Need to (in order):
- replace all `\` characters with `\\` characters
- replace all `'` charactesr with `\'` characters

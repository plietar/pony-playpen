define("ace/mode/pony_highlight_rules",["require","exports","module","ace/lib/oop","ace/mode/text_highlight_rules"], function(require, exports, module) {
"use strict";

var oop = require("../lib/oop");
var TextHighlightRules = require("./text_highlight_rules").TextHighlightRules;

var PonyHighlightRules = function() {

    this.$rules = {
        start: [{
            include: "#block-comments"
        }, {
            include: "#line-comments"
        }, {
            include: "#typedeclarations"
        }, {
            include: "#methoddeclarations"
        }, {
            include: "#keywords"
        }, {
            include: "#constants"
        }, {
            include: "#identifiers"
        }, {
            include: "#strings"
        }],
        "#block-comments": [{
            token: "comment.block.pony",
            regex: /\/\*/,
            push: [{
                token: "comment.block.pony",
                regex: /\*\//,
                next: "pop"
            }, {
                include: "#block-comments"
            }, {
                defaultToken: "comment.block.pony"
            }]
        }],
        "#constants": [{
            token: "constant.other.pony",
            regex: /\bthis\b/
        }, {
            token: "constant.language.pony",
            regex: /\b(?:true|false)\b/
        }, {
            token: "constant.numeric.pony",
            regex: /\b(?:0b[0-1_]*|0x[0-9a-fA-F_]*|[0-9][0-9_]*(?:\.[0-9][0-9_]*)?(?:(?:e|E)(?:\+|-)?[0-9_]+)?)\b/
        }],
        "#methoddeclarations": [{
            token: [
                "keyword.declaration.pony",
                "text",
                "keyword.other.capability.pony",
                "text",
                "entity.name.function.pony"
            ],
            regex: /\b(new|be|fun)(\s+)((?:iso|trn|ref|val|box|tag)?)\b(\s*)([_a-z][_a-zA-Z0-9]*)/
        }],
        "#typedeclarations": [{
            token: [
                "keyword.declaration.pony",
                "text",
                "keyword.other.capability.pony",
                "text",
                "entity.name.type.pony"
            ],
            regex: /\b(type|interface|trait|primitive|struct|class|actor)(\s+)((?:iso|trn|ref|val|box|tag)?)(@?\s*)([_A-Z][_a-zA-Z0-9]*)/
        }],
        "#identifiers": [{
            token: ["support.function.pony", "text"],
            regex: /\b([_a-z][_a-zA-Z0-9]*)\b(\(|\[)/
        }, {
            token: ["text", "variable.parameter.pony", "text"],
            regex: /(\.\s*)([_a-z][_a-zA-Z0-9]*)\b([^\(\[])/
        }, {
            token: [
                "text",
                "support.function.pony",
                "text",
                "text"
            ],
            regex: /(@\s*)([_a-zA-z][_a-zA-Z0-9]*)(\s*)(\(|\[)/
        }, {
            token: "entity.name.class.pony",
            regex: /\b_*[A-Z][_a-zA-Z0-9]*\b/
        }, {
            token: "text",
            regex: /\b_*[a-z][_a-zA-Z0-9']*/
        }],
        "#keywords": [{
            token: "keyword.other.intrinsic.pony",
            regex: /\b(?:compile_intrinsic|compile_error)\b/
        }, {
            token: "keyword.other.import.pony",
            regex: /\buse\b/
        }, {
            token: "keyword.other.declaration.pony",
            regex: /\b(?:var|let|embed|delegate)\b/
        }, {
            token: "entity.other.attribute-name.pony",
            regex: /\b(?:iso|trn|ref|val|box|tag)\b/
        }, {
            token: "keyword.control.jump.pony",
            regex: /\b(?:break|continue|return|error)\b/
        }, {
            token: "keyword.control.pony",
            regex: /\b(?:if|ifdef|then|elseif|else|end|match|where|try|with|as|recover|consume|object|digestof)\b/
        }, {
            token: "keyword.control.loop.pony",
            regex: /\b(?:while|do|repeat|until|for|in)\b/
        }, {
            token: "text",
            regex: /\-|\+|\*|\/(?![\/*])|%|<<|>>/
        }, {
            token: "text",
            regex: /==|!=|<=|>=|<|>/
        }, {
            token: "text",
            regex: /\b(?:is|isnt|not|and|or|xor)\b/
        }, {
            token: "text",
            regex: /=/
        }, {
            token: "text",
            regex: /\?|=>/
        }, {
            token: "text",
            regex: /\||\&|\,|\^/
        }],
        "#line-comments": [{
            token: "comment.line.double-slash.pony",
            regex: /\/\//,
            push: [{
                token: "comment.line.double-slash.pony",
                regex: /$/,
                next: "pop"
            }, {
                defaultToken: "comment.line.double-slash.pony"
            }]
        }],
        "#strings": [{
            token: "punctuation.definition.character.begin.pony",
            regex: /'/,
            push: [{
                token: "punctuation.definition.character.end.pony",
                regex: /'/,
                next: "pop"
            }, {
                token: "constant.character.escape.pony",
                regex: /\\(?:[abfnrtv\\"0]|x[0-9A-Fa-f]{2})/
            }, {
                token: "invalid.illegal.pony",
                regex: /\\./
            }, {
                defaultToken: "constant.character.pony"
            }]
        }, {
            token: "punctuation.definition.string.begin.pony",
            regex: /"""/,
            push: [{
                token: "punctuation.definition.string.end.pony",
                regex: /"""(?!")/,
                next: "pop"
            }, {
                defaultToken: "variable.parameter.pony"
            }]
        }, {
            token: "punctuation.definition.string.begin.pony",
            regex: /"/,
            push: [{
                token: "punctuation.definition.string.end.pony",
                regex: /"/,
                next: "pop"
            }, {
                token: "constant.string.escape.pony",
                regex: /\\(?:[abfnrtv\\"0]|x[0-9A-Fa-f]{2}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{6})/
            }, {
                token: "invalid.illegal.pony",
                regex: /\\./
            }, {
                defaultToken: "string.quoted.double.pony"
            }]
        }]
    }
    
    this.normalizeRules();
};

PonyHighlightRules.metaData = {
    fileTypes: ["pony"],
    name: "Pony",
    scopeName: "source.pony"
}


oop.inherits(PonyHighlightRules, TextHighlightRules);

exports.PonyHighlightRules = PonyHighlightRules;
});

define("ace/mode/folding/cstyle",["require","exports","module","ace/lib/oop","ace/range","ace/mode/folding/fold_mode"], function(require, exports, module) {
"use strict";

var oop = require("../../lib/oop");
var Range = require("../../range").Range;
var BaseFoldMode = require("./fold_mode").FoldMode;

var FoldMode = exports.FoldMode = function(commentRegex) {
    if (commentRegex) {
        this.foldingStartMarker = new RegExp(
            this.foldingStartMarker.source.replace(/\|[^|]*?$/, "|" + commentRegex.start)
        );
        this.foldingStopMarker = new RegExp(
            this.foldingStopMarker.source.replace(/\|[^|]*?$/, "|" + commentRegex.end)
        );
    }
};
oop.inherits(FoldMode, BaseFoldMode);

(function() {
    
    this.foldingStartMarker = /(\{|\[)[^\}\]]*$|^\s*(\/\*)/;
    this.foldingStopMarker = /^[^\[\{]*(\}|\])|^[\s\*]*(\*\/)/;
    this.singleLineBlockCommentRe= /^\s*(\/\*).*\*\/\s*$/;
    this.tripleStarBlockCommentRe = /^\s*(\/\*\*\*).*\*\/\s*$/;
    this.startRegionRe = /^\s*(\/\*|\/\/)#?region\b/;
    this._getFoldWidgetBase = this.getFoldWidget;
    this.getFoldWidget = function(session, foldStyle, row) {
        var line = session.getLine(row);
    
        if (this.singleLineBlockCommentRe.test(line)) {
            if (!this.startRegionRe.test(line) && !this.tripleStarBlockCommentRe.test(line))
                return "";
        }
    
        var fw = this._getFoldWidgetBase(session, foldStyle, row);
    
        if (!fw && this.startRegionRe.test(line))
            return "start"; // lineCommentRegionStart
    
        return fw;
    };

    this.getFoldWidgetRange = function(session, foldStyle, row, forceMultiline) {
        var line = session.getLine(row);
        
        if (this.startRegionRe.test(line))
            return this.getCommentRegionBlock(session, line, row);
        
        var match = line.match(this.foldingStartMarker);
        if (match) {
            var i = match.index;

            if (match[1])
                return this.openingBracketBlock(session, match[1], row, i);
                
            var range = session.getCommentFoldRange(row, i + match[0].length, 1);
            
            if (range && !range.isMultiLine()) {
                if (forceMultiline) {
                    range = this.getSectionRange(session, row);
                } else if (foldStyle != "all")
                    range = null;
            }
            
            return range;
        }

        if (foldStyle === "markbegin")
            return;

        var match = line.match(this.foldingStopMarker);
        if (match) {
            var i = match.index + match[0].length;

            if (match[1])
                return this.closingBracketBlock(session, match[1], row, i);

            return session.getCommentFoldRange(row, i, -1);
        }
    };
    
    this.getSectionRange = function(session, row) {
        var line = session.getLine(row);
        var startIndent = line.search(/\S/);
        var startRow = row;
        var startColumn = line.length;
        row = row + 1;
        var endRow = row;
        var maxRow = session.getLength();
        while (++row < maxRow) {
            line = session.getLine(row);
            var indent = line.search(/\S/);
            if (indent === -1)
                continue;
            if  (startIndent > indent)
                break;
            var subRange = this.getFoldWidgetRange(session, "all", row);
            
            if (subRange) {
                if (subRange.start.row <= startRow) {
                    break;
                } else if (subRange.isMultiLine()) {
                    row = subRange.end.row;
                } else if (startIndent == indent) {
                    break;
                }
            }
            endRow = row;
        }
        
        return new Range(startRow, startColumn, endRow, session.getLine(endRow).length);
    };
    this.getCommentRegionBlock = function(session, line, row) {
        var startColumn = line.search(/\s*$/);
        var maxRow = session.getLength();
        var startRow = row;
        
        var re = /^\s*(?:\/\*|\/\/|--)#?(end)?region\b/;
        var depth = 1;
        while (++row < maxRow) {
            line = session.getLine(row);
            var m = re.exec(line);
            if (!m) continue;
            if (m[1]) depth--;
            else depth++;

            if (!depth) break;
        }

        var endRow = row;
        if (endRow > startRow) {
            return new Range(startRow, startColumn, endRow, line.length);
        }
    };

}).call(FoldMode.prototype);

});

define("ace/mode/pony",["require","exports","module","ace/lib/oop","ace/mode/text","ace/mode/pony_highlight_rules","ace/mode/folding/cstyle"], function(require, exports, module) {
"use strict";

var oop = require("../lib/oop");
var TextMode = require("./text").Mode;
var PonyHighlightRules = require("./pony_highlight_rules").PonyHighlightRules;
var FoldMode = require("./folding/cstyle").FoldMode;

var Mode = function() {
    this.HighlightRules = PonyHighlightRules;
    this.foldingRules = new FoldMode();
};
oop.inherits(Mode, TextMode);

(function() {
    this.$id = "ace/mode/pony"
}).call(Mode.prototype);

exports.Mode = Mode;
});

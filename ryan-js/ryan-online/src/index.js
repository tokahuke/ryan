import * as ryan from "ryan-lang";
import ace from "brace";
import "brace/mode/javascript";
import "brace/mode/json";
import "brace/mode/plain_text";
import "brace/theme/monokai";

window.ryan = ryan;

const editor = ace.edit("editor-area");
editor.getSession().setMode('ace/mode/javascript');
editor.setTheme('ace/theme/monokai');
editor.getSession().setUseWorker(false);
editor.getSession().setUseWrapMode(true);

const display = ace.edit("display-area");
display.getSession().setMode('ace/mode/json');
display.setTheme('ace/theme/monokai');
display.setReadOnly(true);
display.getSession().setUseWrapMode(true);

const runCycle = () => {
    try {
        const value = ryan.fromStr(editor.getValue());
        display.getSession().setMode('ace/mode/json');
        display.setValue(JSON.stringify(value, null, 4));
    } catch(e) {
        const error = e.toString();
        display.getSession().setMode('ace/mode/plain_text');
        display.setValue(error);
    }
    display.clearSelection();
};

runCycle();
editor.on("change", () => runCycle());

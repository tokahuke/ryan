import * as ryan from "ryan-lang";
import ace from "brace";
import "brace/mode/javascript";
import "brace/mode/json";
import "brace/mode/plain_text";
import "brace/theme/monokai";

window.ryan = ryan;

const sampleProgram = `// This is some sample Ryan code
// Edit at will!

let number_of_lights = 4;

{
    picard: number_of_lights,
    gul_madred: number_of_lights + 1,
}
`;
const initialCode = new URLSearchParams(location.search).get("c") || sampleProgram;
document.querySelector("#editor-area").textContent = initialCode;

const editor = ace.edit("editor-area");
editor.setShowPrintMargin(false);
editor.setTheme('ace/theme/monokai');
editor.getSession().setMode('ace/mode/javascript');
editor.getSession().setUseWorker(false);
editor.getSession().setUseWrapMode(true);

const display = ace.edit("display-area");
display.setShowPrintMargin(false);
display.setTheme('ace/theme/monokai');
display.setReadOnly(true);
display.getSession().setMode('ace/mode/json');
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


document.querySelector("#share-button").addEventListener("click", async (_e) => {
    const params = new URLSearchParams(location.search);
    params.set("c", editor.getValue());
    const generatedLink = `${location.href.split('?')[0]}?${params.toString()}`;

    navigator.clipboard.writeText(generatedLink);
    document.querySelector("#share-button .icon-desc").textContent = "COPIED!";

    await new Promise(resolve => setTimeout(resolve, 1_000));

    console.log("Hello!");
    document.querySelector("#share-button .icon-desc").textContent = "COPY LINK";
});

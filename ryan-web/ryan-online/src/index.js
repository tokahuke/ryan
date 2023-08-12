import * as ryan from "ryan-lang-web";
import ace, { edit } from "brace";
import "brace/mode/javascript";
import "brace/mode/json";
import "brace/mode/plain_text";
import "brace/theme/monokai";

// Set git hash in the code:
document
  .querySelectorAll(".git-hash")
  .forEach((e) => (e.textContent = __COMMIT_HASH__));

class Editor {
  constructor() {
    const sampleProgram = `// This is some sample Ryan code
// Edit at will!

let number_of_lights = 4;

{
    lights: {
        gul_madred: \`There are \${number_of_lights + 1} lights!\`,
        picard: \`There are \${number_of_lights} lights!\`,
    }
}
`;
    const initialCode =
      new URLSearchParams(location.search).get("c") ||
      localStorage.getItem("ryanOnline.saved") ||
      sampleProgram;
    document.querySelector("#editor-area").textContent = initialCode;

    const editor = ace.edit("editor-area");
    editor.setShowPrintMargin(false);
    editor.setTheme("ace/theme/monokai");
    editor.getSession().setMode("ace/mode/javascript");
    editor.getSession().setUseWorker(false);
    editor.getSession().setUseWrapMode(true);
    editor.on("change", () =>
      localStorage.setItem("ryanOnline.saved", editor.getValue())
    );

    const display = ace.edit("display-area");
    display.setShowPrintMargin(false);
    display.setTheme("ace/theme/monokai");
    display.setReadOnly(true);
    display.getSession().setMode("ace/mode/json");
    display.getSession().setUseWrapMode(true);

    this.editor = editor;
    this.display = display;
  }

  runCycle() {
    try {
      const value = ryan.fromStr(this.editor.getValue());
      this.display.getSession().setMode("ace/mode/json");
      this.display.setValue(JSON.stringify(value, null, 4));
    } catch (e) {
      const error = e.toString();
      this.display.getSession().setMode("ace/mode/plain_text");
      this.display.setValue(error);
    } finally {
      this.display.clearSelection();
    }
  }

  getValue() {
    return this.editor.getValue();
  }
}

class AutoRun {
  constructor() {
    const autoRun = localStorage.getItem("ryanOnline.autoRun");
    if (autoRun === null) {
      localStorage.setItem("ryanOnline.autoRun", true);
      autoRun = true;
    }

    this.autoRun = JSON.parse(autoRun);
    this.editorOnChangeEvent = (_e) => editor.runCycle();
    this.onKeydownCtrlEventEvent = (e) => {
      if (e.ctrlKey && e.key == "Enter") {
        editor.runCycle();
      }
    };

    this.refresh();

    document.querySelector("#auto-run").addEventListener("click", (_e) => {
      this.toggle();
    });
  }

  refreshButton() {
    if (this.autoRun) {
      document.querySelector("#auto-run-unset").classList.add("hidden");
      document.querySelector("#auto-run-set").classList.remove("hidden");
    } else {
      document.querySelector("#auto-run-unset").classList.remove("hidden");
      document.querySelector("#auto-run-set").classList.add("hidden");
    }
  }

  refreshEventHandler() {
    if (this.autoRun) {
      editor.editor.on("change", this.editorOnChangeEvent);
      document.removeEventListener("keydown", this.onKeydownCtrlEventEvent);
    } else {
      editor.editor.off("change", this.editorOnChangeEvent);
      document.addEventListener("keydown", this.onKeydownCtrlEventEvent);
    }
  }

  refresh() {
    this.refreshButton();
    this.refreshEventHandler();
  }

  toggle() {
    this.autoRun = !this.autoRun;
    localStorage.setItem("ryanOnline.autoRun", this.autoRun);
    this.refresh();
  }
}

// Set components up:
const editor = new Editor();
const autoRun = new AutoRun();

window.ryan = ryan;
window.editor = editor;
window.autoRun = autoRun;

editor.runCycle();

// Copy to clipboard:
document
  .querySelector("#share-button")
  .addEventListener("click", async (_e) => {
    const params = new URLSearchParams(location.search);
    params.set("c", editor.getValue());
    const generatedLink = `${location.href.split("?")[0]}?${params.toString()}`;

    navigator.clipboard.writeText(generatedLink);
    document.querySelector("#share-button .icon-desc").textContent = "COPIED!";

    await new Promise((resolve) => setTimeout(resolve, 1_000));

    document.querySelector("#share-button .icon-desc").textContent =
      "COPY LINK";
  });

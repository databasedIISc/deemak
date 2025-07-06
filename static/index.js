// index.js
const menuContainer = document.getElementById('menu_container');
const terminalContainer = document.getElementById('terminal_container');
const exitContainer = document.getElementById('exit_container');
const aboutContainer = document.getElementById('about_container');
const terminal = document.getElementById('terminal');
const authenticated = window.AUTHENTICATED || false;

let currentDir = "";
let commandHistory = [];
let historyIndex = -1;

function focusTerminal() {
  const input = document.getElementById("terminal_input");
  if (input) input.focus();
}

function startTerminal() {
  terminalContainer.style.display = "flex";
  menuContainer.style.display = "none";
  terminal.innerHTML = `        
    <div class="ascii_art">
          <pre>
 _____                            _
|  __ \\                          | |
| |  | | ___  ___ _ __ ___   __ _| | __
| |  | |/ _ \\/ _ \\ '_ \` _ \\ / _\` | |/ /
| |__| |  __/  __/ | | | | | (_| |   <
|_____/ \\___|\\___|_| |_| |_|\\__,_|_|\\_\\

          </pre>
    </div>
    <p class="startup_msg">
      Built by <strong>Databased Club, IISc</strong> · <a href="https://github.com/databasedIISc/deemak" target="_blank">GitHub</a><br>
      Type <code>help</code> to begin.
    </p>`;
  addNewInput();
}

function stopTerminal() {
  terminalContainer.style.display = "none";
  menuContainer.style.display = "flex";
  clearTerminal();
}

function exitShell() {
  exitContainer.style.display = "flex";
  terminalContainer.style.display = "none";
  menuContainer.style.display = "none";
}

function about() {
  aboutContainer.style.display = "flex";
  terminalContainer.style.display = "none";
  menuContainer.style.display = "none";
}

function backToMenu() {
  aboutContainer.style.display = "none";
  terminalContainer.style.display = "none";
  menuContainer.style.display = "flex";
  clearTerminal();
}

function clearTerminal() {
  terminal.innerHTML = `        
    <div class="terminal_line">
      <h4>Type commands and press Enter. Try <code>help</code> for more info.</h4>
    </div>`;
  addNewInput();
  return;
}

async function processCommand(input) {
  const command = input.value.trim();
  commandHistory.push(command);
  historyIndex = commandHistory.length;
  input.insertAdjacentHTML('afterend', `<div class="previous_input">${command}</div>`);
  document.getElementById("terminal_input").remove();

  const output = document.createElement('div');
  output.classList.add("response");

  if (command === "clear") {
    clearTerminal();
    return;
  } else if (command === "exit") {
    stopTerminal();
    return;
  }

  try {
    const response = await fetch(
      `${window.BACKEND_URL}/backend/run?command=${encodeURIComponent(command)}&current_dir=${encodeURIComponent(currentDir)}`
    );
    const result = await response.json();
    output.innerText = result.output;
    if (result.new_current_dir) {
      currentDir = result.new_current_dir;
    }
  } catch (error) {
    console.error("Error:", error);
    output.innerText = "Error: Could not reach server.";
  }

  terminal.appendChild(output);
  addNewInput();
}

function addNewInput() {
  const line = document.createElement('div');
  line.className = 'terminal_line';

  const prompt = document.createElement('div');
  prompt.textContent = '$';
  prompt.classList.add('dollor');

  const input = document.createElement('input');
  input.type = 'text';
  input.id = 'terminal_input';

  input.addEventListener('keydown', function (e) {
    if (e.key === 'Enter') {
      processCommand(input);
    } else if (e.key === 'ArrowUp') {
      if (historyIndex > 0) {
        historyIndex--;
        input.value = commandHistory[historyIndex];
      }
    } else if (e.key === 'ArrowDown') {
      if (historyIndex < commandHistory.length - 1) {
        historyIndex++;
        input.value = commandHistory[historyIndex];
      } else {
        input.value = "";
      }
    }
  });

  line.appendChild(prompt);
  line.appendChild(input);
  terminal.appendChild(line);

  input.focus();
  input.autocomplete = 'off';
}

window.onload = () => {
  const initialInput = document.querySelector('#terminal_input');
  if (initialInput) {
    initialInput.addEventListener('keydown', function (e) {
      if (e.key === 'Enter') {
        processCommand(e.target);
      }
    });
  }
};

window.startTerminal = startTerminal;
window.exitShell = exitShell;
window.about = about;
window.backToMenu = backToMenu;
window.focusTerminal = focusTerminal;
window.stopTerminal = stopTerminal;
window.clearTerminal = clearTerminal;   
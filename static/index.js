// index.js
const menuContainer      = document.getElementById('menu_container');
const terminalContainer  = document.getElementById('terminal_container');
const authContainer      = document.getElementById('auth_container');
const aboutContainer     = document.getElementById('about_container');
const loginContainer     = document.getElementById('login_container');
const registerContainer  = document.getElementById('register_container');
const terminal           = document.getElementById('terminal');
const loading            = document.getElementById('loading');
const loginMessage       = document.getElementById('login_message');
const registerMessage    = document.getElementById('register_message');

let authenticated           = false;
let registered              = false;
let exitConfirmationPending = false;

let currentDir    = "";
let commandHistory = [];
let historyIndex   = -1;

async function checkSessionOnLoad() {
  const token = localStorage.getItem('token')
  if (token) {
    await verifySession(token);
  } else {
    showAuthScreen();
  }
}

async function verifySession(existingToken) {
  loading.style.display = "flex";
  try {
    const tokenResponse = await fetch(`${window.BACKEND_URL}/backend/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: `token=${encodeURIComponent(existingToken)}&username=&password=`
    });

    const tokenResult = await tokenResponse.json();

    if (tokenResult.status) {
      authenticated = true;
      authContainer.style.display = "none";
      menuContainer.style.display = "flex";
    } else {
      localStorage.removeItem('token');
      showAuthScreen();
    }
  } catch (error) {
    console.error("Session verify error:", error);
    localStorage.removeItem('token');
    showAuthScreen("Server error. Please try again.");
  } finally {
    loading.style.display = "none";
  }
}


function showAuthScreen(errorMsg = "") {
  authenticated = false;
  registered    = false;
  authContainer.style.display     = "flex";
  loginContainer.style.display    = "flex";
  registerContainer.style.display = "none";
  if (errorMsg) loginMessage.textContent = errorMsg;
}

async function login() {
  const usernameInput = document.getElementById('login_username_input');
  const passwordInput = document.getElementById('login_password_input');

  const username = usernameInput.value.trim();
  const password = passwordInput.value.trim();

  if (!username || !password) {
    loginMessage.textContent = 'Please enter both username and password.';
    return;
  }

  loading.style.display = "flex";
  authContainer.style.display = "none";

  try {
    const response = await fetch(`${window.BACKEND_URL}/backend/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: `username=${encodeURIComponent(username)}&password=${encodeURIComponent(password)}`
    });
    const result = await response.json();

    if (result.status) {
      authenticated = true;
      localStorage.setItem('token', result.token);
      loginMessage.textContent = '';
      usernameInput.value = '';
      passwordInput.value = '';
      authContainer.style.display = "none";
      menuContainer.style.display = "flex";
    } else {
      loginMessage.textContent = result.message;
      showAuthScreen();
    }
  } catch (error) {
    console.error("Login error:", error);
    showAuthScreen("Server error. Please try again.");
  } finally {
    loading.style.display = "none";
  }
}

async function register() {
  const usernameInput = document.getElementById('register_username_input');
  const passwordInput = document.getElementById('register_password_input');

  const username = usernameInput.value.trim();
  const password = passwordInput.value.trim();

  if (!username || !password) {
    registerMessage.textContent = 'Please enter both username and password.';
    return;
  }

  loading.style.display = "flex";
  authContainer.style.display = "none";

  try {
    const response = await fetch(`${window.BACKEND_URL}/backend/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: `username=${encodeURIComponent(username)}&password=${encodeURIComponent(password)}`
    });
    const result = await response.json();

    if (result.status) {
      localStorage.setItem('token', result.token);
      authenticated = true;
      registerMessage.textContent = '';
      usernameInput.value = '';
      passwordInput.value = '';
      authContainer.style.display = "none";
      menuContainer.style.display = "flex";
    } else {
      registerMessage.textContent = result.message;
      authContainer.style.display = "flex";
      registerContainer.style.display = "flex";
    }
  } catch (error) {
    console.error("Registration error:", error);
    authContainer.style.display = "flex";
    registerContainer.style.display = "flex";
    registerMessage.textContent = "Server error. Please try again.";
  } finally {
    loading.style.display = "none";
  }
}
function logout() {
  localStorage.removeItem('token');
  authenticated = false
  showAuthScreen();
  terminalContainer.style.display  = "none";
  menuContainer.style.display      = "none";
  aboutContainer.style.display     = "none";
}

window.onload = async () => {
  await checkSessionOnLoad();
  const initialInput = document.querySelector('#terminal_input');
  if (initialInput) {
    initialInput.addEventListener('keydown', function (e) {
      if (e.key === 'Enter') {
        processCommand(e.target);
      }
    });
  }
};

function focusTerminal() {
  const input = document.getElementById("terminal_input");
  if (input) input.focus();
}

function startTerminal() {
  terminalContainer.style.display = "flex";
  menuContainer.style.display     = "none";
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
    Developed by <strong>Databased Club, Indian Institute of Science, Bangalore.</strong><br>
    Official Github Repo:<a href="https://github.com/databasedIISc/deemak" target="_blank">GitHub</a><br>
    <h4>Type commands and press Enter. Try <code>help</code> for more info.</h4>
    </p>`;
  addNewInput();
}

function showRegister() {
  registered = true;
  registerContainer.style.display = "flex";
  loginContainer.style.display    = "none";
}

function showLogin() {
  registered = false;
  registerContainer.style.display = "none";
  loginContainer.style.display    = "flex";
}

function stopTerminal() {
  terminalContainer.style.display = "none";
  menuContainer.style.display     = "flex";
  clearTerminal();
}

function about() {
  aboutContainer.style.display = "flex";
  terminalContainer.style.display = "none";
  menuContainer.style.display = "none";
}

function backToMenu() {
  aboutContainer.style.display    = "none";
  terminalContainer.style.display = "none";
  menuContainer.style.display     = "flex";
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
  } else if (command.startsWith("clear ")) {
    output.innerText = "Command not found. Try `help`.";
    terminal.appendChild(output);
    addNewInput();
    return;
  }

  if (command === "exit") {
    if (exitConfirmationPending) {
      output.innerText = "Exiting terminal...";
      terminal.appendChild(output);
      stopTerminal();
      exitConfirmationPending = false;
    } else {
      output.innerText = "Are you sure you want to exit? Make sure you have saved your progress first? Type `exit` again to confirm.";
      terminal.appendChild(output);
      exitConfirmationPending = true;
      addNewInput();
    }
    return;
  } else if (command.startsWith("exit ")) {
    output.innerText = "Command not found. Try `help`.";
    terminal.appendChild(output);
    addNewInput();
    return;
  } else {
    exitConfirmationPending = false;
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
  input.id   = 'terminal_input';

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

window.startTerminal  = startTerminal;
window.about          = about;
window.backToMenu     = backToMenu;
window.focusTerminal  = focusTerminal;
window.stopTerminal   = stopTerminal;
window.clearTerminal  = clearTerminal;
window.showRegister   = showRegister;
window.showLogin      = showLogin;
window.login          = login;
window.register       = register;
window.logout         = logout;
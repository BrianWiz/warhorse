import { createSignal, onMount, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import FriendList from "./components/FriendList.tsx";
import "./App.css";
import { listen } from "@tauri-apps/api/event";
import { Button } from "./components/Button.tsx";
import Chat from "./components/Chat.tsx";

function App() {
  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [receivedHello, setReceivedHello] = createSignal(false);
  const [isLoggedIn, setIsLoggedIn] = createSignal(false);

  async function login() {
    await invoke("login", {
      username: username(),
      password: password(),
    });

    // safety: clear password field
    setPassword("");
  }

  onMount(async () => {
    // listen for state events
    const unlisten_received_hello = await listen('received_hello', (_) => {
      setReceivedHello(true);
    });

    const unlisten_received_logged_in = await listen('received_logged_in', (_) => {
      setIsLoggedIn(true);
    });

    // get initial states
    const initialHello = await invoke<boolean>('received_hello');
    setReceivedHello(initialHello);
    
    const initialLoggedIn = await invoke<boolean>('received_logged_in');
    setIsLoggedIn(initialLoggedIn);

    return () => {
        unlisten_received_hello();
        unlisten_received_logged_in();
    };
  });

  return (
    <main>
      <header>
        <h1>Warhorse</h1>
      </header>
      <Show when={receivedHello() && !isLoggedIn()}>
        <div class="row center-things">
          <h2>Login</h2>
          <form 
            onSubmit={(e) => {
              e.preventDefault();
              login();
            }}
          >
            <label for="login-input-username">Username or Email</label>
            <input
              id="login-input-username"
              onChange={(e) => setUsername(e.currentTarget.value)}
              placeholder="Enter username or email"
            />
            <label for="login-input-password">Password</label>
            <input
              type="password"
              onChange={(e) => setPassword(e.currentTarget.value)}
              placeholder="Enter password"
            />
            <Button
              text="Login"
              onClick={login}
            />
          </form>
        </div>
      </Show>
      <Show when={isLoggedIn()}>
        <div class="container">
          <div class="sidebar">
            <h3>Friends</h3>
            <FriendList />
          </div>
          <div class="content">
            <h3>Chat</h3>
            <Chat />
          </div>
        </div>
      </Show>
    </main>
  );
}

export default App;
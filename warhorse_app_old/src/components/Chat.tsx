// Import from solid-js
import { createSignal, onMount, For } from "solid-js";
// Import your local ChatMessage component
import ChatMessage from "./ChatMessage";
// Import Tauri APIs
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export default function Chat() {
  const [messages, setMessages] = createSignal([
    { author: "Alice", message: "Hello!", timestamp: new Date() },
    { author: "Bob", message: "Hi!", timestamp: new Date() },
    { author: "Alice", message: "How are you?", timestamp: new Date() },
    { author: "Bob", message: "I'm good, how are you?", timestamp: new Date() },
    { author: "Alice", message: "I'm good too!", timestamp: new Date() },
    { author: "Bob", message: "That's great!", timestamp: new Date() },
    { author: "Alice", message: "Yeah!", timestamp: new Date() },
    { author: "Bob", message: "...", timestamp: new Date() }
  ]);
  
  onMount(async () => {
    type ChatMessage = {
      display_name: string;
      message: string;
      time: number;
    };

    const unlisten_chat_message = await listen<ChatMessage>('chat_message', (event) => {
      const { display_name, message, time } = event.payload;
      setMessages([...messages(), {
        author: display_name,
        message: message,
        timestamp: new Date(time * 1000) // Convert Unix timestamp to Date
      }]);
    });

    return () => {
      unlisten_chat_message();
    };
  });

  async function sendMessage(message: string) {
    await invoke('send_chat_message', { message });
  }

  return (
    <div class="chat">
      <div class="messages-container">
        <For each={messages()}>
          {(msg) => (
            <ChatMessage 
              author={msg.author}
              message={msg.message}
              timestamp={msg.timestamp}
            />
          )}
        </For>
      </div>
      <form
        onSubmit={async (e) => {
          e.preventDefault();
          const input = e.currentTarget.querySelector('input') as HTMLInputElement;
          if (!input?.value.trim()) return;
          
          await sendMessage(input.value);
            input.value = '';
          }}
      >
        <input placeholder="Enter message" />
        <button type="submit">Send</button>
      </form>
    </div> 
  );
}
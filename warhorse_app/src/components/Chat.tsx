import ChatMessage from "./ChatMessage";

export default function Chat() {
  return (
   <div class="chat">
    <div class="messages-container">
      <ChatMessage 
        author="Alice"
        message="Hello!"
        timestamp={new Date()}
      />
      <ChatMessage 
        author="Bob"
        message="Hi!"
        timestamp={new Date()}
      />
      <ChatMessage
        author="Alice"
        message="How are you?"
        timestamp={new Date()}
      />
      <ChatMessage
        author="Bob"
        message="I'm good, how are you?"
        timestamp={new Date()}
      />
      <ChatMessage
        author="Alice"
        message="I'm good too!"
        timestamp={new Date()}
      />
      <ChatMessage
        author="Bob"
        message="That's great!"
        timestamp={new Date()}
      />
      <ChatMessage
        author="Alice"
        message="Yeah!"
        timestamp={new Date()}
      />
      <ChatMessage
        author="Bob"
        message="..."
        timestamp={new Date()}
      />
    </div>
    <form
      onSubmit={(e) => {
        e.preventDefault();
        // send message
      }}
    >
      <input placeholder="Enter message" />
      <button>Send</button>
    </form>
   </div> 
  );
}

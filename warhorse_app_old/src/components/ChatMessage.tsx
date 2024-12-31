
interface ChatMessageProps {
  author: string;
  message: string;
  timestamp: Date,
}

export default function ChatMessage(props: ChatMessageProps) {

  function formatTimestamp(timestamp: Date): string {
    return timestamp.toLocaleString();
  }

  return (
    <div class="chat-message">
      <div class="author">{props.author}</div>
      <div class="timestamp">{formatTimestamp(props.timestamp)}</div>
      <div class="message">{props.message}</div>
    </div>
  );
}

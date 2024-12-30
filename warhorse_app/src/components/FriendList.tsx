import { invoke } from "@tauri-apps/api/core";
import Friend from "./Friend";
import { listen } from "@tauri-apps/api/event";
import { createSignal, onMount, Show } from "solid-js";
import { Button } from "./Button";
import Modal from "./Modal";

export default function FriendList() {
  const [displayModal, setDisplayModal] = createSignal(false);
  const [friends, setFriends] = createSignal([]);

  onMount(async () => {
    const unlisten = await listen('friends_updated', (event) => {
      setFriends(event.payload as any);
    });

    const initialFriends = await invoke('get_friends');
    setFriends(initialFriends as any);

    return () => {
      unlisten();
    };
  });

  return (
    <div class="friend-list">
      <div class="friend-list-content">
        <Show
          when={friends().length > 0}
          fallback={<>No friends</>}
        >
          {friends().map((friend: { display_name: string; id: string; status: string }) => (
            <Friend
              id={friend.id}
              display_name={friend.display_name}
              status={friend.status}
            />
          ))}
        </Show>
      </div>
      <div class="add-friend-container">
        <Button 
          text="Add Friend"
          class="add-friend-button secondary"
          onClick={() => {
            setDisplayModal(true);
          }}
        />
      </div>
      <Modal
        title="Request to become friends"
        isOpen={displayModal()}
        onClose={() => {
          setDisplayModal(false);
        }}
      >
        <form>
          <label for="friend-id">Friend ID</label>
          <input 
            id="friend-id" 
            type="text"
            placeholder="Enter friend's ID"
          />
          <Button 
            text="Send Request"
            onClick={() => {
              console.log('Send friend request');
            }}
          />
        </form>
      </Modal>
    </div>
  );
}

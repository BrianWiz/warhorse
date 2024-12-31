import { JSX, Show, ParentProps } from "solid-js";
import { Button } from "./Button";

interface ModalProps {
  title: string;
  isOpen: boolean;
  onClose: () => void;
}

export default function Modal(props: ParentProps<ModalProps>): JSX.Element {
  return (
    <Show when={props.isOpen}>
      <div class="modal">
        <div class="modal-content">
          <h3>{props.title}</h3>
          {props.children}
        </div>
        <div class="modal-buttons">
          <Button
            text="Close"
            class="secondary modal-close"
            onClick={props.onClose} 
          />
        </div>
      </div>
    </Show>
  );
}

enum ButtonType {
  Primary = 'primary',
  Secondary = 'secondary',
}

interface ButtonProps {
  text: string;
  type?: ButtonType;
  class?: string;
  onClick: () => void;
}

export function Button(props: ButtonProps) {
  const buttonType = props.type || ButtonType.Primary;
  const buttonClass = props.class || '';
  const classesFinal = `${buttonType} ${buttonClass}`;
  return (
    <button 
      class={classesFinal}
      onClick={props.onClick}
    >
      {props.text}
    </button>
  );
}

type ButtonProps = {
  label: string;
  onClick: () => void;
};

export function Button({ label, onClick }: ButtonProps) {
  return <button onClick={onClick}>{label}</button>;
}

export function Badge({ text }: { text: string }) {
  return <span>{text}</span>;
}

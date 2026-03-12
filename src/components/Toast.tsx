import { useEffect } from "react";

interface Props {
  message: string;
  type: "success" | "error" | "info";
  action?: { label: string; onClick: () => void };
  onClose: () => void;
  duration?: number;
}

export default function Toast({ message, type, action, onClose, duration = 6000 }: Props) {
  useEffect(() => {
    const timer = setTimeout(onClose, duration);
    return () => clearTimeout(timer);
  }, [onClose, duration]);

  return (
    <div className={`toast toast-${type}`}>
      <span className="toast-msg">{message}</span>
      {action && (
        <button className="toast-action" onClick={action.onClick}>
          {action.label}
        </button>
      )}
      <button className="toast-close" onClick={onClose}>×</button>
    </div>
  );
}

import { inter } from "../fonts";
import { colors } from "../theme";

export const KeyCap: React.FC<{
  label: string;
  fontSize?: number;
  pressed?: number; // 0..1, how far down the key is pressed
}> = ({ label, fontSize = 44, pressed = 0 }) => {
  return (
    <div
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "0.32em 0.6em",
        borderRadius: "0.3em",
        background: "linear-gradient(180deg, #ffffff 0%, #eef3fa 100%)",
        border: "2px solid #d4e0ef",
        borderBottomWidth: 2 + (1 - pressed) * 5,
        boxShadow: `0 ${(1 - pressed) * 10 + 2}px ${
          (1 - pressed) * 22 + 4
        }px rgba(27, 63, 118, 0.18)`,
        translate: `0px ${pressed * 5}px`,
        fontFamily: inter,
        fontWeight: 700,
        fontSize,
        color: colors.ink,
        letterSpacing: "0.01em",
        whiteSpace: "nowrap",
      }}
    >
      {label}
    </div>
  );
};

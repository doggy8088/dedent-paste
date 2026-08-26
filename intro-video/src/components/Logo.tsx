import { inter } from "../fonts";
import { colors } from "../theme";

/**
 * Clipboard glyph: a board whose text lines are flush left —
 * the "dedented" clipboard.
 */
export const LogoMark: React.FC<{ size?: number }> = ({ size = 120 }) => {
  return (
    <div
      style={{
        width: size,
        height: size,
        borderRadius: size * 0.24,
        background: `linear-gradient(150deg, ${colors.accent} 0%, #1e5fd6 100%)`,
        boxShadow:
          "0 18px 40px rgba(31, 92, 205, 0.35), inset 0 2px 0 rgba(255,255,255,0.35)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <svg
        width={size * 0.58}
        height={size * 0.58}
        viewBox="0 0 48 48"
        fill="none"
      >
        <rect
          x="8"
          y="7"
          width="32"
          height="37"
          rx="6"
          fill="rgba(255,255,255,0.16)"
          stroke="white"
          strokeWidth="3.4"
        />
        <rect x="17" y="3" width="14" height="8" rx="3" fill="white" />
        <path
          d="M15 20h18M15 27h12M15 34h15"
          stroke="white"
          strokeWidth="3.4"
          strokeLinecap="round"
        />
      </svg>
    </div>
  );
};

export const Wordmark: React.FC<{ fontSize?: number }> = ({
  fontSize = 96,
}) => {
  return (
    <div
      style={{
        fontFamily: inter,
        fontSize,
        fontWeight: 800,
        letterSpacing: "-0.035em",
        color: colors.ink,
      }}
    >
      dedent
      <span style={{ color: colors.accent }}>-</span>
      paste
    </div>
  );
};

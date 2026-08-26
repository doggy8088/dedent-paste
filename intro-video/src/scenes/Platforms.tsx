import {
  AbsoluteFill,
  Easing,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { Background } from "../components/Background";
import { KeyCap } from "../components/KeyCap";
import { mono, sans } from "../fonts";
import { colors } from "../theme";

const Card: React.FC<{
  start: number;
  os: string;
  shortcut: string;
  via: string;
}> = ({ start, os, shortcut, via }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const pop = spring({ frame: frame - start, fps, config: { damping: 15 } });

  return (
    <div
      style={{
        width: 560,
        padding: "56px 40px 48px",
        borderRadius: 30,
        background: colors.card,
        boxShadow: colors.cardShadow,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 26,
        opacity: interpolate(frame, [start, start + 8], [0, 1], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        }),
        translate: `0px ${interpolate(pop, [0, 1], [90, 0])}px`,
        scale: String(interpolate(pop, [0, 1], [0.95, 1])),
        fontFamily: sans,
      }}
    >
      <div
        style={{
          fontSize: 46,
          fontWeight: 700,
          letterSpacing: "-0.02em",
          color: colors.ink,
        }}
      >
        {os}
      </div>
      <KeyCap label={shortcut} fontSize={54} />
      <div style={{ fontSize: 30, fontWeight: 500, color: colors.inkSoft }}>
        {via}
      </div>
    </div>
  );
};

export const Platforms: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const pillIn = spring({ frame: frame - 62, fps, config: { damping: 16 } });

  return (
    <AbsoluteFill>
      <Background tone="blue" />
      <AbsoluteFill
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 60,
        }}
      >
        <div
          style={{
            fontFamily: sans,
            fontSize: 74,
            fontWeight: 900,
            letterSpacing: "-0.025em",
            color: colors.ink,
            opacity: interpolate(frame, [4, 18], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
              easing: Easing.bezier(0.16, 1, 0.3, 1),
            }),
            translate: `0px ${interpolate(frame, [4, 20], [26, 0], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
              easing: Easing.bezier(0.16, 1, 0.3, 1),
            })}px`,
          }}
        >
          你貼上的地方，它都能用
        </div>

        <div style={{ display: "flex", gap: 50 }}>
          <Card
            start={22}
            os="macOS"
            shortcut="⌥ V"
            via="搭配 Karabiner-Elements"
          />
          <Card
            start={34}
            os="Windows"
            shortcut="⊞ + V"
            via="搭配 AutoHotkey"
          />
        </div>

        <div
          style={{
            fontFamily: mono,
            fontSize: 36,
            fontWeight: 500,
            color: "#dbeafe",
            background: "#1c2b41",
            borderRadius: 18,
            padding: "24px 46px",
            boxShadow: "0 20px 50px rgba(20, 40, 70, 0.35)",
            opacity: interpolate(frame, [62, 72], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            }),
            translate: `0px ${interpolate(pillIn, [0, 1], [60, 0])}px`,
          }}
        >
          <span style={{ color: "#7a9cc9" }}>$</span> npm install -g
          dedent-paste
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

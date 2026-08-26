import {
  AbsoluteFill,
  Easing,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { Background } from "../components/Background";
import { LogoMark, Wordmark } from "../components/Logo";
import { inter, sans } from "../fonts";
import { colors } from "../theme";

export const Outro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const pop = spring({ frame, fps, config: { damping: 16, mass: 0.9 } });
  const pillIn = spring({ frame: frame - 34, fps, config: { damping: 16 } });

  return (
    <AbsoluteFill>
      <Background tone="blue" />
      <AbsoluteFill
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 44,
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 36,
            scale: String(interpolate(pop, [0, 1], [0.6, 1])),
            opacity: interpolate(frame, [0, 10], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            }),
          }}
        >
          <LogoMark size={110} />
          <Wordmark fontSize={104} />
        </div>

        <div
          style={{
            fontFamily: sans,
            fontSize: 40,
            fontWeight: 500,
            color: colors.inkSoft,
            opacity: interpolate(frame, [20, 34], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
              easing: Easing.bezier(0.16, 1, 0.3, 1),
            }),
          }}
        >
          自由開源 · MIT 授權
        </div>

        <div
          style={{
            fontFamily: inter,
            fontSize: 40,
            fontWeight: 700,
            color: colors.accent,
            background: "rgba(255,255,255,0.9)",
            borderRadius: 999,
            padding: "22px 52px",
            boxShadow: "0 16px 44px rgba(27, 63, 118, 0.16)",
            opacity: interpolate(frame, [34, 44], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            }),
            translate: `0px ${interpolate(pillIn, [0, 1], [50, 0])}px`,
            scale: String(interpolate(pillIn, [0, 1], [0.95, 1])),
          }}
        >
          github.com/doggy8088/dedent-paste
        </div>

        <div
          style={{
            position: "absolute",
            bottom: 64,
            left: 0,
            right: 0,
            textAlign: "center",
            fontFamily: sans,
            fontSize: 28,
            fontWeight: 400,
            color: colors.inkSoft,
            opacity: interpolate(frame, [52, 66], [0, 0.85], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
              easing: Easing.bezier(0.16, 1, 0.3, 1),
            }),
          }}
        >
          Copyright © 2026 Will 保哥
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

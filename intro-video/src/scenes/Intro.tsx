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
import { sans } from "../fonts";
import { colors } from "../theme";

export const Intro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const pop = spring({ frame, fps, config: { damping: 16, mass: 0.9 } });

  return (
    <AbsoluteFill>
      <Background tone="blue" />
      <AbsoluteFill
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 42,
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 40,
            scale: String(interpolate(pop, [0, 1], [0.6, 1])),
            opacity: interpolate(frame, [0, 10], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            }),
          }}
        >
          <LogoMark size={128} />
          <Wordmark fontSize={118} />
        </div>
        <div
          style={{
            fontFamily: sans,
            fontSize: 44,
            fontWeight: 500,
            color: colors.inkSoft,
            letterSpacing: "-0.01em",
            opacity: interpolate(frame, [26, 42], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
              easing: Easing.bezier(0.16, 1, 0.3, 1),
            }),
            translate: `0px ${interpolate(frame, [26, 44], [22, 0], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
              easing: Easing.bezier(0.16, 1, 0.3, 1),
            })}px`,
          }}
        >
          每一次貼上，都乾淨俐落
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

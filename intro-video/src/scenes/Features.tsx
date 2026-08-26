import {
  AbsoluteFill,
  Easing,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { Background } from "../components/Background";
import { sans } from "../fonts";
import { colors } from "../theme";

const FEATURES = [
  {
    icon: "📋",
    title: "純文字貼上",
    detail: "自動去除各種花俏格式",
  },
  {
    icon: "🧹",
    title: "移除共同縮排",
    detail: "相對縮排結構完整保留",
  },
  {
    icon: "✂️",
    title: "清除行尾空白",
    detail: "每一行都乾淨收尾",
  },
  {
    icon: "⏎",
    title: "接合 Codex CLI 換行",
    detail: "視覺折行還原成真正的段落",
  },
];

export const Features: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  return (
    <AbsoluteFill>
      <Background tone="blue" />
      <AbsoluteFill
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 54,
          padding: "90px 0",
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
          一個快捷鍵，全部搞定
        </div>

        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 26,
          }}
        >
          {FEATURES.map((f, i) => {
            const start = 26 + i * 11;
            const pop = spring({
              frame: frame - start,
              fps,
              config: { damping: 16, mass: 0.9 },
            });
            return (
              <div
                key={f.title}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 34,
                  width: 1150,
                  padding: "28px 44px",
                  borderRadius: 24,
                  background: colors.card,
                  boxShadow:
                    "0 18px 50px rgba(27, 63, 118, 0.12), 0 2px 10px rgba(27, 63, 118, 0.06)",
                  opacity: interpolate(frame, [start, start + 8], [0, 1], {
                    extrapolateLeft: "clamp",
                    extrapolateRight: "clamp",
                  }),
                  translate: `0px ${interpolate(pop, [0, 1], [90, 0])}px`,
                  scale: String(interpolate(pop, [0, 1], [0.96, 1])),
                }}
              >
                <div
                  style={{
                    width: 84,
                    height: 84,
                    borderRadius: 22,
                    background: colors.accentSoft,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: 42,
                    flexShrink: 0,
                  }}
                >
                  {f.icon}
                </div>
                <div style={{ fontFamily: sans }}>
                  <div
                    style={{
                      fontSize: 42,
                      fontWeight: 700,
                      letterSpacing: "-0.02em",
                      color: colors.ink,
                    }}
                  >
                    {f.title}
                  </div>
                  <div
                    style={{
                      fontSize: 30,
                      fontWeight: 500,
                      color: colors.inkSoft,
                      marginTop: 4,
                    }}
                  >
                    {f.detail}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

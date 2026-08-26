import {
  AbsoluteFill,
  Easing,
  interpolate,
  spring,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { Background } from "../components/Background";
import { mono, sans } from "../fonts";
import { colors } from "../theme";

const MD_LINES = [
  { text: "# 第三季報告", color: colors.accent },
  { text: "整體營收成長 **18%**", color: colors.ink },
  { text: "- 企業版成長 24%", color: colors.ink },
  { text: "- 中小企業成長 11%", color: colors.ink },
  { text: "| 區域 | 成長率 |", color: "#7a8ca6" },
];

export const ImageToText: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const leftIn = spring({ frame: frame - 26, fps, config: { damping: 16 } });
  const rightIn = spring({ frame: frame - 40, fps, config: { damping: 16 } });
  const arrowIn = spring({ frame: frame - 52, fps, config: { damping: 13 } });

  return (
    <AbsoluteFill>
      <Background tone="soft" />

      <div
        style={{
          position: "absolute",
          top: 108,
          left: 0,
          right: 0,
          textAlign: "center",
          fontFamily: sans,
          fontSize: 62,
          fontWeight: 900,
          letterSpacing: "-0.02em",
          color: colors.ink,
          opacity: interpolate(frame, [4, 18], [0, 1], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
            easing: Easing.bezier(0.16, 1, 0.3, 1),
          }),
          translate: `0px ${interpolate(frame, [4, 20], [24, 0], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
            easing: Easing.bezier(0.16, 1, 0.3, 1),
          })}px`,
        }}
      >
        剪貼簿裡只有圖片？
        <span style={{ color: colors.accent }}>照樣變成文字</span>
      </div>

      <AbsoluteFill
        style={{
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "center",
          gap: 70,
          paddingTop: 60,
        }}
      >
        {/* Screenshot card */}
        <div
          style={{
            width: 560,
            borderRadius: 26,
            background: colors.card,
            boxShadow: colors.cardShadow,
            padding: 26,
            opacity: interpolate(frame, [26, 36], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            }),
            translate: `0px ${interpolate(leftIn, [0, 1], [70, 0])}px`,
            scale: String(interpolate(leftIn, [0, 1], [0.94, 1])),
          }}
        >
          <div
            style={{
              borderRadius: 18,
              overflow: "hidden",
              background: "linear-gradient(160deg, #dfe9f6 0%, #cddcf0 100%)",
              padding: "30px 34px",
              height: 380,
            }}
          >
            {/* Fake screenshot content */}
            <div
              style={{
                width: 190,
                height: 30,
                borderRadius: 8,
                background: "#8fabd0",
              }}
            />
            {[300, 420, 360, 260].map((w, i) => (
              <div
                key={i}
                style={{
                  width: w,
                  height: 18,
                  borderRadius: 6,
                  background: "#b6c9e2",
                  marginTop: 22,
                }}
              />
            ))}
            <div
              style={{
                display: "flex",
                gap: 16,
                marginTop: 30,
              }}
            >
              {["#9cc0ee", "#aecfa8", "#e8c98e"].map((c) => (
                <div
                  key={c}
                  style={{
                    width: 130,
                    height: 90,
                    borderRadius: 12,
                    background: c,
                  }}
                />
              ))}
            </div>
          </div>
          <div
            style={{
              marginTop: 20,
              textAlign: "center",
              fontFamily: sans,
              fontSize: 30,
              fontWeight: 500,
              color: colors.inkSoft,
            }}
          >
            🖼️ 剪貼簿中的螢幕截圖
          </div>
        </div>

        {/* Arrow + Gemini badge */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 20,
            scale: String(interpolate(arrowIn, [0, 1], [0.4, 1])),
            opacity: interpolate(frame, [52, 62], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            }),
          }}
        >
          <div
            style={{
              fontSize: 76,
              color: colors.accent,
              translate: `${interpolate(
                frame,
                [60, 90, 120],
                [-8, 8, -8],
                { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
              )}px 0px`,
            }}
          >
            →
          </div>
          <div
            style={{
              fontFamily: sans,
              fontSize: 28,
              fontWeight: 700,
              color: colors.accent,
              background: "rgba(255,255,255,0.85)",
              border: `2px solid ${colors.accentSoft}`,
              borderRadius: 999,
              padding: "12px 28px",
              boxShadow: "0 10px 30px rgba(27, 63, 118, 0.12)",
              whiteSpace: "nowrap",
            }}
          >
            ✦ Gemini
          </div>
        </div>

        {/* Markdown card */}
        <div
          style={{
            width: 640,
            borderRadius: 26,
            background: colors.card,
            boxShadow: colors.cardShadow,
            padding: "40px 48px",
            height: 500,
            boxSizing: "border-box",
            opacity: interpolate(frame, [40, 50], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            }),
            translate: `0px ${interpolate(rightIn, [0, 1], [70, 0])}px`,
            scale: String(interpolate(rightIn, [0, 1], [0.94, 1])),
          }}
        >
          {MD_LINES.map((line, i) => {
            const start = 74 + i * 16;
            const chars = Math.round(
              interpolate(frame, [start, start + 14], [0, line.text.length], {
                extrapolateLeft: "clamp",
                extrapolateRight: "clamp",
              }),
            );
            return (
              <div
                key={i}
                style={{
                  fontFamily: mono,
                  fontSize: i === 0 ? 42 : 32,
                  fontWeight: i === 0 ? 700 : 500,
                  lineHeight: "76px",
                  color: line.color,
                  whiteSpace: "pre",
                }}
              >
                {line.text.slice(0, chars)}
                {chars > 0 && chars < line.text.length ? (
                  <span style={{ color: colors.accent }}>▍</span>
                ) : null}
              </div>
            );
          })}
          <div
            style={{
              marginTop: 26,
              fontFamily: sans,
              fontSize: 26,
              fontWeight: 500,
              color: colors.inkSoft,
              whiteSpace: "nowrap",
              opacity: interpolate(frame, [160, 172], [0, 1], {
                extrapolateLeft: "clamp",
                extrapolateRight: "clamp",
              }),
            }}
          >
            Markdown 結構完整保留 · 選用功能
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

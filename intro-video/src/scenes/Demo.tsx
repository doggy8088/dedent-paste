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

const CH = 21.6; // width of one monospace character at fontSize 36

type Line = {
  text: string;
  indent: number; // leading spaces as copied
  keep: number; // spaces that survive the dedent (relative indentation)
  trail: number; // junk trailing spaces
};

const LINES: Line[] = [
  { text: "def fetch_users(db):", indent: 8, keep: 0, trail: 0 },
  { text: "rows = db.query(User).all()", indent: 12, keep: 4, trail: 3 },
  { text: "return [r.name for r in rows]", indent: 12, keep: 4, trail: 5 },
];

const PRESS = 168; // frame where the shortcut is pressed

const Caption: React.FC<{
  from: number;
  to?: number;
  children: React.ReactNode;
}> = ({ from, to, children }) => {
  const frame = useCurrentFrame();
  const inOpacity = interpolate(frame, [from, from + 12], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.bezier(0.16, 1, 0.3, 1),
  });
  const outOpacity =
    to === undefined
      ? 1
      : interpolate(frame, [to, to + 10], [1, 0], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        });
  return (
    <div
      style={{
        position: "absolute",
        top: 96,
        left: 0,
        right: 0,
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        gap: 22,
        fontFamily: sans,
        fontSize: 54,
        fontWeight: 700,
        letterSpacing: "-0.02em",
        color: colors.ink,
        opacity: inOpacity * outOpacity,
        translate: `0px ${interpolate(frame, [from, from + 14], [20, 0], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: Easing.bezier(0.16, 1, 0.3, 1),
        })}px`,
      }}
    >
      {children}
    </div>
  );
};

export const Demo: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const cardIn = spring({ frame, fps, config: { damping: 17, mass: 0.9 } });
  const press = interpolate(
    frame,
    [PRESS - 4, PRESS, PRESS + 7],
    [0, 1, 0],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
  );
  const keycapIn = spring({
    frame: frame - 118,
    fps,
    config: { damping: 15, mass: 0.8 },
  });

  return (
    <AbsoluteFill>
      <Background tone="blue" />

      <Caption from={28} to={130}>
        <span>
          複製到的程式碼，帶著
          <span style={{ color: colors.accent }}>多餘的空白</span>
        </span>
      </Caption>
      <Caption from={140} to={205}>
        <span>接著，用</span>
        <KeyCap label="⌥ V" fontSize={46} pressed={press} />
        <span>貼上</span>
      </Caption>
      <Caption from={215}>
        <span
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: 20,
          }}
        >
          <span
            style={{
              width: 56,
              height: 56,
              borderRadius: 28,
              background: colors.good,
              color: "white",
              display: "inline-flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 34,
              boxShadow: "0 10px 26px rgba(22, 163, 74, 0.35)",
            }}
          >
            ✓
          </span>
          乾淨、對齊、直接可用
        </span>
      </Caption>

      {/* Terminal card */}
      <AbsoluteFill
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <div
          style={{
            width: 1360,
            borderRadius: 26,
            background: colors.card,
            boxShadow: colors.cardShadow,
            overflow: "hidden",
            scale: String(interpolate(cardIn, [0, 1], [0.9, 1])),
            opacity: interpolate(frame, [0, 12], [0, 1], {
              extrapolateLeft: "clamp",
              extrapolateRight: "clamp",
            }),
            translate: `0px ${interpolate(cardIn, [0, 1], [60, 26])}px`,
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              padding: "26px 34px",
              borderBottom: "1.5px solid #e8eef7",
            }}
          >
            {["#ff5f57", "#febc2e", "#28c840"].map((c) => (
              <div
                key={c}
                style={{
                  width: 20,
                  height: 20,
                  borderRadius: 10,
                  background: c,
                }}
              />
            ))}
            <div
              style={{
                flex: 1,
                textAlign: "center",
                fontFamily: sans,
                fontSize: 26,
                fontWeight: 500,
                color: "#9db0c7",
                marginRight: 76,
              }}
            >
              編輯器 — 貼上
            </div>
          </div>

          <div style={{ padding: "52px 64px 60px" }}>
            {LINES.map((line, i) => {
              const appear = 38 + i * 12;
              const collapseStart = PRESS + 4 + i * 4;
              const indentWidth = interpolate(
                frame,
                [collapseStart, collapseStart + 20],
                [line.indent * CH, line.keep * CH],
                {
                  extrapolateLeft: "clamp",
                  extrapolateRight: "clamp",
                  easing: Easing.bezier(0.3, 0.9, 0.25, 1),
                },
              );
              const junkOpacity = interpolate(
                frame,
                [collapseStart, collapseStart + 14],
                [1, 0],
                { extrapolateLeft: "clamp", extrapolateRight: "clamp" },
              );
              return (
                <div
                  key={i}
                  style={{
                    display: "flex",
                    alignItems: "baseline",
                    fontFamily: mono,
                    fontSize: 36,
                    lineHeight: "68px",
                    color: colors.ink,
                    whiteSpace: "pre",
                    opacity: interpolate(frame, [appear, appear + 10], [0, 1], {
                      extrapolateLeft: "clamp",
                      extrapolateRight: "clamp",
                    }),
                    translate: `0px ${interpolate(
                      frame,
                      [appear, appear + 12],
                      [18, 0],
                      {
                        extrapolateLeft: "clamp",
                        extrapolateRight: "clamp",
                        easing: Easing.bezier(0.16, 1, 0.3, 1),
                      },
                    )}px`,
                  }}
                >
                  <span
                    style={{
                      display: "inline-block",
                      width: indentWidth,
                      overflow: "hidden",
                      color: "rgba(47, 124, 246, 0.4)",
                    }}
                  >
                    <span style={{ opacity: junkOpacity }}>
                      {"·".repeat(line.indent)}
                    </span>
                  </span>
                  <span>{line.text}</span>
                  <span
                    style={{
                      color: "rgba(239, 68, 68, 0.5)",
                      opacity: junkOpacity,
                    }}
                  >
                    {"·".repeat(line.trail)}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      </AbsoluteFill>

      {/* Shortcut pulse ring below the card */}
      <div
        style={{
          position: "absolute",
          left: 0,
          right: 0,
          bottom: 108,
          display: "flex",
          justifyContent: "center",
          opacity: interpolate(frame, [118, 126], [0, 1], {
            extrapolateLeft: "clamp",
            extrapolateRight: "clamp",
          }),
        }}
      >
        <div style={{ position: "relative" }}>
          <div
            style={{
              position: "absolute",
              inset: -14,
              borderRadius: 32,
              border: "3px solid rgba(47, 124, 246, 0.55)",
              scale: String(
                interpolate(frame, [PRESS, PRESS + 26], [0.8, 1.55], {
                  extrapolateLeft: "clamp",
                  extrapolateRight: "clamp",
                  easing: Easing.bezier(0.16, 1, 0.3, 1),
                }),
              ),
              opacity: interpolate(frame, [PRESS, PRESS + 26], [0.9, 0], {
                extrapolateLeft: "clamp",
                extrapolateRight: "clamp",
              }),
            }}
          />
          <div
            style={{
              scale: String(interpolate(keycapIn, [0, 1], [0.5, 1])),
            }}
          >
            <KeyCap label="⌥ V" fontSize={52} pressed={press} />
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};

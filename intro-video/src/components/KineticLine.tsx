import { Easing, interpolate, useCurrentFrame } from "remotion";
import { sans } from "../fonts";
import { colors } from "../theme";

type Word = {
  text: string;
  highlight?: boolean;
};

/**
 * A centered line of text whose words pop in one by one.
 * Highlighted words get a soft blue marker that sweeps in
 * after the whole line has appeared.
 */
export const KineticLine: React.FC<{
  words: Word[];
  from: number;
  wordEvery?: number;
  fontSize?: number;
  out?: number;
}> = ({ words, from, wordEvery = 5, fontSize = 92, out }) => {
  const frame = useCurrentFrame();
  const lineDone = from + words.length * wordEvery + 8;

  const exitOpacity =
    out === undefined
      ? 1
      : interpolate(frame, [out, out + 12], [1, 0], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
        });
  const exitShift =
    out === undefined
      ? 0
      : interpolate(frame, [out, out + 12], [0, -26], {
          extrapolateLeft: "clamp",
          extrapolateRight: "clamp",
          easing: Easing.bezier(0.55, 0, 0.55, 0.2),
        });

  return (
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        justifyContent: "center",
        columnGap: "0.06em",
        rowGap: "0.2em",
        maxWidth: 1560,
        fontFamily: sans,
        fontSize,
        fontWeight: 700,
        letterSpacing: "-0.02em",
        color: colors.ink,
        opacity: exitOpacity,
        translate: `0px ${exitShift}px`,
      }}
    >
      {words.map((word, i) => {
        const start = from + i * wordEvery;
        return (
          <span
            key={i}
            style={{
              position: "relative",
              display: "inline-block",
              opacity: interpolate(frame, [start, start + 9], [0, 1], {
                extrapolateLeft: "clamp",
                extrapolateRight: "clamp",
                easing: Easing.bezier(0.16, 1, 0.3, 1),
              }),
              translate: `0px ${interpolate(
                frame,
                [start, start + 11],
                [26, 0],
                {
                  extrapolateLeft: "clamp",
                  extrapolateRight: "clamp",
                  easing: Easing.bezier(0.16, 1, 0.3, 1),
                },
              )}px`,
              filter: `blur(${interpolate(frame, [start, start + 9], [7, 0], {
                extrapolateLeft: "clamp",
                extrapolateRight: "clamp",
              })}px)`,
            }}
          >
            {word.highlight ? (
              <span
                style={{
                  position: "absolute",
                  left: "-0.06em",
                  right: "-0.06em",
                  top: "0.02em",
                  bottom: "-0.04em",
                  background: colors.accentSoft,
                  borderRadius: "0.12em",
                  transformOrigin: "left center",
                  scale: `${interpolate(
                    frame,
                    [lineDone, lineDone + 14],
                    [0, 1],
                    {
                      extrapolateLeft: "clamp",
                      extrapolateRight: "clamp",
                      easing: Easing.bezier(0.3, 0.8, 0.3, 1),
                    },
                  )} 1`,
                }}
              />
            ) : null}
            <span style={{ position: "relative" }}>{word.text}</span>
          </span>
        );
      })}
    </div>
  );
};

import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";

type Tone = "blue" | "soft";

const streak = (
  left: string,
  width: number,
  opacity: number,
  drift: number,
): React.CSSProperties => ({
  position: "absolute",
  top: "-80%",
  left,
  width,
  height: "260%",
  rotate: "24deg",
  translate: `${drift}px 0px`,
  background:
    "linear-gradient(90deg, rgba(255,255,255,0) 0%, rgba(255,255,255,0.9) 50%, rgba(255,255,255,0) 100%)",
  opacity,
  filter: "blur(48px)",
});

export const Background: React.FC<{ tone?: Tone }> = ({ tone = "blue" }) => {
  const frame = useCurrentFrame();
  const drift = interpolate(frame, [0, 600], [0, -70]);

  const base =
    tone === "blue"
      ? "linear-gradient(118deg, #a9cdf3 0%, #cfe3f9 26%, #eef5fd 52%, #d7e7fa 76%, #b6d3f4 100%)"
      : "linear-gradient(118deg, #e9f1fb 0%, #f7fafd 40%, #eff5fc 70%, #e2edf9 100%)";

  return (
    <AbsoluteFill style={{ background: base, overflow: "hidden" }}>
      <div style={streak("6%", 340, tone === "blue" ? 0.85 : 0.6, drift)} />
      <div style={streak("22%", 180, tone === "blue" ? 0.5 : 0.35, drift * 1.6)} />
      <div style={streak("64%", 420, tone === "blue" ? 0.65 : 0.45, drift * 0.8)} />
      <div style={streak("86%", 200, tone === "blue" ? 0.4 : 0.3, drift * 1.3)} />
      <AbsoluteFill
        style={{
          background:
            "radial-gradient(ellipse 60% 50% at 50% 46%, rgba(255,255,255,0.55) 0%, rgba(255,255,255,0) 70%)",
        }}
      />
    </AbsoluteFill>
  );
};

import { AbsoluteFill } from "remotion";
import { Background } from "../components/Background";
import { KineticLine } from "../components/KineticLine";

export const Problem: React.FC = () => {
  return (
    <AbsoluteFill>
      <Background tone="soft" />
      <AbsoluteFill
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: 120,
        }}
      >
        <KineticLine
          from={5}
          out={85}
          words={[
            { text: "你從文件、" },
            { text: "AI 對話、" },
            { text: "終端機" },
            { text: "複製程式碼" },
          ]}
        />
        <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center", padding: 120 }}>
          <KineticLine
            from={105}
            wordEvery={8}
            words={[
              { text: "一貼上，" },
              { text: "縮排", highlight: true },
              { text: "就" },
              { text: "亂成一團", highlight: true },
            ]}
          />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

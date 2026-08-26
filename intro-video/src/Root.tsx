import "./index.css";
import { Composition, Folder } from "remotion";
import { DedentPasteIntro } from "./DedentPasteIntro";
import { Demo } from "./scenes/Demo";
import { Features } from "./scenes/Features";
import { ImageToText } from "./scenes/ImageToText";
import { Intro } from "./scenes/Intro";
import { Outro } from "./scenes/Outro";
import { Platforms } from "./scenes/Platforms";
import { Problem } from "./scenes/Problem";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="DedentPasteIntro"
        component={DedentPasteIntro}
        durationInFrames={1370}
        fps={30}
        width={1920}
        height={1080}
      />
      <Folder name="Scenes">
        <Composition
          id="Intro"
          component={Intro}
          durationInFrames={120}
          fps={30}
          width={1920}
          height={1080}
        />
        <Composition
          id="Problem"
          component={Problem}
          durationInFrames={210}
          fps={30}
          width={1920}
          height={1080}
        />
        <Composition
          id="Demo"
          component={Demo}
          durationInFrames={360}
          fps={30}
          width={1920}
          height={1080}
        />
        <Composition
          id="Features"
          component={Features}
          durationInFrames={240}
          fps={30}
          width={1920}
          height={1080}
        />
        <Composition
          id="ImageToText"
          component={ImageToText}
          durationInFrames={200}
          fps={30}
          width={1920}
          height={1080}
        />
        <Composition
          id="Platforms"
          component={Platforms}
          durationInFrames={180}
          fps={30}
          width={1920}
          height={1080}
        />
        <Composition
          id="Outro"
          component={Outro}
          durationInFrames={150}
          fps={30}
          width={1920}
          height={1080}
        />
      </Folder>
    </>
  );
};

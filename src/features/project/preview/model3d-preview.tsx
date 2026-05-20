import { Suspense, useRef } from "react";
import { Canvas } from "@react-three/fiber";
import { OrbitControls, Grid, Environment, useGLTF } from "@react-three/drei";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ArrowCounterClockwise } from "@phosphor-icons/react";
import type { PreviewPayload } from "../../../lib/types";

type Model3dPayload = Extract<PreviewPayload, { type: "model3_d" }>;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function GltfModel({ src }: { src: string }) {
  const { scene } = useGLTF(src);
  return <primitive object={scene} />;
}

function ModelLoader({ path, format }: { path: string; format: string }) {
  const src = convertFileSrc(path);
  if (format === "glb" || format === "gltf") {
    return <GltfModel src={src} />;
  }
  // FBX/OBJ: return a placeholder box until loaders are wired
  return (
    <mesh>
      <boxGeometry args={[1, 1, 1]} />
      <meshStandardMaterial color="#3F3F46" />
    </mesh>
  );
}

export function Model3dPreview({ path, format }: Model3dPayload) {
  const fileName = path.split(/[\\/]/).pop() ?? path;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const orbitRef = useRef<any>(null);

  const resetCamera = () => {
    if (orbitRef.current) orbitRef.current.reset();
  };

  return (
    <div className="flex-1 relative bg-[#0e0e10]">
      <Canvas
        camera={{ position: [3, 2, 4], fov: 45 }}
        style={{ width: "100%", height: "100%" }}
      >
        <Suspense fallback={null}>
          <ambientLight intensity={0.5} />
          <directionalLight position={[5, 5, 5]} intensity={1} />
          <ModelLoader path={path} format={format} />
          <OrbitControls ref={orbitRef} makeDefault />
          <Grid
            infiniteGrid
            cellSize={0.5}
            cellThickness={0.6}
            sectionSize={3}
            sectionThickness={1.5}
            sectionColor="#27272A"
            cellColor="#1c1c1f"
            fadeDistance={30}
            fadeStrength={1}
          />
          <Environment preset="city" />
        </Suspense>
      </Canvas>

      {/* File label top-left */}
      <div className="absolute top-3 left-3 flex items-center gap-2 px-2 py-1 bg-elevated/80 backdrop-blur-md border border-border-default rounded-[var(--radius-md)] pointer-events-none">
        <span className="font-mono text-[10px] text-secondary">{fileName}</span>
        <span className="font-mono text-[10px] text-tertiary uppercase">{format}</span>
      </div>

      {/* Camera reset */}
      <button
        onClick={resetCamera}
        className="absolute bottom-4 right-4 flex items-center gap-1.5 px-2.5 h-7 bg-elevated/90 backdrop-blur-md border border-border-default rounded-[var(--radius-md)] font-mono text-[11px] text-secondary hover:text-primary transition-colors"
      >
        <ArrowCounterClockwise size={13} />
        Reset
      </button>
    </div>
  );
}

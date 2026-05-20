// kind_table.rs — file extension to AssetKind mapping.

use std::path::Path;

use crate::domain::tree::AssetKind;

/// Map a file path's extension to an `AssetKind`.
///
/// Matching is case-insensitive. Unknown extensions return `AssetKind::Binary`.
/// Directories must be classified by the caller (use `AssetKind::Folder`).
pub fn ext_to_kind(path: &Path) -> AssetKind {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Textures / images
        "png" | "jpg" | "jpeg" | "tga" | "bmp" | "psd" | "tiff" | "tif" | "exr" | "hdr"
        | "gif" | "webp" => AssetKind::Texture,

        // Audio
        "ogg" | "wav" | "mp3" | "aif" | "aiff" | "flac" | "xm" | "it" | "mod" | "s3m" => {
            AssetKind::Audio
        }

        // 3-D meshes
        "fbx" | "obj" | "gltf" | "glb" | "dae" | "3ds" | "blend" | "stl" => AssetKind::Mesh,

        // Scripts / assemblies
        "cs" | "dll" | "js" | "boo" => AssetKind::Script,

        // Text / data formats
        "txt" | "json" | "xml" | "yaml" | "yml" | "csv" | "md" | "ini" | "cfg" | "toml"
        | "html" | "htm" | "log" => AssetKind::Text,

        // Unity-specific
        "unity" | "scene" => AssetKind::Scene,
        "mat" | "material" => AssetKind::Material,
        "shader" | "cginc" | "hlsl" | "glsl" => AssetKind::Shader,
        "anim" | "controller" | "overridecontroller" => AssetKind::Animation,
        "prefab" => AssetKind::Prefab,

        // Everything else
        _ => AssetKind::Binary,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn kind(p: &str) -> AssetKind {
        ext_to_kind(Path::new(p))
    }

    #[test]
    fn texture_extensions() {
        assert_eq!(kind("sprite.png"), AssetKind::Texture);
        assert_eq!(kind("Texture.JPG"), AssetKind::Texture);
        assert_eq!(kind("normal.TGA"), AssetKind::Texture);
        assert_eq!(kind("icon.bmp"), AssetKind::Texture);
    }

    #[test]
    fn audio_extensions() {
        assert_eq!(kind("music.ogg"), AssetKind::Audio);
        assert_eq!(kind("effect.WAV"), AssetKind::Audio);
        assert_eq!(kind("theme.mp3"), AssetKind::Audio);
    }

    #[test]
    fn mesh_extensions() {
        assert_eq!(kind("model.fbx"), AssetKind::Mesh);
        assert_eq!(kind("mesh.OBJ"), AssetKind::Mesh);
        assert_eq!(kind("avatar.glb"), AssetKind::Mesh);
    }

    #[test]
    fn script_extensions() {
        assert_eq!(kind("Player.cs"), AssetKind::Script);
        assert_eq!(kind("Assembly-CSharp.dll"), AssetKind::Script);
    }

    #[test]
    fn text_extensions() {
        assert_eq!(kind("config.json"), AssetKind::Text);
        assert_eq!(kind("data.yaml"), AssetKind::Text);
        assert_eq!(kind("readme.txt"), AssetKind::Text);
        assert_eq!(kind("locales.xml"), AssetKind::Text);
    }

    #[test]
    fn unity_specific_extensions() {
        assert_eq!(kind("Main.unity"), AssetKind::Scene);
        assert_eq!(kind("Ground.mat"), AssetKind::Material);
        assert_eq!(kind("Run.anim"), AssetKind::Animation);
        assert_eq!(kind("Enemy.prefab"), AssetKind::Prefab);
        assert_eq!(kind("Lit.shader"), AssetKind::Shader);
    }

    #[test]
    fn binary_fallback() {
        assert_eq!(kind("assets.bundle"), AssetKind::Binary);
        assert_eq!(kind("data.bin"), AssetKind::Binary);
        assert_eq!(kind("noextension"), AssetKind::Binary);
    }
}

# Test Fixtures

## unity-synthetic-mono/

A **synthetic** Unity Mono build structure created programmatically for unit/integration tests.
No real Unity build is required. The fixture is committed to the repository.

### Structure

```
unity-synthetic-mono/
├── unity-synthetic-mono.exe          (fake PE — MZ header only, 128 bytes)
└── unity-synthetic-mono_Data/
    ├── globalgamemanagers             (64-byte synthetic header, version "2022.3.18f1")
    ├── sharedassets0.assets           (1 KiB random bytes)
    ├── Managed/
    │   ├── Assembly-CSharp.dll        (fake PE — MZ header, 128 bytes)
    │   └── UnityEngine.dll            (fake PE — MZ header, 128 bytes)
    └── Resources/
        └── unity_default_resources    (1 KiB random bytes)
```

### globalgamemanagers byte layout

Matches Unity serialized-file format >= 9 (Unity 5.x through 6000.x):

| Offset | Size | Value       | Description              |
|--------|------|-------------|--------------------------|
| 0      | 4    | 0x00000000  | metadata_size (BE u32)   |
| 4      | 4    | 0x00000000  | file_size (BE u32)       |
| 8      | 4    | 0x00000016  | format_version = 22 (BE) |
| 12     | 4    | 0x00000000  | data_offset (BE u32)     |
| 16     | 4    | 0x00000000  | endianness + reserved    |
| 20     | 12  | "2022.3.18f1\0" | null-terminated version string |

### Detection expectations

- `detect()` should return `confidence >= 0.95`, `backend = Mono`
- `version::parse_engine_version_from_ggm()` should return `"2022.3.18f1"`
- `tree_builder::walk()` should return >= 5 nodes

## IL2CPP fixture

TODO: IL2CPP fixture requires a real Unity IL2CPP build (~200 MB GameAssembly.dll).
Acquisition: build a Unity project with IL2CPP backend, or download from a public
game that ships IL2CPP (check license). Place under `tests/fixtures/unity-il2cpp/`.

Tests marked `#[ignore]` require this fixture:
- `test_detect_il2cpp_unity`
- `test_open_il2cpp_round_trip`

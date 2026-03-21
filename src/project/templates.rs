pub const DEFAULT_CONFIG: &str = r#"[app]
mode = "launch"
command = "npm"
args = ["run", "start"]

[paths]
features = ["features"]
steps = ["steps"]
artifacts = ".electrotest/artifacts"
"#;

pub const DEFAULT_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "esModuleInterop": true
  },
  "include": ["steps/**/*.ts"]
}
"#;

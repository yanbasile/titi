use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub font: FontConfig,
    pub colors: ColorScheme,
    pub window: WindowConfig,
    pub shell: ShellConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub background: [f32; 4],
    pub foreground: [f32; 4],
    pub black: [f32; 4],
    pub red: [f32; 4],
    pub green: [f32; 4],
    pub yellow: [f32; 4],
    pub blue: [f32; 4],
    pub magenta: [f32; 4],
    pub cyan: [f32; 4],
    pub white: [f32; 4],
    pub bright_black: [f32; 4],
    pub bright_red: [f32; 4],
    pub bright_green: [f32; 4],
    pub bright_yellow: [f32; 4],
    pub bright_blue: [f32; 4],
    pub bright_magenta: [f32; 4],
    pub bright_cyan: [f32; 4],
    pub bright_white: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub program: Option<String>,
    pub args: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font: FontConfig {
                family: "monospace".to_string(),
                size: 14.0,
            },
            colors: ColorScheme::default(),
            window: WindowConfig {
                width: 1280,
                height: 720,
                title: "Titi Terminal".to_string(),
            },
            shell: ShellConfig {
                program: None,
                args: vec![],
            },
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        // Solarized Dark color scheme
        Self {
            background: [0.0, 0.169, 0.212, 1.0],
            foreground: [0.514, 0.580, 0.588, 1.0],
            black: [0.0, 0.169, 0.212, 1.0],
            red: [0.863, 0.196, 0.184, 1.0],
            green: [0.522, 0.600, 0.0, 1.0],
            yellow: [0.710, 0.537, 0.0, 1.0],
            blue: [0.149, 0.545, 0.824, 1.0],
            magenta: [0.827, 0.212, 0.510, 1.0],
            cyan: [0.165, 0.631, 0.596, 1.0],
            white: [0.933, 0.910, 0.835, 1.0],
            bright_black: [0.0, 0.169, 0.212, 1.0],
            bright_red: [0.796, 0.294, 0.086, 1.0],
            bright_green: [0.522, 0.600, 0.0, 1.0],
            bright_yellow: [0.710, 0.537, 0.0, 1.0],
            bright_blue: [0.149, 0.545, 0.824, 1.0],
            bright_magenta: [0.424, 0.443, 0.769, 1.0],
            bright_cyan: [0.576, 0.631, 0.631, 1.0],
            bright_white: [0.992, 0.965, 0.890, 1.0],
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> anyhow::Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("titi").join("config.toml"))
    }
}

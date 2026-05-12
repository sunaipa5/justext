use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
}

impl WindowConfig {
    fn get_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("org", "sunaipa", "justext") {
            let config_dir = proj_dirs.config_dir();
            let _ = fs::create_dir_all(config_dir);
            return config_dir.join("window.sz");
        }
        PathBuf::from("window.sz")
    }

    pub fn load() -> Self {
        let data = match fs::read_to_string(Self::get_path()) {
            Ok(content) => content,
            Err(_) => {
                return Self {
                    width: 800.0,
                    height: 600.0,
                };
            }
        };

        let parts: Vec<&str> = data.split('x').collect();
        if parts.len() == 2 {
            let w = parts[0].parse::<f32>().unwrap_or(800.0);
            let h = parts[1].parse::<f32>().unwrap_or(600.0);
            return Self {
                width: w,
                height: h,
            };
        }

        Self {
            width: 800.0,
            height: 600.0,
        }
    }
    pub fn save(width: f32, height: f32) {
        let data = format!("{}x{}", width, height);
        let _ = fs::write(Self::get_path(), data);
    }
}

use anyhow::{bail, Result};
use serde_derive::Deserialize;
// use lazy_static::lazy_static;

// use super::index::PyPI;

// lazy_static! {
//     pub static ref PYPI_MIRRORS: [PyPI; 1] = [PyPI::new(
//         "清华源",
//         "https://pypi.tuna.tsinghua.edu.cn/simple"
//     ),];
// }

pub static OBLIGATED_PACKAGES: [&str; 2] = ["setuptools>=68.0.0", "wheel>=0.38.0"];

#[derive(Debug, Deserialize)]
pub struct Config {
    pip_version: String,
    pypi: Vec<PyPIMirror>,
    cpython: Vec<CPythonDistSource>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PyPIMirror {
    name: String,
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CPythonDistSource {
    python_version: String,
    version: String,
    url: String,
    checksum: String,
}

impl Config {
    pub fn load() -> Result<Config> {
        use crate::resources::EmbededRequirements;
        let embed_config = EmbededRequirements::get("config.toml").unwrap();
        let content = String::from_utf8_lossy(embed_config.data.as_ref());
        let config: Config = toml::from_str(content.as_ref())?;

        Ok(config)
    }

    pub fn pip_version(&self) -> &str {
        &self.pip_version
    }

    pub fn get_cpytion_source(&self) -> Result<&CPythonDistSource> {
        use super::utils::get_windows_major_versoin;
        let win_major = get_windows_major_versoin()?;
        let python_version = if win_major > 7 { "3.11" } else { "3.8" };

        for dist in &self.cpython {
            if dist.python_version == python_version {
                return Ok(dist);
            }
        }

        bail!("在安装配置文件没找到{}下载信息", python_version)
    }

    pub fn get_pypi_mirrors(&self) -> &[PyPIMirror] {
        &self.pypi
    }
}

impl PyPIMirror {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn package_url(&self, canonical_name: &str) -> String {
        if self.url.ends_with('/') {
            format!("{}{}/", self.url, canonical_name)
        } else {
            format!("{}/{}/", self.url, canonical_name)
        }
    }
}

impl CPythonDistSource {
    pub fn cpython_version(&self) -> &str {
        &self.version
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn checksum(&self) -> &str {
        &self.checksum
    }
}

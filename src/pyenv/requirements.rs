use anyhow::{bail, Result};
use pep508_rs::Requirement;
use std::{fs::File, path::PathBuf};

use super::installer::Installer;

pub async fn install_requirements(installer: &Installer) -> Result<()> {
    use super::index::PyPI;
    let pypi = PyPI::new("清华源", "https://pypi.tuna.tsinghua.edu.cn/simple");

    let cached_packages_dir = &installer.cached_packages_dir;
    if let Err(_err) = std::fs::create_dir_all(cached_packages_dir) {
        bail!(
            "创建下载文件临时目录{}失败: {}",
            cached_packages_dir.display(),
            _err
        )
    }

    let requirements_path = &get_requirements_path(installer).await?;

    let mut requirements = extract_requirements(requirements_path).await?;
    requirements.append(&mut obligated_packages()?);

    use super::index::project::download_requirement;
    for requirement in &requirements {
        download_requirement(installer, &pypi, requirement).await?;
    }

    offline_install_requirements(installer, requirements_path, cached_packages_dir)?;

    Ok(())
}

fn offline_install_requirements(
    installer: &Installer,
    requirements_path: &PathBuf,
    cached_packages_dir: &PathBuf,
) -> Result<()> {
    use super::venv::venv_python_cmd;

    let output = match venv_python_cmd(
        installer,
        &vec![
            "-m",
            "pip",
            "install",
            "--no-index",
            "--find-links",
            &cached_packages_dir.to_string_lossy(),
            "-r",
            &requirements_path.to_string_lossy(),
        ],
    ) {
        Ok(output) => output,
        Err(err) => {
            bail!("调用python执行pip出现错误: {}", err)
        }
    };

    let status = output.status;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STATUS:{}\n{}\nSTDERR:\n{}", status, stdout, stderr);

    Ok(())
}

async fn get_requirements_path(installer: &Installer) -> Result<PathBuf> {
    let requirements_filename = format!(
        "requirements-{}-{}.txt",
        installer.python_version,
        installer.platform_tag.as_ref().unwrap()
    );

    let requirements_file = installer.tgba_dir().join(requirements_filename);
    if !requirements_file.is_file() {
        bail!("unimplemented: {}", requirements_file.display())
    }

    Ok(requirements_file)
}

async fn extract_requirements(requirements_path: &PathBuf) -> Result<Vec<Requirement>> {
    let file = File::open(requirements_path).unwrap();

    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(file);

    let mut requirements = Vec::new();
    let mut errors = Vec::new();
    for (line_idx, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Ignore errors.

        use std::str::FromStr;
        match Requirement::from_str(line.as_str()) {
            Ok(requirement) => {
                requirements.push(requirement);
            }
            Err(err) => {
                errors.push((line_idx + 1, err));
            }
        };
    }

    if errors.len() > 0 {
        let mut lines = Vec::new();
        for (line_no, err) in errors {
            lines.push(format!("Line {}: {}", line_no, err));
        }
        bail!(
            "errors in parsing requirements file: \n{}",
            lines.join("\n")
        )
    }

    Ok(requirements)
}

use super::config::OBLIGATED_PACKAGES;

fn obligated_packages() -> Result<Vec<Requirement>> {
    let mut requirements = Vec::new();
    let mut errors = Vec::new();
    for (idx, requirement) in OBLIGATED_PACKAGES.iter().enumerate() {
        use std::str::FromStr;
        match Requirement::from_str(requirement) {
            Ok(requirement) => {
                requirements.push(requirement);
            }
            Err(err) => {
                errors.push((idx + 1, err));
            }
        };
    }

    if errors.len() > 0 {
        let mut lines = Vec::new();
        for (line_no, err) in errors {
            lines.push(format!("Line {}: {}", line_no, err));
        }
        bail!(
            "errors in parsing requirements file: \n{}",
            lines.join("\n")
        )
    }

    Ok(requirements)
}

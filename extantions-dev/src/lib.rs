use std::{env, fs};
use zed::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result, serde_json::json};

const BINARY_NAME: &str = "vscode-html-language-server";
const SERVER_PATH: &str =
    "node_modules/@zed-industries/vscode-langservers-extracted/bin/vscode-html-language-server";
const PACKAGE_NAME: &str = "@zed-industries/vscode-langservers-extracted";

struct HtmlExtension {
    cached_binary_path: Option<String>,
}

impl HtmlExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).is_ok_and(|metadata| metadata.is_file())
    }

    fn server_script_path(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        let server_exists = self.server_exists();
        if self.cached_binary_path.is_some() && server_exists {
            return Ok(SERVER_PATH.to_string());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

        if !server_exists
            || zed::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let installation = zed::npm_install_package(PACKAGE_NAME, &version);
            if let Err(error) = installation
                && !self.server_exists()
            {
                return Err(error);
            }
            if !self.server_exists() {
                return Err(format!(
                    "installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'"
                ));
            }
        }

        Ok(SERVER_PATH.to_string())
    }
}

impl zed::Extension for HtmlExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        if let Some(path) = worktree.which(BINARY_NAME) {
            return Ok(zed::Command {
                command: path,
                args: vec!["--stdio".to_string()],
                env: Default::default(),
            });
        }

        let server_path = self.server_script_path(language_server_id)?;
        self.cached_binary_path = Some(server_path.clone());
        let server_path = env::current_dir()
            .map_err(|error| error.to_string())?
            .join(server_path)
            .to_string_lossy()
            .to_string();

        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![server_path, "--stdio".to_string()],
            env: Default::default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let mut settings = LspSettings::for_worktree(server_id.as_ref(), worktree)?
            .settings
            .unwrap_or_else(|| json!({}));
        let settings_object = settings
            .as_object_mut()
            .ok_or_else(|| "HTML language server settings must be a JSON object".to_string())?;
        let html_settings = settings_object
            .entry("html")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| {
                "HTML language server 'html' settings must be a JSON object".to_string()
            })?;
        html_settings.insert("autoClosingTags".to_string(), json!(false));

        Ok(Some(settings))
    }
}

zed::register_extension!(HtmlExtension);

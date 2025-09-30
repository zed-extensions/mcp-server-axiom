use std::fs;

use schemars::JsonSchema;
use serde::Deserialize;
use zed_extension_api::settings::ContextServerSettings;
use zed_extension_api::{
    self as zed, latest_github_release, serde_json, Command, ContextServerConfiguration,
    ContextServerId, Project, Result,
};

#[derive(Debug, Deserialize, JsonSchema)]
struct McpServerAxiomSettings {
    config_path: String,
    org_id: Option<String>,
    api_url: Option<String>,
}

struct McpServerAxiomExtension {
    cached_binary_path: Option<String>,
}

impl McpServerAxiomExtension {
    fn context_server_binary_path(
        &mut self,
        _context_server_id: &ContextServerId,
    ) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let release = latest_github_release(
            "axiomhq/mcp-server-axiom",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let version_dir = format!("mcp-server-axiom-{}", release.version);
        let binary_path = format!(
            "{version_dir}/mcp-server-axiom{extension}",
            extension = match platform {
                zed::Os::Mac | zed::Os::Linux => "",
                zed::Os::Windows => ".exe",
            }
        );

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            let asset_name = format!(
                "mcp-server-axiom_{os}_{arch}.{extension}",
                os = match platform {
                    zed::Os::Mac => "Darwin",
                    zed::Os::Linux => "Linux",
                    zed::Os::Windows => "Windows",
                },
                arch = match arch {
                    zed::Architecture::Aarch64 => "arm64",
                    zed::Architecture::X8664 => "x86_64",
                    zed::Architecture::X86 => return Err("axiom-mcp not available for x86".into()),
                },
                extension = match platform {
                    zed::Os::Mac | zed::Os::Linux => "tar.gz",
                    zed::Os::Windows => "zip",
                }
            );

            let asset = release
                .assets
                .iter()
                .find(|asset| asset.name == asset_name)
                .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

            zed::download_file(
                &asset.download_url,
                &version_dir,
                match platform {
                    zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
                    zed::Os::Windows => zed::DownloadedFileType::Zip,
                },
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for McpServerAxiomExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn context_server_command(
        &mut self,
        context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let settings = ContextServerSettings::for_project("mcp-server-axiom", project)?;
        let Some(settings) = settings.settings else {
            return Err("missing `axiom-mcp-server` setting".into());
        };
        let settings: McpServerAxiomSettings =
            serde_json::from_value(settings).map_err(|e| e.to_string())?;

        let mut env = vec![(
            "AXIOM_URL".into(),
            settings
                .api_url
                .unwrap_or_else(|| "https://api.axiom.co".into()),
        )];

        if let Some(org_id) = settings.org_id {
            env.push(("AXIOM_ORG_ID".into(), org_id));
        }

        Ok(Command {
            command: self.context_server_binary_path(context_server_id)?,
            args: vec!["--config".into(), settings.config_path],
            env,
        })
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();
        let default_settings = include_str!("../configuration/default_settings.jsonc").to_string();
        let settings_schema = serde_json::to_string(&schemars::schema_for!(McpServerAxiomSettings))
            .map_err(|e| e.to_string())?;

        Ok(Some(ContextServerConfiguration {
            installation_instructions,
            default_settings,
            settings_schema,
        }))
    }
}

zed::register_extension!(McpServerAxiomExtension);

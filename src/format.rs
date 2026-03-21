use anyhow::{anyhow, Result};
use biome_configuration::ConfigurationPathHint;
use biome_fs::{BiomePath, OsFileSystem};
use biome_resolver::FsWithResolverProxy;
use biome_service::configuration::{load_configuration, LoadedConfiguration};
use biome_service::workspace;
use biome_service::workspace::{
    FeaturesBuilder, FileContent, FormatFileParams, OpenFileParams, OpenProjectParams,
    SupportsFeatureParams, UpdateSettingsParams,
};
use camino::{Utf8Path, Utf8PathBuf};
use std::path::PathBuf;
use std::sync::Arc;

pub enum FormatResult {
    Success { formatted_content: String },
    Ignored,
    Error { error: String },
}

pub fn format(
    given_file_name: String,
    file_content: String,
    current_dir: PathBuf,
) -> Result<FormatResult> {
    // Convert PathBuf to Utf8PathBuf
    let current_dir_utf8 = Utf8PathBuf::from_path_buf(current_dir.clone())
        .map_err(|_| anyhow!("Path contains invalid UTF-8"))?;

    let fs: Arc<dyn FsWithResolverProxy> = Arc::new(OsFileSystem::new(current_dir_utf8.clone()));
    let workspace = workspace::server(fs, None);

    let LoadedConfiguration {
        configuration: biome_configuration,
        extended_configurations,
        ..
    } = load_configuration(
        &*workspace.fs(),
        ConfigurationPathHint::FromWorkspace(current_dir_utf8.clone()),
    )?;

    // Open the project
    let open_project_result = workspace.open_project(OpenProjectParams {
        path: BiomePath::new(&current_dir_utf8),
        open_uninitialized: true,
    })?;

    let project_key = open_project_result.project_key;

    // Update settings for the project
    workspace.update_settings(UpdateSettingsParams {
        project_key,
        configuration: biome_configuration,
        workspace_directory: workspace.fs().working_directory().map(BiomePath::new),
        extended_configurations: extended_configurations
            .into_iter()
            .map(|(path, configuration)| (BiomePath::new(path), configuration))
            .collect(),
        module_graph_resolution_kind: Default::default(),
    })?;

    let file_path = BiomePath::new(Utf8Path::new(&given_file_name));

    // Check if the file supports formatting
    let file_features = workspace.file_features(SupportsFeatureParams {
        project_key,
        path: file_path.clone(),
        features: FeaturesBuilder::new().with_formatter().build(),
        inline_config: None,
        skip_ignore_check: false,
        not_requested_features: Default::default(),
    })?;

    if !file_features.features_supported.supports_format() {
        return Ok(FormatResult::Ignored);
    }

    // Open the file with content from client
    workspace.open_file(OpenFileParams {
        project_key,
        path: file_path.clone(),
        content: FileContent::from_client(file_content),
        document_file_source: None,
        persist_node_cache: false,
        inline_config: None,
    })?;

    // Format the file
    let res = workspace.format_file(FormatFileParams {
        project_key,
        path: file_path,
        inline_config: None,
    });

    match res {
        Ok(p) => Ok(FormatResult::Success {
            formatted_content: p.into_code(),
        }),
        Err(e) => Ok(FormatResult::Error {
            error: e.to_string(),
        }),
    }
}

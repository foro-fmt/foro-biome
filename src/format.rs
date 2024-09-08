use anyhow::{anyhow, Result};
use biome_configuration::ConfigurationPathHint;
use biome_fs::{BiomePath, FileSystem, OsFileSystem};
use biome_service::configuration::{
    load_configuration, LoadedConfiguration, PartialConfigurationExt,
};
use biome_service::workspace::{
    FeaturesBuilder, FormatFileParams, OpenFileParams, RegisterProjectFolderParams,
    SupportsFeatureParams, UpdateSettingsParams,
};
use biome_service::{workspace, DynRef};
use std::path::{Path, PathBuf};

fn resolve_manifest(fs: &DynRef<'_, dyn FileSystem>) -> Result<Option<(BiomePath, String)>> {
    match fs.auto_search(
        &fs.working_directory().unwrap_or_default(),
        &["package.json"],
        false,
    ) {
        Ok(Some(result)) => Ok(Some((BiomePath::new(result.file_path), result.content))),
        Ok(None) => Ok(None),
        Err(e) => Err(anyhow!(e)),
    }
}

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
    let workspace = workspace::server();

    let fs = OsFileSystem::new(current_dir.clone());
    let dfs: &DynRef<'_, dyn FileSystem> = &DynRef::Owned(Box::new(fs));

    let LoadedConfiguration {
        configuration: biome_configuration,
        directory_path: configuration_path,
        ..
    } = load_configuration(dfs, ConfigurationPathHint::None)?;

    workspace.register_project_folder(RegisterProjectFolderParams {
        set_as_current_workspace: true,
        path: Some(current_dir),
    })?;

    let vcs_base_path = configuration_path.or(dfs.working_directory());
    let (vcs_base_path, gitignore_matches) =
        biome_configuration.retrieve_gitignore_matches(dfs, vcs_base_path.as_deref())?;

    let manifest_data = resolve_manifest(dfs)?;

    if let Some(manifest_data) = manifest_data {
        workspace.set_manifest_for_project(manifest_data.into())?;
    }

    workspace.update_settings(UpdateSettingsParams {
        workspace_directory: dfs.working_directory(),
        configuration: biome_configuration,
        vcs_base_path,
        gitignore_matches,
    })?;

    let file_features = workspace.file_features(SupportsFeatureParams {
        path: BiomePath::new(Path::new(&given_file_name)),
        features: FeaturesBuilder::new().with_formatter().build(),
    })?;

    if !file_features.is_supported() {
        return Ok(FormatResult::Ignored);
    }

    workspace.open_file(OpenFileParams {
        path: BiomePath::new(Path::new(&given_file_name)),
        content: file_content,
        version: 0,
        document_file_source: None,
    })?;

    let res = workspace.format_file(FormatFileParams {
        path: BiomePath::new(Path::new(&given_file_name)),
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

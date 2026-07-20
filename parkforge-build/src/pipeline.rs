use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use globset::Glob;
use parkforge_model::{MANIFEST_FILE_NAME, VirtualPath};

use crate::staging;
use crate::{BuildConfig, BuildRequest, BuildRule, Error, Result, compiler};

pub(crate) fn build(request: &BuildRequest<'_>) -> Result<()> {
    request.config.validate()?;
    let parent = request.build_root.parent().ok_or_else(|| Error::Io {
        path: request.build_root.to_path_buf(),
        source: io::Error::other("build path has no parent directory"),
    })?;
    fs::create_dir_all(parent).map_err(|source| Error::Io {
        path: parent.to_path_buf(),
        source,
    })?;

    let staging = parkforge_model::StagingDirectory::create(
        parent,
        &format!("{}.build-staging", request.game_id),
    )
    .map_err(|source| Error::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    copy_tree_excluding(
        request.original_root,
        staging.path(),
        Some(MANIFEST_FILE_NAME),
    )?;
    apply_source(request.source_root, staging.path(), request.config)?;
    staging::replace(staging, request.build_root)
}

fn apply_source(source_root: &Path, build_root: &Path, config: &BuildConfig) -> Result<()> {
    if !source_root.exists() {
        return Ok(());
    }
    let actions = plan_source(source_root, config)?;
    for action in actions {
        let output = action.output.to_path_under(build_root);
        compile_source(action.rule, &action.source, &output)?;
    }
    Ok(())
}

fn plan_source<'a>(source_root: &Path, config: &'a BuildConfig) -> Result<Vec<BuildAction<'a>>> {
    let rules = compiled_rules(config)?;
    let mut outputs = HashMap::new();
    let mut actions = Vec::new();
    for source_path in walk_files(source_root)? {
        let relative = relative_virtual_path(source_root, &source_path)?;
        let matching: Vec<_> = rules
            .iter()
            .filter(|rule| rule.matcher.is_match(relative.as_str()))
            .collect();
        match matching.as_slice() {
            [] => return Err(Error::UnmatchedBuildInput { path: source_path }),
            [rule] => {
                let output = output_path(&relative, rule.rule.output_extension.as_deref())?;
                actions.push(BuildAction {
                    output,
                    source: source_path,
                    rule: rule.rule,
                });
            }
            many => {
                return Err(Error::AmbiguousBuildRule {
                    path: source_path,
                    rules: many.iter().map(|rule| rule.rule.name.clone()).collect(),
                });
            }
        }
    }
    for action in &actions {
        if let Some(first) = outputs.insert(action.output.clone(), action.source.clone()) {
            return Err(Error::DuplicateBuildOutput {
                path: action.output.to_path_under(source_root),
                first,
                second: action.source.clone(),
            });
        }
    }
    Ok(actions)
}

fn compiled_rules(config: &BuildConfig) -> Result<Vec<CompiledBuildRule<'_>>> {
    config
        .rules
        .iter()
        .map(|rule| {
            Glob::new(&rule.source)
                .map(|glob| CompiledBuildRule {
                    rule,
                    matcher: glob.compile_matcher(),
                })
                .map_err(|error| Error::InvalidBuildRule {
                    rule: rule.name.clone(),
                    message: format!("invalid source pattern: {error}"),
                })
        })
        .collect()
}

fn compile_source(rule: &BuildRule, input: &Path, output: &Path) -> Result<()> {
    compiler::find(&rule.compiler).map_or_else(
        || {
            Err(Error::CompilerUnavailable {
                rule: rule.name.clone(),
                compiler: rule.compiler.clone(),
            })
        },
        |compiler| compiler.compile(input, output),
    )
}

struct CompiledBuildRule<'a> {
    rule: &'a BuildRule,
    matcher: globset::GlobMatcher,
}

struct BuildAction<'a> {
    source: PathBuf,
    output: VirtualPath,
    rule: &'a BuildRule,
}

fn output_path(input: &VirtualPath, extension: Option<&str>) -> Result<VirtualPath> {
    let Some(extension) = extension else {
        return Ok(input.clone());
    };
    let (stem, _) = input
        .as_str()
        .rsplit_once('.')
        .ok_or_else(|| Error::InvalidBuildRule {
            rule: input.to_string(),
            message: "a transformed source file must have an extension".to_owned(),
        })?;
    VirtualPath::new(format!("{stem}.{extension}")).map_err(Error::from)
}

fn copy_tree_excluding(
    source: &Path,
    target: &Path,
    excluded_root_file: Option<&str>,
) -> Result<()> {
    fs::create_dir_all(target).map_err(|source_error| Error::Io {
        path: target.to_path_buf(),
        source: source_error,
    })?;
    for entry in fs::read_dir(source).map_err(|source_error| Error::Io {
        path: source.to_path_buf(),
        source: source_error,
    })? {
        let entry = entry.map_err(|source_error| Error::Io {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        let path = entry.path();
        let file_name = entry.file_name();
        if excluded_root_file.is_some_and(|name| file_name == name) {
            continue;
        }
        let output = target.join(&file_name);
        let file_type = entry.file_type().map_err(|source_error| Error::Io {
            path: path.clone(),
            source: source_error,
        })?;
        if file_type.is_dir() {
            copy_tree_excluding(&path, &output, None)?;
        } else if file_type.is_file() {
            copy_baseline_file(&path, &output)?;
        } else {
            return Err(Error::UnsupportedFilesystemEntry { path });
        }
    }
    Ok(())
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root).map_err(|source| Error::Io {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| Error::Io {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            files.extend(walk_files(&path)?);
        } else if file_type.is_file() {
            files.push(path);
        } else {
            return Err(Error::UnsupportedFilesystemEntry { path });
        }
    }
    files.sort();
    Ok(files)
}

fn relative_virtual_path(root: &Path, path: &Path) -> Result<VirtualPath> {
    let relative = path.strip_prefix(root).map_err(|_error| Error::Io {
        path: path.to_path_buf(),
        source: io::Error::other("source file is outside its source root"),
    })?;
    let value = relative
        .to_str()
        .ok_or_else(|| Error::CompilerFailed {
            compiler: "build".to_owned(),
            path: path.to_path_buf(),
            message: "source file path contains non-UTF-8 components".to_owned(),
        })?
        .replace(std::path::MAIN_SEPARATOR, "/");
    VirtualPath::new(value).map_err(Error::from)
}

fn copy_baseline_file(source: &Path, target: &Path) -> Result<()> {
    fs::copy(source, target).map_err(|source_error| Error::Io {
        path: target.to_path_buf(),
        source: source_error,
    })?;
    Ok(())
}

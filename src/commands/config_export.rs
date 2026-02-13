//! Configuration export command for nu_plugin_secret

use std::path::PathBuf;

use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Record, Signature, SyntaxShape, Type, Value,
};

use crate::config::ConfigManager;

/// Command to export configuration to a file
pub struct SecretConfigExportCommand;

/// Build the result record summarising a successful configuration export.
fn build_export_result(
    manager: &ConfigManager,
    export_path: &std::path::Path,
    span: nu_protocol::Span,
) -> Record {
    let mut record = Record::new();

    record.push(
        "status",
        Value::string("Configuration exported successfully", span),
    );
    record.push(
        "export_path",
        Value::string(export_path.to_string_lossy().to_string(), span),
    );

    if let Some(source_path) = crate::config::get_config_file_path() {
        record.push(
            "source_config",
            Value::string(source_path.to_string_lossy().to_string(), span),
        );
    }

    record.push(
        "redaction_template",
        Value::string(manager.config().redaction.get_redaction_template(), span),
    );
    record.push(
        "security_level",
        Value::string(
            format!("{:?}", manager.config().security.level).to_lowercase(),
            span,
        ),
    );

    record
}

impl PluginCommand for SecretConfigExportCommand {
    type Plugin = crate::SecretPlugin;

    fn name(&self) -> &str {
        "secret config export"
    }

    fn description(&self) -> &str {
        "Export secret plugin configuration to a file"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::Record(Box::new([])))])
            .required(
                "path",
                SyntaxShape::Filepath,
                "Path to export configuration file",
            )
            .switch(
                "overwrite",
                "Overwrite existing file if it exists",
                Some('o'),
            )
            .category(Category::Custom("secret".into()))
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "secret config export backup_config.toml",
                description: "Export configuration to backup_config.toml",
                result: None,
            },
            Example {
                example: "secret config export --overwrite production_config.toml",
                description: "Export configuration, overwriting existing file",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let span = call.head;

        // Get the export path
        let export_path: String = call.req(0)?;
        let export_path = PathBuf::from(&export_path);

        // Check if file exists and overwrite flag
        if export_path.exists() && !call.has_flag("overwrite")? {
            return Err(LabeledError::new("File Exists").with_label(
                "Export file already exists. Use --overwrite to replace it",
                span,
            ));
        }

        // Load current configuration
        let manager = ConfigManager::load().map_err(|e| {
            LabeledError::new("Configuration Error")
                .with_label(format!("Failed to load configuration: {}", e), span)
        })?;

        // Export configuration to specified path
        manager.save_to_path(&export_path).map_err(|e| {
            LabeledError::new("Export Failed")
                .with_label(format!("Failed to export configuration: {}", e), span)
        })?;

        let record = build_export_result(&manager, &export_path, span);

        Ok(PipelineData::Value(Value::record(record, span), None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let command = SecretConfigExportCommand;
        assert_eq!(command.name(), "secret config export");
    }

    #[test]
    fn test_signature() {
        let command = SecretConfigExportCommand;
        let signature = command.signature();

        assert_eq!(signature.name, "secret config export");
        assert!(!signature.required_positional.is_empty());
        assert_eq!(signature.required_positional[0].name, "path");
    }
}

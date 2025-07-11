use {
    super::{
        biome,
        cargo_json,
        cpp,
        eslint,
        nextest,
        python,
        standard,
        swift,
    },
    crate::*,
    serde::{
        Deserialize,
        Serialize,
    },
};

/// A stateless operator building a report from a list of command output lines.
///
/// Implementation routing will probably change at some point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyzerRef {
    #[default]
    Standard,
    CargoJson,
    Nextest,
    Eslint,
    Biome,
    PythonPytest,
    PythonRuff,
    PythonUnittest,
    Cpp,
    CppDoctest,
    SwiftBuild,
    SwiftLint,
}

impl AnalyzerRef {
    pub fn create_analyzer(self) -> Box<dyn Analyzer> {
        match self {
            Self::Standard => Box::new(standard::StandardAnalyzer::default()),
            Self::Nextest => Box::new(nextest::NextestAnalyzer::default()),
            Self::Eslint => Box::new(eslint::EslintAnalyzer::default()),
            Self::Biome => Box::new(biome::BiomeAnalyzer::default()),
            Self::PythonUnittest => Box::new(python::unittest::UnittestAnalyzer::default()),
            Self::PythonPytest => Box::new(python::pytest::PytestAnalyzer::default()),
            Self::PythonRuff => Box::new(python::ruff::RuffAnalyzer::default()),
            Self::CargoJson => Box::new(cargo_json::CargoJsonAnalyzer::default()),
            Self::Cpp => Box::new(cpp::CppAnalyzer::default()),
            Self::CppDoctest => Box::new(cpp::CppDoctestAnalyzer::default()),
            Self::SwiftBuild => Box::new(swift::build::SwiftBuildAnalyzer::default()),
            Self::SwiftLint => Box::new(swift::lint::SwiftLintAnalyzer::default()),
        }
    }
}

pub trait Analyzer {
    fn start(
        &mut self,
        mission: &Mission,
    );

    fn receive_line(
        &mut self,
        line: CommandOutputLine,
        command_output: &mut CommandOutput,
    );

    fn build_report(&mut self) -> anyhow::Result<Report>;
}

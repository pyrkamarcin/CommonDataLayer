use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};

const ORDERING_NUMBER_LEN: usize = 4;

pub fn check_docs() -> Result<()> {
    check_rfc()?;

    Ok(())
}

fn check_rfc() -> Result<()> {
    let rfc_files = xshell::read_dir("docs/rfc")?;

    let mut rfcs = rfc_files
        .iter()
        .filter_map(|f| f.file_name())
        .filter_map(|f| f.to_str())
        .filter(|f| f.ends_with(".md"))
        .map(|f| f.trim_end_matches(".md"));

    let mut validator = RFCValidator::default();
    if !rfcs.all(|f| validator.validate_rfc(f)) {
        std::process::exit(1);
    }

    Ok(())
}

#[derive(Default)]
struct RFCValidator<'a> {
    ordering_numbers: HashMap<u16, &'a str>,
}

impl<'a> RFCValidator<'a> {
    fn validate_rfc(&mut self, filename: &'a str) -> bool {
        match self
            .validate_rfc_inner(filename)
            .with_context(|| format!("Validating RFC: `{}` failed", filename))
        {
            Ok(()) => true,
            Err(e) => {
                eprintln!("{:?}", e);
                false
            }
        }
    }

    fn validate_rfc_inner(&mut self, filename: &'a str) -> Result<()> {
        let mut split = filename.split('_');
        let ordering_number = split.next().context("Expected ordering number")?;
        self.validate_ordering_number(ordering_number, filename)?;

        let _name = split.next().context("Expected name")?;
        let version_number = split.last().context("Expected version number")?;
        self.validate_version_number(version_number)?;
        Ok(())
    }

    fn validate_version_number(&mut self, version_number: &str) -> Result<()> {
        if !version_number.chars().all(|c| c.is_ascii_digit()) {
            bail!("Version number must contain only digits");
        }

        Ok(())
    }

    fn validate_ordering_number(&mut self, ordering_number: &str, filename: &'a str) -> Result<()> {
        if ordering_number.len() != ORDERING_NUMBER_LEN {
            bail!("Ordering number must have {} digits", ORDERING_NUMBER_LEN);
        }
        if !ordering_number.chars().all(|c| c.is_ascii_digit()) {
            bail!("Ordering number must contain only digits");
        }
        let ordering_number = ordering_number.parse()?;

        if let Some(existing) = self.ordering_numbers.get(&ordering_number) {
            return Err(
                anyhow!("RFC with this ordering number already exist: {}", existing)
                    .context("Ordering number must be unique"),
            );
        }

        self.ordering_numbers.insert(ordering_number, filename);

        Ok(())
    }
}

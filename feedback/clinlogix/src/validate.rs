use crate::model::{Issue, Severity};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("File I/O error: {0}")]
    File(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("FHIR resource requires a 'resourceType' field")]
    MissingResourceType,
    #[error("Validation failed with status {0}")]
    ValidationFailed(StatusCode),
}

type Result<T> = std::result::Result<T, ValidationError>;

#[derive(Deserialize, Debug)]
pub struct OperationOutcome {
    #[serde(default)]
    issue: Vec<Issue>,
}

pub async fn run_validate(fhir_file: &str, base_url: &str) -> Result<()> {
    let raw_json = fs::read_to_string(fhir_file)?;
    let fhir_value = parse_fhir_resource(&raw_json)?;

    let resource_type = fhir_value
        .get("resourceType")
        .and_then(|v| v.as_str())
        .ok_or(ValidationError::MissingResourceType)?;

    let url = build_validation_url(base_url, resource_type);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Accept", "application/fhir+json")
        .header("Content-Type", "application/fhir+json")
        .body(raw_json)
        .send()
        .await?;

    let status = response.status();
    let outcome: OperationOutcome = response.json().await?;

    print_validation_report(fhir_file, base_url, status, &outcome.issue);

    if status.is_success() && outcome.issue.iter().all(|i| !matches!(i.severity, Severity::Error | Severity::Fatal)) {
        Ok(())
    } else {
        Err(ValidationError::ValidationFailed(status))
    }
}

fn parse_fhir_resource(raw_json: &str) -> Result<Value> {
    Ok(serde_json::from_str(raw_json)?)
}

fn build_validation_url(base_url: &str, resource_type: &str) -> String {
    format!(
        "{}/{}/$validate",
        base_url.trim_end_matches('/'),
        resource_type
    )
}

fn print_validation_report(
    fhir_file: &str,
    base_url: &str,
    status: StatusCode,
    issues: &[Issue],
) {
    let (errors, warnings): (Vec<_>, Vec<_>) =
        issues.iter().partition(|i| matches!(i.severity, Severity::Error | Severity::Fatal));

    println!("FHIR Validation");
    println!("--------------");
    println!("File: {}", fhir_file);
    println!("Base: {}", base_url);
    println!("HTTP: {}", status);

    if errors.is_empty() && status.is_success() {
        println!("Result: PASS ✅");
    } else {
        println!("Result: FAIL ❌");
    }

    if !errors.is_empty() {
        println!("\nErrors:");
        for (i, e) in errors.iter().enumerate() {
            println!("{}. [{:?}] {} - {}", i + 1, e.severity, e.code, e.diagnostics);
             if let Some(details) = &e.details {
                println!("   Details: {}", details.text);
            }
        }
    }

    if !warnings.is_empty() {
        println!("\nWarnings:");
        for (i, w) in warnings.iter().enumerate() {
            println!("{}. [{:?}] {} - {}", i + 1, w.severity, w.code, w.diagnostics);
            if let Some(details) = &w.details {
                println!("   Details: {}", details.text);
            }
        }
    }

    if issues.is_empty() {
        println!("\nNo issues reported.");
    }
}

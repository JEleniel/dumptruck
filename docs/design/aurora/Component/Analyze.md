# analyze

- Type: component
- ID: analyze
- UUID: 2b3c4d5e-6f7a-4b8c-9d0e-1f2a3b4c5d6e
- Version: 1.3.0
- JSON: [analyze-2b3c4d5e-6f7a-4b8c-9d0e-1f2a3b4c5d6e.json](./analyze-2b3c4d5e-6f7a-4b8c-9d0e-1f2a3b4c5d6e.json)

## Description

Bulk ingestion + normalization + detection + enrichment pipeline for breach/threat-intel datasets, optimized for streaming and scale.

## Attributes

```json
{
	"rust_module": "src/analyze"
}
```

## Links

- implements: [CLI and HTTPS API modes](../Feature/CliAndHttpsApiModes.md) ([JSON](../Feature/cli_and_https_api_modes-7b6b8d4b-1c6f-4c0e-9c08-5c6a1f5a8b2f.json))
- implements: [Normalization and deduplication engine](../Feature/NormalizationAndDeduplicationEngine.md) ([JSON](../Feature/normalization_and_deduplication_engine-6e6e5d5a-2f0b-4b2a-9e9a-0c2c4fb1d46b.json))
- implements: [Intelligence enrichment and risk scoring](../Feature/IntelligenceEnrichmentAndRiskScoring.md) ([JSON](../Feature/intelligence_enrichment_and_risk_scoring-a58fd4d6-01e3-4a6f-8a79-3f9e64b2b8b0.json))
- implements: [NPI and credential detection](../Feature/NpiAndCredentialDetection.md) ([JSON](../Feature/npi_and_credential_detection-8c1f7f33-0a4e-4c1b-8a9f-1fdb3bbd4e6b.json))
- generates: [Analysis Report](../Artifact/AnalysisReport.md) ([JSON](../Artifact/analysis_report-b549f9ba-f06a-4f99-adf0-3b65ed8bab95.json))
- generates: [Learned Data](../Artifact/LearnedData.md) ([JSON](../Artifact/learned_data-3d85ffd1-4613-4283-96b1-9d059b2e5687.json))

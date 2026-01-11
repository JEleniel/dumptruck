# Safe streaming ingestion

- Type: feature
- ID: safe_streaming_ingestion
- UUID: 2c4a8d90-9b59-4ac0-8f2e-29c2e8f764a1
- Version: 1.0.0
- JSON: [safe_streaming_ingestion-2c4a8d90-9b59-4ac0-8f2e-29c2e8f764a1.json](./safe_streaming_ingestion-2c4a8d90-9b59-4ac0-8f2e-29c2e8f764a1.json)

## Description

Ingest large delimited datasets (CSV/TSV/PSV) via memory-efficient streaming with conservative safety checks (binary rejection, executable bit checks on Posix, UTF-8 validation) and resilient handling of malformed input.

## Attributes

```json
{
	"source": "README.md"
}
```

## Links

- satisfies: [Breach Data Analysis Tool](../Requirement/BreachDataAnalysisTool.md) ([JSON](../Requirement/breach_data_analysis_tool-aaf48cba-55e2-4d9d-89f8-787461af1416.json))

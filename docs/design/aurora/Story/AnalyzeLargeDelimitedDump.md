# Analyze a large delimited dump safely

- Type: story
- ID: analyze_large_delimited_dump
- UUID: a5e1f8c8-7d58-4f59-8b3d-3a1b6b8d6c2f
- Version: 1.0.0
- JSON: [analyze_large_delimited_dump-a5e1f8c8-7d58-4f59-8b3d-3a1b6b8d6c2f.json](./analyze_large_delimited_dump-a5e1f8c8-7d58-4f59-8b3d-3a1b6b8d6c2f.json)

## Description

A User ingests a large CSV/TSV/PSV breach dataset. The system streams records with conservative safety checks (binary rejection, exec-bit checks where applicable, UTF-8 validation) and produces a usable analysis without exhausting memory.

## Attributes

```json
{
	"source": "README.md"
}
```

## Links

- explains: [Safe streaming ingestion](../Feature/SafeStreamingIngestion.md) ([JSON](../Feature/safe_streaming_ingestion-2c4a8d90-9b59-4ac0-8f2e-29c2e8f764a1.json))

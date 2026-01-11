# database

- Type: component
- ID: database
- UUID: 1e7b3c2d-0f1a-4b6d-9c7e-2a1b3c4d5e6f
- Version: 1.3.0
- JSON: [database-1e7b3c2d-0f1a-4b6d-9c7e-2a1b3c4d5e6f.json](./database-1e7b3c2d-0f1a-4b6d-9c7e-2a1b3c4d5e6f.json)

## Description

Persistence and historical intelligence storage layer (SQLite + schema/migrations), storing only non-reversible hashes for privacy-preserving recall.

## Attributes

```json
{
	"rust_module": "src/database"
}
```

## Links

- implements: [CLI and HTTPS API modes](../Feature/CliAndHttpsApiModes.md) ([JSON](../Feature/cli_and_https_api_modes-7b6b8d4b-1c6f-4c0e-9c08-5c6a1f5a8b2f.json))
- implements: [Privacy-first persistence](../Feature/PrivacyFirstPersistence.md) ([JSON](../Feature/privacy_first_persistence-0e6b2c44-6b2c-4b1b-9e34-9e52a5c3f6a4.json))
- implements: [Cross-instance sharing (import/export)](../Feature/CrossInstanceSharingImportExport.md) ([JSON](../Feature/cross_instance_sharing_import_export-3f0b6c1a-0c5a-4a1e-8a5d-4d7f0f8c2b4a.json))
- depends_on: [SQLite](../Data_store/Sqlite.md) ([JSON](../Data_store/sqlite-05b01a6d-22f1-4fc0-af27-be5cde0be72f.json))

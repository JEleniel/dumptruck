# Design

This folder contains design artifacts for Dumptruck. The primary source of truth is an Aurora model stored as JSON cards.

## Quick Links

- Human mission: [docs/design/aurora/DumptruckMission.md](./aurora/DumptruckMission.md)
- Diagrams:
    + [docs/design/diagrams/EverythingView.md](./diagrams/EverythingView.md)
    + [docs/design/diagrams/RequirementsView.md](./diagrams/RequirementsView.md)
    + [docs/design/diagrams/ComponentView.md](./diagrams/ComponentView.md)
    + [docs/design/diagrams/StoryView.md](./diagrams/StoryView.md)

## Structure

- Aurora model cards (JSON): [docs/design/aurora](./aurora/)
- Aurora human cards (Markdown): [docs/design/aurora](./aurora/)
- Aurora schema: [docs/design/aurora/Aurora.schema.json](./aurora/Aurora.schema.json)

## Aurora Cards

### Mission

- Dumptruck Mission: [Human](./aurora/DumptruckMission.md) [JSON](./aurora/mission-5477c32b-28fb-4a18-b359-89437520fe02.json)

### Applications

- Dumptruck: [Human](./aurora/Application/DumptruckApplication.md) [JSON](./aurora/Application/dumptruck_application-5c1f4f1d-5cfe-4e46-b117-7102662e40f0.json)

### Actors

- User: [Human](./aurora/Actor/User.md) [JSON](./aurora/Actor/user-2f4c0a60-1a18-4e21-9d16-1d1e396f3d4a.json)

### Components

- analyze: [Human](./aurora/Component/Analyze.md) [JSON](./aurora/Component/analyze-2b3c4d5e-6f7a-4b8c-9d0e-1f2a3b4c5d6e.json)
- api: [Human](./aurora/Component/Api.md) [JSON](./aurora/Component/api-5e6f7a8b-9c0d-4e1f-2a3b-4c5d6e7f8a9b.json)
- cli: [Human](./aurora/Component/Cli.md) [JSON](./aurora/Component/cli-9c1d2e3f-4a5b-4c6d-8e7f-0a1b2c3d4e5f.json)
- configuration: [Human](./aurora/Component/Configuration.md) [JSON](./aurora/Component/configuration-6f7a8b9c-0d1e-4f2a-3b4c-5d6e7f8a9b0c.json)
- datafile: [Human](./aurora/Component/Datafile.md) [JSON](./aurora/Component/datafile-12c1aa8e-b0b1-4db5-a3e0-1b6e8b8704f5.json)
- database: [Human](./aurora/Component/Database.md) [JSON](./aurora/Component/database-1e7b3c2d-0f1a-4b6d-9c7e-2a1b3c4d5e6f.json)
- server: [Human](./aurora/Component/Server.md) [JSON](./aurora/Component/server-3c4d5e6f-7a8b-4c9d-0e1f-2a3b4c5d6e7f.json)
- status: [Human](./aurora/Component/Status.md) [JSON](./aurora/Component/status-7a8b9c0d-1e2f-4a3b-4c5d-6e7f8a9b0c1d.json)
- util: [Human](./aurora/Component/Util.md) [JSON](./aurora/Component/util-4d5e6f7a-8b9c-4d0e-1f2a-3b4c5d6e7f8a.json)

### Data Stores

- SQLite: [Human](./aurora/Data_store/Sqlite.md) [JSON](./aurora/Data_store/sqlite-05b01a6d-22f1-4fc0-af27-be5cde0be72f.json)

### Artifacts

- Analysis Report: [Human](./aurora/Artifact/AnalysisReport.md) [JSON](./aurora/Artifact/analysis_report-b549f9ba-f06a-4f99-adf0-3b65ed8bab95.json)
- Learned Data: [Human](./aurora/Artifact/LearnedData.md) [JSON](./aurora/Artifact/learned_data-3d85ffd1-4613-4283-96b1-9d059b2e5687.json)

### Capabilities

- Run tool in secure environment: [Human](./aurora/Capability/RunToolInSecureEnvironment.md) [JSON](./aurora/Capability/run_tool_in_secure_environment-8d2d3b1a-4c5e-4f6a-8b7c-9d0e1f2a3b4c.json)

### Processes

- Setup, Use, and Tear Down Secure Environment: [Human](./aurora/Process/SetupUseAndTearDownSecureEnvironment.md) [JSON](./aurora/Process/setup_use_and_tear_down_secure_environment-c3f48e7a-2b91-4ed5-a45c-1f2b3c4d5e6f.json)

### Constraints

- Data Protection: [Human](./aurora/Constraint/DataProtection.md) [JSON](./aurora/Constraint/data_protection-2d3e4f5a-6b7c-4d8e-9f0a-1b2c3d4e5f6a.json)
- Secure File Deletion: [Human](./aurora/Constraint/SecureFileDeletion.md) [JSON](./aurora/Constraint/secure_file_deletion-0b7b2d5b-9d45-4f3a-9d9c-1c2b3a4d5e6f.json)
- TLS 1.3: [Human](./aurora/Constraint/Tls13.md) [JSON](./aurora/Constraint/tls_1_3-1c2d3e4f-5a6b-4c7d-8e9f-0a1b2c3d4e5f.json)

### Features

- CLI and HTTPS API modes: [Human](./aurora/Feature/CliAndHttpsApiModes.md) [JSON](./aurora/Feature/cli_and_https_api_modes-7b6b8d4b-1c6f-4c0e-9c08-5c6a1f5a8b2f.json)
- Cross-instance sharing (import/export): [Human](./aurora/Feature/CrossInstanceSharingImportExport.md) [JSON](./aurora/Feature/cross_instance_sharing_import_export-3f0b6c1a-0c5a-4a1e-8a5d-4d7f0f8c2b4a.json)
- Intelligence enrichment and risk scoring: [Human](./aurora/Feature/IntelligenceEnrichmentAndRiskScoring.md) [JSON](./aurora/Feature/intelligence_enrichment_and_risk_scoring-a58fd4d6-01e3-4a6f-8a79-3f9e64b2b8b0.json)
- Normalization and deduplication engine: [Human](./aurora/Feature/NormalizationAndDeduplicationEngine.md) [JSON](./aurora/Feature/normalization_and_deduplication_engine-6e6e5d5a-2f0b-4b2a-9e9a-0c2c4fb1d46b.json)
- NPI and credential detection: [Human](./aurora/Feature/NpiAndCredentialDetection.md) [JSON](./aurora/Feature/npi_and_credential_detection-8c1f7f33-0a4e-4c1b-8a9f-1fdb3bbd4e6b.json)
- Privacy-first persistence: [Human](./aurora/Feature/PrivacyFirstPersistence.md) [JSON](./aurora/Feature/privacy_first_persistence-0e6b2c44-6b2c-4b1b-9e34-9e52a5c3f6a4.json)
- Safe streaming ingestion: [Human](./aurora/Feature/SafeStreamingIngestion.md) [JSON](./aurora/Feature/safe_streaming_ingestion-2c4a8d90-9b59-4ac0-8f2e-29c2e8f764a1.json)

### Stories

- Analyze a large delimited dump safely: [Human](./aurora/Story/AnalyzeLargeDelimitedDump.md) [JSON](./aurora/Story/analyze_large_delimited_dump-a5e1f8c8-7d58-4f59-8b3d-3a1b6b8d6c2f.json)
- Detect NPI and credential risk: [Human](./aurora/Story/DetectNpiAndCredentialRisk.md) [JSON](./aurora/Story/detect_npi_and_credential_risk-8d4e6c8c-4e8d-4d79-bd0c-91f6b8e5c2d3.json)
- Enrich findings and score risk: [Human](./aurora/Story/EnrichFindingsAndScoreRisk.md) [JSON](./aurora/Story/enrich_findings_and_score_risk-3b2a1c0d-9e8f-4a7b-8c6d-5e4f3a2b1c0d.json)
- Normalize and deduplicate messy records: [Human](./aurora/Story/NormalizeAndDeduplicateMessyRecords.md) [JSON](./aurora/Story/normalize_and_deduplicate_messy_records-2f4f1f0d-6d6b-45d7-a1d7-0b8c4c9b5a17.json)
- Persist intelligence without storing raw data: [Human](./aurora/Story/PersistIntelligencePrivacyFirst.md) [JSON](./aurora/Story/persist_intelligence_privacy_first-6a3d2c9b-7f0e-4a4d-9e4b-1c2d3e4f5a6b.json)
- Run as CLI or secure HTTPS API: [Human](./aurora/Story/RunAsCliOrSecureHttpsApi.md) [JSON](./aurora/Story/run_as_cli_or_secure_https_api-4c5d6e7f-8a9b-4c0d-8e1f-2a3b4c5d6e7f.json)
- Share intelligence between instances: [Human](./aurora/Story/ShareIntelligenceBetweenInstances.md) [JSON](./aurora/Story/share_intelligence_between_instances-9a8b7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c6d.json)

### Drivers

- Strengthen CTI program maturity: [Human](./aurora/Driver/StrengthenCtiProgramMaturity.md) [JSON](./aurora/Driver/strengthen_cti_program_maturity-df3e1636-d23c-4517-9464-8f04c7638c1a.json)
- Enhance IOC ingestion and curation: [Human](./aurora/Driver/EnhanceIocIngestionAndCuration.md) [JSON](./aurora/Driver/enhance_ioc_ingestion_and_curation-fcc2b3a2-71c6-4338-aae5-cd59aefcdc67.json)
- Operationalize the Threat Intelligence Platform (TIP): [Human](./aurora/Driver/OperationalizeTip.md) [JSON](./aurora/Driver/operationalize_tip-da524b38-aee3-4aa1-8c99-48931e47f59b.json)
- Rationalize and consolidate the CTI stack: [Human](./aurora/Driver/RationalizeAndConsolidateCtiStack.md) [JSON](./aurora/Driver/rationalize_and_consolidate_cti_stack-d09eab72-a640-4396-bada-61cd259b75f9.json)
- Develop a CTI workforce maturity plan: [Human](./aurora/Driver/DevelopCtiWorkforceMaturityPlan.md) [JSON](./aurora/Driver/develop_cti_workforce_maturity_plan-a11962af-32b2-4b09-a936-441d1364452f.json)

### Requirements

- Breach Data Analysis Tool: [Human](./aurora/Requirement/BreachDataAnalysisTool.md) [JSON](./aurora/Requirement/breach_data_analysis_tool-aaf48cba-55e2-4d9d-89f8-787461af1416.json)

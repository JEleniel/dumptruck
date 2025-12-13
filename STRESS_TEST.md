# Dumptruck Server Stress Test

A comprehensive stress testing utility for the Dumptruck HTTP/2 server with TLS 1.3+ support.

## Overview

The stress test tool (`stress-test` binary) submits concurrent requests to the server using all available test fixtures and provides detailed performance metrics including:

- **Throughput**: Requests per second
- **Latency**: Min, average, P95, P99, and maximum response times
- **Success Rate**: Percentage of successful vs failed requests
- **Concurrency Testing**: Configurable concurrent request handling

## Building

```bash
cargo build --bin stress-test --release
```

The binary will be created at `target/release/stress-test`.

## Running the Stress Test

### Basic Usage

```bash
# Start the server in one terminal
./target/debug/dumptruck server \
  --cert tests/fixtures/tls.crt \
  --key tests/fixtures/tls.key \
  --oauth-client-id test-client \
  --oauth-client-secret test-secret \
  --oauth-token-endpoint https://oauth.example.com/token \
  -vvv

# In another terminal, run the stress test
./target/release/stress-test
```

### Configuration via Environment Variables

The stress test is configured using environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `STRESS_TEST_URL` | `https://localhost:8443` | Server URL |
| `STRESS_TEST_TOKEN` | `test-token-12345` | OAuth bearer token |
| `STRESS_TEST_CONCURRENT` | `10` | Number of concurrent requests |
| `STRESS_TEST_REQUESTS` | `100` | Total number of requests to send |
| `STRESS_TEST_VERBOSE` | (disabled) | Enable verbose output (set to any value to enable) |

### Examples

```bash
# Test with 20 concurrent requests, 500 total requests
STRESS_TEST_CONCURRENT=20 STRESS_TEST_REQUESTS=500 ./target/release/stress-test

# Verbose mode with custom URL
STRESS_TEST_URL=https://myserver.local:8443 STRESS_TEST_VERBOSE=1 ./target/release/stress-test

# Heavy load test: 50 concurrent, 5000 requests
STRESS_TEST_CONCURRENT=50 STRESS_TEST_REQUESTS=5000 ./target/release/stress-test
```

## Test Fixtures

The stress test automatically loads all files from `tests/fixtures/` directory. Currently supports:

- CSV files (`.csv`)
- TSV files (`.tsv`)
- JSON files (`.json`)
- YAML files (`.yaml`)
- And other text-based formats

The test submits each fixture in a round-robin fashion across concurrent workers.

## Output Format

### Summary Statistics

```text
=== Dumptruck Server Stress Test ===
Server URL: https://localhost:8443
Concurrent Requests: 10
Total Requests Target: 100

Loaded 20 test fixtures
  - test_creds_100.csv (2845 bytes)
  - test_creds_mixed.csv (1234 bytes)
  ...

Waiting for 5 pending requests...

=== Results ===
Total Requests: 100
Successful: 100 (100.0%)
Failed: 0 (0.0%)

Throughput: 1234.56 requests/second

Latency (ms):
  Min:  0.45
  Avg:  8.12
  P95:  15.23
  P99:  22.50
  Max:  45.67

Total Time: 0.08s
```

### Verbose Mode

When `STRESS_TEST_VERBOSE=1`, each request is logged:

```text
[0] OK - test_creds_100.csv → job_id: 550e8400-e29b-41d4-a716-446655440000 (2.34ms)
[1] OK - test_creds_mixed.csv → job_id: 550e8400-e29b-41d4-a716-446655440001 (3.12ms)
[2] FAIL - malformed_missing_quote.csv - Server returned 400 (5.67ms)
...
```

## Parallel Processing

The Dumptruck server implements a parallel job processor that:

1. **Worker Spawning**: Automatically spawns N workers (default: number of CPU cores)
2. **Job Queue**: Each worker polls the job queue for queued jobs
3. **Concurrent Processing**: Multiple jobs are processed simultaneously
4. **Status Updates**: Each job's status transitions through Queued → Processing → Completed/Failed
5. **Error Handling**: Failed jobs are tracked with error messages

### Worker Behavior

- Workers are long-lived, spawned once at server startup
- Each worker continuously monitors the job queue
- When a job is found, the worker claims it and processes it
- Processing includes 100ms simulation delay (can be extended for real work)
- Completed/failed jobs are available for status queries

## Performance Analysis

### Expected Results

On a modern multi-core system, typical results with default settings:

- **Throughput**: 100-500 requests/second (depends on network and TLS overhead)
- **Latency**: 5-50ms per request (mostly TLS handshake + HTTP overhead)
- **Success Rate**: 100% (with proper server configuration)

### Optimization Tips

1. **Increase Concurrency**: Higher concurrent requests can increase throughput

   ```bash
   STRESS_TEST_CONCURRENT=50 ./target/release/stress-test
   ```

2. **Use Release Build**: Release binaries are significantly faster

   ```bash
   cargo run --release --bin stress-test
   ```

3. **Monitor Server**: Watch server logs to ensure workers are processing

   ```bash
   ./dumptruck server ... -vvv  # Verbose output
   ```

4. **Check System Resources**:
   + Monitor CPU: `top`, `htop`
   + Monitor Memory: `free -h`
   + Monitor Network: `nethogs`, `iftop`

## Interpreting Results

### High Latency

- **TLS Handshake Overhead**: Each connection requires TLS 1.3 setup (~5-10ms)
- **Server Load**: If latency increases with more concurrent requests, server is CPU-bound
- **Network**: Check network latency to server with `ping`

### High Error Rate

- **Server Misconfiguration**: Check server logs for errors
- **Authentication Issues**: Verify OAuth token format and configuration
- **Server Overload**: Reduce concurrent requests or increase server resources

### Low Throughput

- **Single Threaded Bottleneck**: Ensure server is using parallel workers
- **I/O Bound**: Monitor disk usage if processing large files
- **TLS Overhead**: Consider connection pooling or HTTP/2 multiplexing benefits

## Integration with CI/CD

Use the stress test in your CI/CD pipeline:

```bash
# Build release binary
cargo build --bin stress-test --release

# Start server in background
./target/release/dumptruck server ... &
SERVER_PID=$!

# Give server time to start
sleep 2

# Run stress test
if STRESS_TEST_CONCURRENT=20 STRESS_TEST_REQUESTS=200 ./target/release/stress-test; then
  echo "✓ Stress test passed"
else
  echo "✗ Stress test failed"
  kill $SERVER_PID
  exit 1
fi

# Cleanup
kill $SERVER_PID
```

## Usage Examples

### Basic Single Fixture Test

Test server performance with a small credential dataset:

```bash
# Terminal 1: Start server
./target/debug/dumptruck server \
  --cert tests/fixtures/tls.crt \
  --key tests/fixtures/tls.key \
  --oauth-client-id test-client \
  --oauth-client-secret test-secret \
  --oauth-token-endpoint https://oauth.example.com/token

# Terminal 2: Run stress test with 5 concurrent requests
STRESS_TEST_CONCURRENT=5 STRESS_TEST_REQUESTS=50 ./target/release/stress-test
```

**Expected Output:**

```text
=== Dumptruck Server Stress Test ===
Server URL: https://localhost:8443
Concurrent Requests: 5
Total Requests Target: 50

Loaded 22 test fixtures
  - test_creds_small.csv (251 bytes)
  - json_credentials.json (450 bytes)
  - yaml_credentials.yaml (366 bytes)
  ...

=== Results ===
Total Requests: 50
Successful: 50 (100.0%)
Failed: 0 (0.0%)

Throughput: 123.45 requests/second
```

### Light Load Test

Test sustained performance with moderate concurrency:

```bash
STRESS_TEST_CONCURRENT=10 STRESS_TEST_REQUESTS=200 ./target/release/stress-test
```

Simulates typical production usage with 10 concurrent users submitting 200 total requests.

### Heavy Load Test

Push server to its limits to identify bottlenecks:

```bash
STRESS_TEST_CONCURRENT=50 STRESS_TEST_REQUESTS=2000 ./target/release/stress-test
```

Tests server behavior under high concurrent load (50 workers, 2000 requests).

### Verbose Testing with Custom URL

Debug request/response details for specific test scenario:

```bash
STRESS_TEST_URL=https://localhost:8443 \
  STRESS_TEST_CONCURRENT=5 \
  STRESS_TEST_REQUESTS=20 \
  STRESS_TEST_VERBOSE=1 \
  ./target/release/stress-test
```

**Sample Verbose Output:**

```text
[0] OK - test_creds_100.csv → job_id: 550e8400-e29b-41d4-a716-446655440000 (2.34ms)
[1] OK - json_credentials.json → job_id: 550e8400-e29b-41d4-a716-446655440001 (3.12ms)
[2] OK - yaml_credentials.yaml → job_id: 550e8400-e29b-41d4-a716-446655440002 (1.89ms)
[3] FAIL - malformed_missing_quote.csv - Server returned 400 (5.67ms)
[4] OK - well_formed_credentials.csv → job_id: 550e8400-e29b-41d4-a716-446655440003 (2.11ms)
```

### Testing Different Data Formats

Verify support for multiple input formats by running targeted tests:

#### CSV Format Test

```bash
# test_creds_small.csv contains: ID, credential columns
STRESS_TEST_REQUESTS=100 ./target/release/stress-test
# Results will show CSV fixture loading
```

#### JSON Format Test

```bash
# json_credentials.json contains: email, username, password objects
STRESS_TEST_REQUESTS=100 ./target/release/stress-test
# Results will show JSON fixture loading
```

#### YAML Format Test

```bash
# yaml_credentials.yaml contains: email, username, password entries
STRESS_TEST_REQUESTS=100 ./target/release/stress-test
# Results will show YAML fixture loading
```

### Error Handling Test

Test how the server handles malformed data:

```bash
STRESS_TEST_VERBOSE=1 STRESS_TEST_REQUESTS=50 ./target/release/stress-test
```

The stress test will attempt to upload fixtures including:

- `malformed_missing_quote.csv` - Unclosed quotation marks
- `malformed_mismatched_columns.csv` - Column count mismatches
- `empty_fields.csv` - Empty credential fields
- `missing_header.csv` - Missing header row

Monitor the output to see which fixtures fail and their error codes.

### Baseline Performance Comparison

Establish baseline performance before optimization:

```bash
# Baseline: Single concurrent request
time STRESS_TEST_CONCURRENT=1 STRESS_TEST_REQUESTS=100 ./target/release/stress-test

# Then test with increased concurrency
time STRESS_TEST_CONCURRENT=4 STRESS_TEST_REQUESTS=100 ./target/release/stress-test
time STRESS_TEST_CONCURRENT=8 STRESS_TEST_REQUESTS=100 ./target/release/stress-test
```

Compare latency and throughput metrics to identify optimal concurrency levels.

### Test Fixture Reference

| Fixture | Format | Content | Use Case |
|---------|--------|---------|----------|
| `test_creds_small.csv` | CSV | 10 email/password pairs | Quick smoke tests |
| `test_creds_100.csv` | CSV | 100 credential entries | Standard load testing |
| `test_creds_mixed.csv` | CSV | Mixed username/email formats | Format diversity testing |
| `json_credentials.json` | JSON | 8 user objects | JSON format validation |
| `yaml_credentials.yaml` | YAML | 5 user entries | YAML format validation |
| `unicode_addresses.csv` | CSV | International characters | Unicode handling |
| `special_characters.csv` | CSV | Symbols and escapes | Character encoding |
| `duplicate_rows.csv` | CSV | Repeated entries | Deduplication testing |
| `well_formed_credentials.csv` | CSV | Valid data | Success path testing |
| `malformed_missing_quote.csv` | CSV | Broken syntax | Error handling |
| `malformed_mismatched_columns.csv` | CSV | Wrong column count | Validation testing |

## Troubleshooting

### Connection Refused

```text
Request failed: error trying to connect
```

- Ensure server is running: `./dumptruck server ...`
- Check server port: default is `8443`
- Check firewall: `sudo ufw allow 8443/tcp`

### TLS Certificate Error

```text
error validating certificate
```

The stress test disables TLS certificate verification for testing purposes. If you see this error, check server certificate configuration.

### No Test Fixtures Found

```text
Loaded 0 test fixtures
```

- Ensure `tests/fixtures/` directory exists
- Verify test files are present: `ls tests/fixtures/`
- Use absolute paths if running from different directory

## Performance Metrics Reference

### Sample Output Interpretation

```text
Throughput: 234.5 requests/second
```

- Server can handle ~234 ingest requests per second under test conditions
- With 4-8 worker threads, this is expected for file ingestion

```text
Latency P99: 45.2 ms
```

- 99% of requests complete within 45.2ms
- Indicates consistent performance with occasional slower requests

```text
Success Rate: 99.2% (100 of 101 requests)
```

- One request failed, investigate logs for cause
- High success rate indicates stable server operation

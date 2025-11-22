//! Benchmarks for observation upload performance
//!
//! These benchmarks measure the upload speed and throughput for different
//! observation payload sizes and quantities.
//!
//! Run with: cargo bench --bench upload_benchmarks
//!
//! Metrics measured:
//! - Single small observation upload time (< 64KB)
//! - Single large observation upload time (> 64KB, uses blob storage)
//! - Max throughput of small observations
//! - Max throughput of large observations

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use observation_tools_client::{observe, Client, ClientBuilder, BLOB_THRESHOLD_BYTES};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::sleep;

/// Test server wrapper for benchmarks
struct BenchServer {
    addr: SocketAddr,
    _handle: tokio::task::JoinHandle<()>,
}

impl BenchServer {
    /// Start a new test server on a random port
    async fn new() -> Self {
        let data_dir = tempfile::tempdir().expect("Failed to create temp dir");

        // Bind to port 0 to get a random available port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");

        let addr = listener.local_addr().expect("Failed to get local address");

        let config = observation_tools_server::Config::new()
            .with_bind_addr(addr)
            .with_data_dir(data_dir.path().to_path_buf());

        let server = observation_tools_server::Server::new(config);

        let handle = tokio::spawn(async move {
            // Keep the tempdir alive for the duration of the server
            let _data_dir = data_dir;
            server.run(listener).await.expect("Server failed to run");
        });

        // Give the server a moment to start up
        sleep(Duration::from_millis(300)).await;

        Self {
            addr,
            _handle: handle,
        }
    }

    /// Create an observation tools client connected to this test server
    fn create_client(&self) -> anyhow::Result<Client> {
        let base_url = format!("http://{}", self.addr);
        Ok(ClientBuilder::new().base_url(&base_url).build()?)
    }
}

/// Benchmark uploading a single small observation (< 64KB)
fn bench_single_small_observation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = rt.block_on(BenchServer::new());
    let client = server.create_client().unwrap();

    // Create a small payload (1KB)
    let small_payload = "x".repeat(1024);
    let payload_size = small_payload.len();

    let mut group = c.benchmark_group("single_small_observation");
    group.throughput(Throughput::Bytes(payload_size as u64));
    group.measurement_time(Duration::from_secs(10));

    group.bench_function(BenchmarkId::new("1KB", payload_size), |b| {
        b.to_async(&rt).iter(|| {
            let client = client.clone();
            let small_payload = small_payload.clone();
            async move {
                let execution = client
                    .begin_execution("bench-execution")
                    .unwrap()
                    .wait_for_upload()
                    .await
                    .unwrap();

                observation_tools_client::with_execution(execution, async {
                    observe!(
                        name = "small-observation",
                        payload = small_payload
                    )
                    .unwrap()
                    .wait_for_upload()
                    .await
                    .unwrap();
                    Ok::<_, anyhow::Error>(())
                })
                .await
                .unwrap();
            }
        });
    });

    group.finish();
    rt.block_on(client.shutdown()).unwrap();
}

/// Benchmark uploading a single large observation (> 64KB, uses blob storage)
fn bench_single_large_observation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = rt.block_on(BenchServer::new());
    let client = server.create_client().unwrap();

    // Test various large payload sizes
    let sizes = vec![
        100_000,   // 100KB
        500_000,   // 500KB
        1_000_000, // 1MB
        5_000_000, // 5MB
    ];

    let mut group = c.benchmark_group("single_large_observation");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10); // Reduce sample size for large payloads

    for size in sizes {
        let large_payload = "x".repeat(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_function(BenchmarkId::from_parameter(format!("{}KB", size / 1024)), |b| {
            b.to_async(&rt).iter(|| {
                let client = client.clone();
                let large_payload = large_payload.clone();
                async move {
                    let execution = client
                        .begin_execution("bench-execution")
                        .unwrap()
                        .wait_for_upload()
                        .await
                        .unwrap();

                    observation_tools_client::with_execution(execution, async {
                        observe!(
                            name = "large-observation",
                            payload = large_payload
                        )
                        .unwrap()
                        .wait_for_upload()
                        .await
                        .unwrap();
                        Ok::<_, anyhow::Error>(())
                    })
                    .await
                    .unwrap();
                }
            });
        });
    }

    group.finish();
    rt.block_on(client.shutdown()).unwrap();
}

/// Benchmark max throughput of small observations
fn bench_throughput_small_observations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = rt.block_on(BenchServer::new());
    let client = server.create_client().unwrap();

    // Create a small payload (1KB) - well below the blob threshold
    let small_payload = "x".repeat(1024);
    let payload_size = small_payload.len();

    // Test different batch sizes
    let batch_sizes = vec![10, 50, 100, 500, 1000];

    let mut group = c.benchmark_group("throughput_small_observations");
    group.measurement_time(Duration::from_secs(15));

    for batch_size in batch_sizes {
        group.throughput(Throughput::Bytes((payload_size * batch_size) as u64));

        group.bench_function(
            BenchmarkId::new("batch", batch_size),
            |b| {
                b.to_async(&rt).iter(|| {
                    let client = client.clone();
                    let small_payload = small_payload.clone();
                    async move {
                        let execution = client
                            .begin_execution("bench-execution")
                            .unwrap()
                            .wait_for_upload()
                            .await
                            .unwrap();

                        observation_tools_client::with_execution(execution, async {
                            for i in 0..batch_size {
                                observe!(
                                    name = format!("observation-{}", i),
                                    payload = small_payload.clone()
                                )
                                .unwrap()
                                .wait_for_upload()
                                .await
                                .unwrap();
                            }
                            Ok::<_, anyhow::Error>(())
                        })
                        .await
                        .unwrap();
                    }
                });
            },
        );
    }

    group.finish();
    rt.block_on(client.shutdown()).unwrap();
}

/// Benchmark max throughput of large observations (uses blob storage)
fn bench_throughput_large_observations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = rt.block_on(BenchServer::new());
    let client = server.create_client().unwrap();

    // Create a large payload (100KB) - above the blob threshold (64KB)
    let large_payload = "x".repeat(100_000);
    let payload_size = large_payload.len();

    // Test different batch sizes (smaller than small observations due to size)
    let batch_sizes = vec![5, 10, 25, 50];

    let mut group = c.benchmark_group("throughput_large_observations");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10); // Reduce sample size for large payloads

    for batch_size in batch_sizes {
        group.throughput(Throughput::Bytes((payload_size * batch_size) as u64));

        group.bench_function(
            BenchmarkId::new("batch", batch_size),
            |b| {
                b.to_async(&rt).iter(|| {
                    let client = client.clone();
                    let large_payload = large_payload.clone();
                    async move {
                        let execution = client
                            .begin_execution("bench-execution")
                            .unwrap()
                            .wait_for_upload()
                            .await
                            .unwrap();

                        observation_tools_client::with_execution(execution, async {
                            for i in 0..batch_size {
                                observe!(
                                    name = format!("observation-{}", i),
                                    payload = large_payload.clone()
                                )
                                .unwrap()
                                .wait_for_upload()
                                .await
                                .unwrap();
                            }
                            Ok::<_, anyhow::Error>(())
                        })
                        .await
                        .unwrap();
                    }
                });
            },
        );
    }

    group.finish();
    rt.block_on(client.shutdown()).unwrap();
}

/// Benchmark comparing inline vs blob storage at the threshold boundary
fn bench_threshold_boundary(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let server = rt.block_on(BenchServer::new());
    let client = server.create_client().unwrap();

    let mut group = c.benchmark_group("threshold_boundary");
    group.measurement_time(Duration::from_secs(10));

    // Test just below threshold (inline storage)
    let below_threshold = "x".repeat(BLOB_THRESHOLD_BYTES - 1);
    group.throughput(Throughput::Bytes((BLOB_THRESHOLD_BYTES - 1) as u64));
    group.bench_function("below_threshold_63KB", |b| {
        b.to_async(&rt).iter(|| {
            let client = client.clone();
            let payload = below_threshold.clone();
            async move {
                let execution = client
                    .begin_execution("bench-execution")
                    .unwrap()
                    .wait_for_upload()
                    .await
                    .unwrap();

                observation_tools_client::with_execution(execution, async {
                    observe!(
                        name = "observation-below-threshold",
                        payload = payload
                    )
                    .unwrap()
                    .wait_for_upload()
                    .await
                    .unwrap();
                    Ok::<_, anyhow::Error>(())
                })
                .await
                .unwrap();
            }
        });
    });

    // Test at threshold (blob storage)
    let at_threshold = "x".repeat(BLOB_THRESHOLD_BYTES);
    group.throughput(Throughput::Bytes(BLOB_THRESHOLD_BYTES as u64));
    group.bench_function("at_threshold_64KB", |b| {
        b.to_async(&rt).iter(|| {
            let client = client.clone();
            let payload = at_threshold.clone();
            async move {
                let execution = client
                    .begin_execution("bench-execution")
                    .unwrap()
                    .wait_for_upload()
                    .await
                    .unwrap();

                observation_tools_client::with_execution(execution, async {
                    observe!(
                        name = "observation-at-threshold",
                        payload = payload
                    )
                    .unwrap()
                    .wait_for_upload()
                    .await
                    .unwrap();
                    Ok::<_, anyhow::Error>(())
                })
                .await
                .unwrap();
            }
        });
    });

    // Test just above threshold (blob storage)
    let above_threshold = "x".repeat(BLOB_THRESHOLD_BYTES + 1);
    group.throughput(Throughput::Bytes((BLOB_THRESHOLD_BYTES + 1) as u64));
    group.bench_function("above_threshold_65KB", |b| {
        b.to_async(&rt).iter(|| {
            let client = client.clone();
            let payload = above_threshold.clone();
            async move {
                let execution = client
                    .begin_execution("bench-execution")
                    .unwrap()
                    .wait_for_upload()
                    .await
                    .unwrap();

                observation_tools_client::with_execution(execution, async {
                    observe!(
                        name = "observation-above-threshold",
                        payload = payload
                    )
                    .unwrap()
                    .wait_for_upload()
                    .await
                    .unwrap();
                    Ok::<_, anyhow::Error>(())
                })
                .await
                .unwrap();
            }
        });
    });

    group.finish();
    rt.block_on(client.shutdown()).unwrap();
}

criterion_group!(
    benches,
    bench_single_small_observation,
    bench_single_large_observation,
    bench_throughput_small_observations,
    bench_throughput_large_observations,
    bench_threshold_boundary
);
criterion_main!(benches);

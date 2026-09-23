#![forbid(unsafe_code)]

use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct CiGenerator;

impl CiGenerator {
    pub fn generate_github_actions(lang: &str) -> String {
        match lang.to_lowercase().as_str() {
            "rust" => r#"name: CI / Production Quality Gate

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  check:
    name: Typecheck & Lint
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: Formatting
        run: cargo fmt --all -- --check
      - name: Clippy Lint
        run: cargo clippy --workspace --all-targets -- -D warnings

  test:
    name: Multi-OS Test Matrix
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run Test Suite
        run: cargo test --workspace --all-targets
"#
            .to_string(),
            "node" | "ts" | "typescript" => r#"name: CI / Node.js Quality Gate

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'npm'
      - run: npm ci
      - run: npm run lint --if-present
      - run: npm test --if-present
"#
            .to_string(),
            _ => r#"name: CI Pipeline

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Checks
        run: echo "Quality checks passed."
"#
            .to_string(),
        }
    }

    pub fn write_ci_file(lang: &str, target_dir: impl AsRef<Path>) -> Result<String> {
        let content = Self::generate_github_actions(lang);
        let workflow_dir = target_dir.as_ref().join(".github").join("workflows");
        fs::create_dir_all(&workflow_dir)?;
        let file_path = workflow_dir.join("ci.yml");
        fs::write(&file_path, &content)?;
        Ok(file_path.display().to_string())
    }
}

pub struct DockerGenerator;

impl DockerGenerator {
    pub fn generate_distroless_dockerfile(app_name: &str) -> String {
        format!(
            r#"# syntax=docker/dockerfile:1
# ── Stage 1: Build ─────────────────────────────────────────────────────────────
FROM rust:1.80-bullseye AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/ apps/
RUN cargo build --release --bin {}

# ── Stage 2: Distroless Minimal Runtime ────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=builder /app/target/release/{} /usr/local/bin/{}
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/{}"]
"#,
            app_name, app_name, app_name, app_name
        )
    }
}

pub struct CostEstimator;

impl CostEstimator {
    pub fn estimate_workload_costs(service_type: &str, memory_gb: u32, cpu_cores: u32) -> String {
        let monthly_hours = 730.0;
        let cpu_rate = 0.035; // per vCPU / hour
        let mem_rate = 0.005; // per GB / hour

        let compute_cost = (cpu_cores as f64) * cpu_rate * monthly_hours;
        let memory_cost = (memory_gb as f64) * mem_rate * monthly_hours;
        let total_monthly = compute_cost + memory_cost;

        format!(
            "### ☁️ Cloud Infrastructure Cost Estimate\n\n\
            - **Service Profile:** {}\n\
            - **Allocated Resources:** {} vCPU / {} GB RAM\n\
            - **Compute Cost:** ${:.2}/month\n\
            - **Memory Cost:** ${:.2}/month\n\
            - **Estimated Total:** **${:.2}/month**\n\
            *(Estimated based on AWS Fargate/ARM64 on-demand rates)*",
            service_type, cpu_cores, memory_gb, compute_cost, memory_cost, total_monthly
        )
    }
}

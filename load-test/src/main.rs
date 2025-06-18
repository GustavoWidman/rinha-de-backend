use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use reqwest::{Client, Url};
use rlt::{BenchSuite, IterInfo, IterReport, cli::BenchCli};
use tokio::time::Instant;

mod tests;
mod utils;

#[derive(Parser, Clone)]
pub struct HttpBench {
    /// Target URL.
    pub url: Url,

    #[arg(skip)]
    counter: u128,

    #[command(flatten)]
    pub bench_opts: BenchCli,
}

#[async_trait]
impl BenchSuite for HttpBench {
    type WorkerState = Client;

    async fn setup(&mut self, client: &mut Self::WorkerState, _: u32) -> Result<()> {
        client
            .post(self.url.join("reset")?)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn state(&self, _: u32) -> Result<Self::WorkerState> {
        Ok(Client::builder()
            .user_agent("rinha-load-test/1.0")
            .build()?)
    }

    async fn bench(&mut self, client: &mut Self::WorkerState, _: &IterInfo) -> Result<IterReport> {
        let (status, bytes, duration) = match self.counter % 34 {
            // Spread debito requests evenly: positions 0, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18, 20, 21, 23, 24, 26, 27, 29, 30, 32
            n if n % 2 == 0 && n != 22 && n != 33 => {
                tests::testar_debito(&self.url, client).await?
            }
            n if n % 3 == 1 && n != 22 && n != 33 => {
                tests::testar_debito(&self.url, client).await?
            }
            // Spread credito requests: positions 1, 4, 7, 10, 13, 16, 19, 25, 28, 31, 22
            n if (n % 3 == 1 && n != 22) || n == 22 || n % 6 == 4 || n % 9 == 7 => {
                tests::testar_credito(&self.url, client).await?
            }
            // 1 extrato request at position 33
            33 => tests::testar_extrato(&self.url, client).await?,
            // Default to debito for any remaining positions
            _ => tests::testar_debito(&self.url, client).await?,
        };

        if self.counter % 34 == 0 {
            // Reset balances every 34 iterations
            client
                .post(self.url.join("reset")?)
                .send()
                .await?
                .error_for_status()?;
        }

        self.counter += 1;
        Ok(IterReport {
            duration,
            status: status.into(),
            bytes,
            items: 1,
        })
    }

    async fn teardown(self, client: Self::WorkerState, _: IterInfo) -> Result<()> {
        client
            .post(self.url.join("reset")?)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let bs = HttpBench::parse();
    rlt::cli::run(bs.bench_opts, bs).await
}

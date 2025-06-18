use std::time::Duration;

use anyhow::{Result, anyhow};
use reqwest::{Client, StatusCode, Url};
use serde_json::Value;

use crate::utils;

fn validar_consistencia_saldo_limite(
    response: &Value,
    saldo_path: &str,
    limite_path: &str,
) -> Result<()> {
    let saldo = response
        .get(saldo_path)
        .ok_or(anyhow!("Saldo não encontrado na resposta"))?
        .as_i64()
        .ok_or(anyhow!(
            "Saldo {} não é um número válido",
            response.get(saldo_path).unwrap_or(&Value::Null).to_string()
        ))?;
    let limite = response
        .get(limite_path)
        .ok_or(anyhow!("Limite não encontrado na resposta"))?
        .as_i64()
        .ok_or(anyhow!(
            "Limite {} não é um número válido",
            response
                .get(limite_path)
                .unwrap_or(&Value::Null)
                .to_string()
        ))?;

    match (saldo, limite) {
        (s, l) if s < l * -1 => Err(anyhow!("Limite ultrapassado!")),
        (s, l) if s >= l * -1 => Ok(()),
        _ => Err(anyhow!("Erro ao validar saldo e limite")),
    }
}

pub async fn testar_debito(url: &Url, client: &Client) -> Result<(StatusCode, u64, Duration)> {
    let descricao = utils::random_description();
    let cliente_id = utils::random_client_id();
    let valor = utils::random_transaction_value();
    let payload = format!(
        r#"{{"valor": {}, "tipo": "d", "descricao": "{}"}}"#,
        valor, descricao
    );

    let t = tokio::time::Instant::now();
    let response = client
        .post(url.join(&format!("/clientes/{}/transacoes", cliente_id))?)
        .header("content-type", "application/json")
        .body(payload)
        .send()
        .await?;
    let elapsed = t.elapsed();

    let status = response.status();
    let bytes = response.content_length().unwrap_or(0) as u64;

    // status can be either 200 or 422. if 200, run validation
    if status == StatusCode::UNPROCESSABLE_ENTITY {
        return Ok((StatusCode::OK, bytes, elapsed));
    }

    if status == StatusCode::OK {
        validar_consistencia_saldo_limite(&response.json::<Value>().await?, "saldo", "limite")?;

        return Ok((status, bytes, elapsed));
    }

    Ok((status, bytes, elapsed))
}

pub async fn testar_credito(url: &Url, client: &Client) -> Result<(StatusCode, u64, Duration)> {
    let descricao = utils::random_description();
    let cliente_id = utils::random_client_id();
    let valor = utils::random_transaction_value();
    let payload = format!(
        r#"{{"valor": {}, "tipo": "c", "descricao": "{}"}}"#,
        valor, descricao
    );

    let t = tokio::time::Instant::now();
    let response = client
        .post(url.join(&format!("/clientes/{}/transacoes", cliente_id))?)
        .header("content-type", "application/json")
        .body(payload)
        .send()
        .await?;
    let elapsed = t.elapsed();

    let status = response.status();
    let bytes = response.content_length().unwrap_or(0) as u64;

    if status == StatusCode::OK {
        validar_consistencia_saldo_limite(&response.json::<Value>().await?, "saldo", "limite")?;

        return Ok((status, bytes, elapsed));
    }

    Err(anyhow!("Unexpected status code: {}", status))
}

pub async fn testar_extrato(url: &Url, client: &Client) -> Result<(StatusCode, u64, Duration)> {
    let t = tokio::time::Instant::now();
    let response = client
        .get(url.join(&format!("/clientes/{}/extrato", utils::random_client_id()))?)
        .send()
        .await?;
    let elapsed = t.elapsed();

    let status = response.status();
    let bytes = response.content_length().unwrap_or(0) as u64;

    if status == StatusCode::OK {
        validar_consistencia_saldo_limite(
            &response
                .json::<Value>()
                .await?
                .get("saldo")
                .unwrap_or(&Value::Null),
            "total",
            "limite",
        )?;

        return Ok((status, bytes, elapsed));
    }

    Err(anyhow!("Unexpected status code: {}", status))
}

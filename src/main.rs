use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result as ActixResult};
use chrono::{DateTime, Utc};
use deadpool_postgres::{Config, Pool, Runtime};
use log::error;
use serde::{Deserialize, Serialize};
use std::env;
use tokio_postgres::NoTls;

#[derive(Debug, Serialize, Deserialize)]
struct TransacaoRequest {
    valor: i32,
    tipo: String,
    descricao: String,
}

#[derive(Debug, Serialize)]
struct TransacaoResponse {
    limite: i32,
    saldo: i32,
}

#[derive(Debug, Serialize)]
struct SaldoInfo {
    total: i32,
    data_extrato: DateTime<Utc>,
    limite: i32,
}

#[derive(Debug, Serialize)]
struct TransacaoInfo {
    valor: i32,
    tipo: String,
    descricao: String,
    realizada_em: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct ExtratoResponse {
    saldo: SaldoInfo,
    ultimas_transacoes: Vec<TransacaoInfo>,
}

async fn criar_transacao(
    path: web::Path<i32>,
    payload: web::Json<TransacaoRequest>,
    pool: web::Data<Pool>,
) -> ActixResult<HttpResponse> {
    let cliente_id = path.into_inner();
    let transacao = payload.into_inner();

    // Validações básicas primeiro
    if transacao.tipo != "c" && transacao.tipo != "d" {
        return Ok(HttpResponse::UnprocessableEntity().json("Tipo inválido"));
    }

    if transacao.descricao.is_empty() || transacao.descricao.len() > 10 {
        return Ok(HttpResponse::UnprocessableEntity().json("Descrição inválida"));
    }

    if transacao.valor <= 0 {
        return Ok(HttpResponse::UnprocessableEntity().json("Valor inválido"));
    }

    // Validar se cliente_id está na faixa válida (1-5)
    if cliente_id < 1 || cliente_id > 5 {
        return Ok(HttpResponse::NotFound().json("Cliente não encontrado"));
    }

    let mut client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            error!(
                "Erro ao obter conexão do pool para cliente {}: {:?}",
                cliente_id, e
            );
            return Ok(HttpResponse::InternalServerError().json("Erro de conexão"));
        }
    };

    // Usar transação atômica para evitar race conditions
    let tx = match client.transaction().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(
                "Erro ao iniciar transação para cliente {}: {:?}",
                cliente_id, e
            );
            return Ok(HttpResponse::InternalServerError().json("Erro na transação"));
        }
    };

    // Usar SELECT FOR UPDATE para bloquear a linha durante a atualização
    let row = match tx
        .query_one(
            "SELECT id, limite, saldo FROM clientes WHERE id = $1 FOR UPDATE",
            &[&cliente_id],
        )
        .await
    {
        Ok(row) => row,
        Err(e) => {
            error!(
                "Erro ao buscar cliente {} com SELECT FOR UPDATE: {:?}",
                cliente_id, e
            );
            return Ok(HttpResponse::NotFound().json("Cliente não encontrado"));
        }
    };

    let limite: i32 = row.get("limite");
    let saldo_atual: i32 = row.get("saldo");

    // Calcular novo saldo
    let novo_saldo = if transacao.tipo == "c" {
        saldo_atual + transacao.valor
    } else {
        saldo_atual - transacao.valor
    };

    // Verificar se débito não ultrapassa limite
    if transacao.tipo == "d" && novo_saldo < -limite {
        // Rollback automático quando tx sai de escopo
        return Ok(HttpResponse::UnprocessableEntity().json("Saldo insuficiente"));
    }

    // Atualizar saldo do cliente
    if let Err(e) = tx
        .execute(
            "UPDATE clientes SET saldo = $1 WHERE id = $2",
            &[&novo_saldo, &cliente_id],
        )
        .await
    {
        error!("Erro ao atualizar saldo cliente {}: {:?}", cliente_id, e);
        return Ok(HttpResponse::InternalServerError().json("Erro ao atualizar saldo"));
    }

    // Inserir transação
    let now = chrono::Utc::now();
    if let Err(e) = tx
        .execute(
            "INSERT INTO transacoes (cliente_id, valor, tipo, descricao, realizada_em) VALUES ($1, $2, $3, $4, $5)",
            &[&cliente_id, &transacao.valor, &transacao.tipo, &transacao.descricao, &now],
        )
        .await
    {
        error!("Erro ao inserir transação cliente {}: {:?}", cliente_id, e);
        return Ok(HttpResponse::InternalServerError().json("Erro ao inserir transação"));
    }

    // Commit da transação
    if let Err(e) = tx.commit().await {
        error!(
            "Erro ao confirmar transação cliente {}: {:?}",
            cliente_id, e
        );
        return Ok(HttpResponse::InternalServerError().json("Erro ao confirmar transação"));
    }

    let response = TransacaoResponse {
        limite,
        saldo: novo_saldo,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn obter_extrato(path: web::Path<i32>, pool: web::Data<Pool>) -> ActixResult<HttpResponse> {
    let cliente_id = path.into_inner();

    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            error!(
                "Erro ao obter conexão do pool para extrato cliente {}: {:?}",
                cliente_id, e
            );
            return Ok(HttpResponse::InternalServerError().json("Erro de conexão"));
        }
    };

    // Verificar se cliente existe e buscar dados atuais
    let row = match client
        .query_one(
            "SELECT id, limite, saldo FROM clientes WHERE id = $1",
            &[&cliente_id],
        )
        .await
    {
        Ok(row) => row,
        Err(e) => {
            error!(
                "Erro ao buscar cliente {} para extrato: {:?}",
                cliente_id, e
            );
            return Ok(HttpResponse::NotFound().json("Cliente não encontrado"));
        }
    };

    let limite: i32 = row.get("limite");
    let saldo_atual: i32 = row.get("saldo");

    // Buscar últimas 10 transações
    let rows = match client
        .query(
            "SELECT valor, tipo, descricao, realizada_em FROM transacoes
             WHERE cliente_id = $1 ORDER BY realizada_em DESC LIMIT 10",
            &[&cliente_id],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!("Erro ao buscar transações cliente {}: {:?}", cliente_id, e);
            return Ok(HttpResponse::InternalServerError().json("Erro ao buscar transações"));
        }
    };

    let ultimas_transacoes: Vec<TransacaoInfo> = rows
        .iter()
        .map(|row| {
            let realizada_em: DateTime<Utc> = row.get("realizada_em");
            TransacaoInfo {
                valor: row.get("valor"),
                tipo: row.get("tipo"),
                descricao: row.get("descricao"),
                realizada_em,
            }
        })
        .collect();

    let response = ExtratoResponse {
        saldo: SaldoInfo {
            total: saldo_atual,
            data_extrato: Utc::now(),
            limite,
        },
        ultimas_transacoes,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn reset_client_balances(pool: web::Data<Pool>) -> ActixResult<HttpResponse> {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            error!("Erro ao obter conexão do pool: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json("Erro de conexão"));
        }
    };

    // Setar o saldo dos clientes para como deveriam ser inicialmente
    let reset_query = "
        UPDATE clientes
        SET saldo = CASE id
            WHEN 1 THEN 0
            WHEN 2 THEN 0
            WHEN 3 THEN 0
            WHEN 4 THEN 0
            WHEN 5 THEN 0
        END,
        limite = CASE id
            WHEN 1 THEN 100000
            WHEN 2 THEN 80000
            WHEN 3 THEN 1000000
            WHEN 4 THEN 10000000
            WHEN 5 THEN 500000
        END
        WHERE id IN (1, 2, 3, 4, 5)";
    if let Err(e) = client.execute(reset_query, &[]).await {
        error!("Erro ao resetar saldos dos clientes: {:?}", e);
        return Ok(HttpResponse::InternalServerError().json("Erro ao resetar saldos"));
    }

    Ok(HttpResponse::Ok().json("Saldos dos clientes resetados com sucesso"))
}

async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json("OK"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:123@db:5432/rinha".to_string());

    let mut cfg = Config::new();
    cfg.url = Some(database_url);

    // Configurar pool para alta concorrência
    cfg.pool = Some(deadpool_postgres::PoolConfig {
        max_size: 20, // Máximo de 20 conexões no pool
        ..Default::default()
    });

    let pool = match cfg.create_pool(Some(Runtime::Tokio1), NoTls) {
        Ok(pool) => pool,
        Err(e) => {
            error!("Falha ao criar pool de conexões: {:?}", e);
            std::process::exit(1);
        }
    };

    println!("Servidor iniciando na porta 8080...");
    println!("Pool de conexões configurado com max_size: 20");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .route("/health", web::get().to(health_check))
            .route("/clientes/{id}/transacoes", web::post().to(criar_transacao))
            .route("/clientes/{id}/extrato", web::get().to(obter_extrato))
            .route("/reset", web::post().to(reset_client_balances))
    })
    .bind("0.0.0.0:8080")?
    .workers(4)
    .run()
    .await
}

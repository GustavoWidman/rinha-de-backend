use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result as ActixResult};
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::env;

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
    pool: web::Data<Pool<Sqlite>>,
) -> ActixResult<HttpResponse> {
    let cliente_id = path.into_inner();
    let transacao = payload.into_inner();

    if transacao.valor <= 0 {
        return Ok(HttpResponse::UnprocessableEntity().finish());
    }
    if transacao.descricao.is_empty() || transacao.descricao.len() > 10 {
        return Ok(HttpResponse::UnprocessableEntity().finish());
    }
    if cliente_id < 1 || cliente_id > 5 {
        return Ok(HttpResponse::NotFound().finish());
    }

    let valor_a_aplicar = if transacao.tipo == "c" {
        transacao.valor
    } else if transacao.tipo == "d" {
        -transacao.valor
    } else {
        return Ok(HttpResponse::UnprocessableEntity().finish());
    };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return Ok(HttpResponse::ServiceUnavailable().finish()), // Or another appropriate error
    };

    let sql = "
        UPDATE clientes
        SET saldo = saldo + $1
        WHERE id = $2 AND saldo + $1 >= -limite
        RETURNING limite, saldo as novo_saldo
    ";
    let result: Option<(i32, i32)> = match sqlx::query_as(sql)
        .bind(valor_a_aplicar)
        .bind(cliente_id)
        .fetch_optional(&mut *tx)
        .await
    {
        Ok(res) => res,
        Err(e) => {
            error!(
                "DB error during update-check for client {}: {:?}",
                cliente_id, e
            );
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };

    let (limite, novo_saldo) = match result {
        Some((limite, novo_saldo)) => (limite, novo_saldo),
        None => {
            return Ok(HttpResponse::UnprocessableEntity().finish());
        }
    };

    let now = Utc::now();
    if let Err(e) = sqlx::query(
        "INSERT INTO transacoes (cliente_id, valor, tipo, descricao, realizada_em) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(cliente_id)
    .bind(transacao.valor)
    .bind(&transacao.tipo)
    .bind(&transacao.descricao)
    .bind(now.to_rfc3339())
    .execute(&mut *tx)
    .await
    {
        error!("DB error inserting transaction for client {}: {:?}", cliente_id, e);
        return Ok(HttpResponse::InternalServerError().finish());
    }

    if let Err(e) = tx.commit().await {
        error!(
            "DB error committing transaction for client {}: {:?}",
            cliente_id, e
        );
        return Ok(HttpResponse::InternalServerError().finish());
    }

    let response = TransacaoResponse {
        limite,
        saldo: novo_saldo,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn obter_extrato(
    path: web::Path<i32>,
    pool: web::Data<Pool<Sqlite>>,
) -> ActixResult<HttpResponse> {
    let cliente_id = path.into_inner();

    // Validar se cliente_id está na faixa válida (1-5)
    if cliente_id < 1 || cliente_id > 5 {
        return Ok(HttpResponse::NotFound().json("Cliente não encontrado"));
    }

    // Verificar se cliente existe e buscar dados atuais
    let row = match sqlx::query("SELECT id, limite, saldo FROM clientes WHERE id = ?")
        .bind(cliente_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(row) => row,
        Err(e) => {
            error!(
                "ERROR Erro ao buscar cliente {} para extrato: {:?}",
                cliente_id, e
            );
            return Ok(HttpResponse::NotFound().json("Cliente não encontrado"));
        }
    };

    let limite: i32 = row.get("limite");
    let saldo_atual: i32 = row.get("saldo");

    // Buscar últimas 10 transações
    let rows = match sqlx::query(
        "SELECT valor, tipo, descricao, realizada_em FROM transacoes
         WHERE cliente_id = ? ORDER BY realizada_em DESC LIMIT 10",
    )
    .bind(cliente_id)
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!(
                "ERROR Erro ao buscar transações cliente {}: {:?}",
                cliente_id, e
            );
            return Ok(HttpResponse::InternalServerError().json("Erro ao buscar transações"));
        }
    };

    let ultimas_transacoes: Vec<TransacaoInfo> = rows
        .iter()
        .map(|row| {
            let realizada_em_str: String = row.get("realizada_em");
            let realizada_em = DateTime::parse_from_rfc3339(&realizada_em_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

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

async fn reset_client_balances(pool: web::Data<Pool<Sqlite>>) -> ActixResult<HttpResponse> {
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
    if let Err(e) = sqlx::query(reset_query).execute(pool.get_ref()).await {
        error!("ERROR Erro ao resetar saldos dos clientes: {:?}", e);
        return Ok(HttpResponse::InternalServerError().json("Erro ao resetar saldos"));
    }

    Ok(HttpResponse::Ok().json("Saldos dos clientes resetados com sucesso"))
}

async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json("OK"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:///shared/rinha.db".to_string());

    // Extrair o caminho do arquivo SQLite e garantir que o diretório existe
    let db_path = database_url
        .strip_prefix("sqlite://")
        .unwrap_or(&database_url);
    if let Some(parent_dir) = std::path::Path::new(db_path).parent() {
        if let Err(e) = std::fs::create_dir_all(parent_dir) {
            error!("ERROR Erro ao criar diretório do banco: {:?}", e);
            std::process::exit(1);
        }
        println!("Diretório do banco assegurado: {:?}", parent_dir);
    }

    // Configurar SQLite pool com WAL mode e otimizações para concorrência
    let pool = match sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(14)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                .busy_timeout(std::time::Duration::from_millis(5000)) // 5 seconds timeout for locks
                .pragma("cache_size", "1000000")
                .pragma("foreign_keys", "true")
                .pragma("temp_store", "memory")
                .pragma("wal_autocheckpoint", "1000") // Checkpoint WAL after 1000 pages
                .pragma("journal_size_limit", "67108864") // 64MB WAL limit
                .pragma("mmap_size", "268435456"), // 256MB memory-mapped I/O
        )
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            error!("ERROR Falha ao conectar ao SQLite: {:?}", e);
            std::process::exit(1);
        }
    };

    // Executar schema
    if let Err(e) = sqlx::query(include_str!("../script.sql"))
        .execute(&pool)
        .await
    {
        error!("Erro ao executar schema: {:?}", e);
        std::process::exit(1);
    }

    println!("Servidor iniciando na porta 8080...");
    println!("SQLite configurado com WAL mode para alta performance");

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
    .workers(14)
    .run()
    .await
}

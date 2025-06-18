PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = 1000000;
PRAGMA foreign_keys = true;
PRAGMA temp_store = memory;

CREATE TABLE IF NOT EXISTS clientes (
    id INTEGER PRIMARY KEY,
    nome TEXT NOT NULL,
    limite INTEGER NOT NULL,
    saldo INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS transacoes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cliente_id INTEGER NOT NULL,
    valor INTEGER NOT NULL,
    tipo TEXT NOT NULL CHECK (tipo IN ('c', 'd')),
    descricao TEXT NOT NULL,
    realizada_em TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (cliente_id) REFERENCES clientes(id)
);

CREATE INDEX IF NOT EXISTS idx_transacoes_cliente_id_realizada_em
ON transacoes(cliente_id, realizada_em DESC);
CREATE INDEX IF NOT EXISTS idx_clientes_id ON clientes(id);

INSERT OR REPLACE INTO clientes (id, nome, limite, saldo) VALUES
    (1, 'Cliente 1', 100000, 0),
    (2, 'Cliente 2', 80000, 0),
    (3, 'Cliente 3', 1000000, 0),
    (4, 'Cliente 4', 10000000, 0),
    (5, 'Cliente 5', 500000, 0);

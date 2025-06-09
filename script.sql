-- Script de inicialização do banco de dados
CREATE TABLE IF NOT EXISTS clientes (
    id SERIAL PRIMARY KEY,
    nome VARCHAR(255) NOT NULL,
    limite INTEGER NOT NULL,
    saldo INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS transacoes (
    id SERIAL PRIMARY KEY,
    cliente_id INTEGER NOT NULL,
    valor INTEGER NOT NULL,
    tipo CHAR(1) NOT NULL CHECK (tipo IN ('c', 'd')),
    descricao VARCHAR(10) NOT NULL,
    realizada_em TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    FOREIGN KEY (cliente_id) REFERENCES clientes(id)
);

-- Índices para otimização de performance
CREATE INDEX IF NOT EXISTS idx_transacoes_cliente_id_realizada_em 
ON transacoes(cliente_id, realizada_em DESC);

CREATE INDEX IF NOT EXISTS idx_clientes_id ON clientes(id);

-- Inserir os 5 clientes obrigatórios
INSERT INTO clientes (id, nome, limite, saldo) VALUES 
    (1, 'Cliente 1', 100000, 0),
    (2, 'Cliente 2', 80000, 0),
    (3, 'Cliente 3', 1000000, 0),
    (4, 'Cliente 4', 10000000, 0),
    (5, 'Cliente 5', 500000, 0)
ON CONFLICT (id) DO NOTHING;

-- Resetar sequência para garantir IDs corretos
SELECT setval('clientes_id_seq', 5, true);

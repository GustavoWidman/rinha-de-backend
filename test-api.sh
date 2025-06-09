#!/bin/bash

# Teste básico dos endpoints da API
set -e

BASE_URL="http://localhost:9999"

echo "🧪 Testando API Rinha de Backend..."
echo "==============================================="

# Aguardar serviços iniciarem
echo "⏳ Aguardando serviços..."
sleep 5

# Teste 1: Health check
echo "1️⃣ Testando health check..."
curl -s "$BASE_URL/health" > /dev/null && echo "✅ Health check OK" || echo "❌ Health check falhou"

# Teste 2: Criar uma transação de crédito
echo "2️⃣ Testando transação de crédito..."
RESPONSE=$(curl -s -X POST "$BASE_URL/clientes/1/transacoes" \
  -H "Content-Type: application/json" \
  -d '{"valor": 1000, "tipo": "c", "descricao": "deposito"}')
echo "Resposta: $RESPONSE"

# Teste 3: Criar uma transação de débito
echo "3️⃣ Testando transação de débito..."
RESPONSE=$(curl -s -X POST "$BASE_URL/clientes/1/transacoes" \
  -H "Content-Type: application/json" \
  -d '{"valor": 100, "tipo": "d", "descricao": "saque"}')
echo "Resposta: $RESPONSE"

# Teste 4: Consultar extrato
echo "4️⃣ Testando consulta de extrato..."
RESPONSE=$(curl -s "$BASE_URL/clientes/1/extrato")
echo "Resposta: $RESPONSE"

# Teste 5: Cliente não existente (deve retornar 404)
echo "5️⃣ Testando cliente inexistente..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/clientes/6/extrato")
if [ "$HTTP_CODE" = "404" ]; then
    echo "✅ Cliente inexistente retornou 404 corretamente"
else
    echo "❌ Cliente inexistente retornou código $HTTP_CODE (esperado 404)"
fi

# Teste 6: Validação de dados inválidos
echo "6️⃣ Testando validação de dados..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/clientes/1/transacoes" \
  -H "Content-Type: application/json" \
  -d '{"valor": 1000, "tipo": "x", "descricao": "invalido"}')
if [ "$HTTP_CODE" = "422" ]; then
    echo "✅ Dados inválidos retornaram 422 corretamente"
else
    echo "❌ Dados inválidos retornaram código $HTTP_CODE (esperado 422)"
fi

echo "==============================================="
echo "🎯 Testes básicos concluídos!"

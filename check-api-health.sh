#!/bin/bash

# Simple API readiness check script
# This script validates that the Rinha de Backend API is ready for load testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

API_BASE_URL="http://localhost:9999"

print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[ℹ]${NC} $1"
}

echo "🔍 Checking API readiness..."
echo ""

# Check nginx load balancer status
print_info "Checking nginx load balancer..."
if curl -s --max-time 3 "$API_BASE_URL/nginx_status" > /dev/null 2>&1; then
    print_status "Nginx load balancer is running"
else
    print_info "Nginx status endpoint not accessible (normal if using different LB)"
fi

# Check if API is responding
print_info "Testing API connectivity..."
if curl -s --max-time 5 "$API_BASE_URL/clientes/1/extrato" > /dev/null 2>&1; then
    print_status "API is responding at $API_BASE_URL"
else
    print_error "API is not responding at $API_BASE_URL"
    echo "   💡 Try running: ./run.sh"
    exit 1
fi

# Test all 5 clients
print_info "Testing all client endpoints..."
for client_id in {1..5}; do
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_BASE_URL/clientes/$client_id/extrato")
    if [ "$HTTP_CODE" = "200" ]; then
        print_status "Client $client_id: OK"
    else
        print_error "Client $client_id: HTTP $HTTP_CODE"
        exit 1
    fi
done

# Test transaction endpoint
print_info "Testing transaction creation..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"valor": 1, "tipo": "c", "descricao": "test"}' \
    "$API_BASE_URL/clientes/1/transacoes")

if [ "$HTTP_CODE" = "200" ]; then
    print_status "Transaction endpoint: OK"
else
    print_error "Transaction endpoint: HTTP $HTTP_CODE"
    exit 1
fi

# Test 404 for non-existent client
print_info "Testing error handling..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_BASE_URL/clientes/999/extrato")
if [ "$HTTP_CODE" = "404" ]; then
    print_status "404 error handling: OK"
else
    print_error "404 error handling: Expected 404, got HTTP $HTTP_CODE"
    exit 1
fi

# Test 422 for invalid data
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"valor": "invalid", "tipo": "c", "descricao": "test"}' \
    "$API_BASE_URL/clientes/1/transacoes")

if [ "$HTTP_CODE" = "422" ] || [ "$HTTP_CODE" = "400" ]; then
    print_status "422/400 error handling: OK"
else
    print_error "422/400 error handling: Expected 422 or 400, got HTTP $HTTP_CODE"
    exit 1
fi

echo ""
echo "🎉 API is ready for load testing!"
echo "   Run: ./executar-teste-local.sh"

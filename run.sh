#!/bin/bash

# Script para facilitar desenvolvimento e testes

set -e

case "$1" in
    "build")
        echo "🔨 Building application..."
        docker-compose build
        ;;
    "up")
        echo "🚀 Starting services..."
        docker-compose up -d
        echo "✅ Services started! API available at http://localhost:9999"
        ;;
    "down")
        echo "🛑 Stopping services..."
        docker-compose down
        ;;
    "logs")
        echo "📋 Showing logs..."
        docker-compose logs -f
        ;;
    "test")
        echo "🧪 Running basic tests..."
        
        # Wait for services to be ready
        echo "Waiting for services to start..."
        sleep 10
        
        # Test health endpoint
        echo "Testing health endpoint..."
        curl -f http://localhost:9999/health || (echo "❌ Health check failed" && exit 1)
        
        # Test transaction creation
        echo "Testing transaction creation..."
        curl -X POST http://localhost:9999/clientes/1/transacoes \
             -H "Content-Type: application/json" \
             -d '{"valor": 1000, "tipo": "c", "descricao": "teste"}' || (echo "❌ Transaction test failed" && exit 1)
        
        # Test account statement
        echo "Testing account statement..."
        curl -f http://localhost:9999/clientes/1/extrato || (echo "❌ Statement test failed" && exit 1)
        
        # Test non-existent client
        echo "Testing non-existent client..."
        curl -f http://localhost:9999/clientes/6/extrato && (echo "❌ Should return 404" && exit 1) || echo "✅ 404 test passed"
        
        echo "✅ All basic tests passed!"
        ;;
    "clean")
        echo "🧹 Cleaning up..."
        docker-compose down -v
        docker system prune -f
        ;;
    *)
        echo "Usage: $0 {build|up|down|logs|test|clean}"
        echo ""
        echo "Commands:"
        echo "  build  - Build the application"
        echo "  up     - Start all services"
        echo "  down   - Stop all services"
        echo "  logs   - Show service logs"
        echo "  test   - Run basic API tests"
        echo "  clean  - Clean up containers and volumes"
        exit 1
        ;;
esac

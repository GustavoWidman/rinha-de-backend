#!/bin/bash

# Pre-test validation script for Rinha de Backend 2024 Q1
# This script ensures all components are ready for load testing

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}=================================================${NC}"
    echo -e "${BLUE}     Rinha de Backend - Pre-Test Validation     ${NC}"
    echo -e "${BLUE}=================================================${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}🔍 $1${NC}"
}

# Main validation function
validate_environment() {
    local errors=0

    print_header

    # Check Docker containers
    print_info "Checking Docker containers..."
    if docker-compose ps | grep -q "Up"; then
        print_success "Docker containers are running"

        # Check individual services
        if docker-compose ps | grep "api01" | grep -q "Up"; then
            print_success "API Instance 1: Running"
        else
            print_error "API Instance 1: Not running"
            ((errors++))
        fi

        if docker-compose ps | grep "api02" | grep -q "Up"; then
            print_success "API Instance 2: Running"
        else
            print_error "API Instance 2: Not running"
            ((errors++))
        fi

        if docker-compose ps | grep "nginx" | grep -q "Up"; then
            print_success "Load Balancer (nginx): Running"
        else
            print_error "Load Balancer (nginx): Not running"
            ((errors++))
        fi

        if docker-compose ps | grep "db" | grep -q "Up"; then
            print_success "Database (PostgreSQL): Running"
        else
            print_error "Database (PostgreSQL): Not running"
            ((errors++))
        fi
    else
        print_error "Docker containers are not running"
        print_info "Run: ./run.sh to start the services"
        ((errors++))
    fi

    echo ""

    # Check API endpoints
    print_info "Testing API endpoints..."

    # Test load balancer
    if curl -s --max-time 5 "http://localhost:9999/clientes/1/extrato" > /dev/null; then
        print_success "Load balancer responding on port 9999"
    else
        print_error "Load balancer not responding on port 9999"
        ((errors++))
    fi

    # Test all clients
    for client_id in {1..5}; do
        if HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:9999/clientes/$client_id/extrato"); then
            if [ "$HTTP_CODE" = "200" ]; then
                print_success "Client $client_id: Available (HTTP 200)"
            else
                print_error "Client $client_id: HTTP $HTTP_CODE"
                ((errors++))
            fi
        else
            print_error "Client $client_id: Connection failed"
            ((errors++))
        fi
    done

    # Test transaction endpoint
    if HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"valor": 1, "tipo": "c", "descricao": "validation"}' \
        "http://localhost:9999/clientes/1/transacoes"); then
        if [ "$HTTP_CODE" = "200" ]; then
            print_success "Transaction endpoint: Working (HTTP 200)"
        else
            print_error "Transaction endpoint: HTTP $HTTP_CODE"
            ((errors++))
        fi
    else
        print_error "Transaction endpoint: Connection failed"
        ((errors++))
    fi

    # Test error handling
    if HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:9999/clientes/999/extrato"); then
        if [ "$HTTP_CODE" = "404" ]; then
            print_success "404 Error handling: Working"
        else
            print_error "404 Error handling: Expected 404, got HTTP $HTTP_CODE"
            ((errors++))
        fi
    fi

    echo ""

    # Check Gatling prerequisites
    print_info "Checking Gatling prerequisites..."

    if command -v java &> /dev/null; then
        JAVA_VERSION=$(java -version 2>&1 | head -n 1 | cut -d'"' -f2)
        print_success "Java: $JAVA_VERSION"
    else
        print_error "Java: Not installed"
        ((errors++))
    fi

    if [ -n "$GATLING_HOME" ] && [ -d "$GATLING_HOME" ]; then
        print_success "GATLING_HOME: $GATLING_HOME"

        if [ -f "$GATLING_HOME/bin/gatling.sh" ]; then
            print_success "Gatling executable: Found"
        else
            print_error "Gatling executable: Not found at $GATLING_HOME/bin/gatling.sh"
            ((errors++))
        fi
    else
        print_error "GATLING_HOME: Not set or invalid"
        print_info "Set with: export GATLING_HOME=/path/to/gatling"
        ((errors++))
    fi

    # Check load test files
    if [ -f "load-test/user-files/simulations/rinhabackend/RinhaBackendCrebitosSimulation.scala" ]; then
        print_success "Gatling simulation: Found"
    else
        print_error "Gatling simulation: Not found"
        ((errors++))
    fi

    echo ""

    # Final result
    if [ $errors -eq 0 ]; then
        echo -e "${GREEN}🎉 All validations passed! Ready for load testing.${NC}"
        echo -e "${GREEN}   Run: ./executar-teste-local.sh${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}⚠️  $errors validation error(s) found.${NC}"
        echo -e "${RED}   Fix the issues above before running load tests.${NC}"
        echo ""
        return 1
    fi
}

# Performance check
quick_performance_check() {
    print_info "Running quick performance check..."

    echo "Testing sustained load for 10 seconds..."

    # Simple stress test using curl
    for i in {1..10}; do
        curl -s "http://localhost:9999/clientes/1/extrato" > /dev/null &
        curl -s -X POST -H "Content-Type: application/json" \
             -d '{"valor": 1, "tipo": "c", "descricao": "stress"}' \
             "http://localhost:9999/clientes/2/transacoes" > /dev/null &
    done

    wait
    print_success "Quick stress test completed"
}

# Main execution
if validate_environment; then
    if [ "$1" = "--quick-perf" ]; then
        quick_performance_check
    fi
else
    exit 1
fi

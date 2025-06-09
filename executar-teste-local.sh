#!/bin/bash

# Rinha de Backend 2024 Q1 - Gatling Load Test Execution Script
# This script runs the official Gatling simulation for the Rinha de Backend challenge

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_BASE_URL="http://localhost:9999"
SIMULATION_CLASS="rinhabackend.RinhaBackendCrebitosSimulation"
LOAD_TEST_DIR="$(pwd)/load-test"
USER_FILES_DIR="$LOAD_TEST_DIR/user-files"
RESULTS_DIR="$USER_FILES_DIR/results"

print_header() {
    echo -e "${BLUE}=================================================${NC}"
    echo -e "${BLUE}  Rinha de Backend 2024 Q1 - Load Test Runner  ${NC}"
    echo -e "${BLUE}=================================================${NC}"
    echo ""
}

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_java() {
    print_status "Checking Java installation..."

    if ! command -v java &> /dev/null; then
        print_error "Java is not installed or not in PATH"
        print_error "Please install Java 8+ and ensure it's accessible via 'java' command"
        exit 1
    fi

    JAVA_VERSION=$(java -version 2>&1 | head -n 1 | cut -d'"' -f2)
    print_status "Found Java version: $JAVA_VERSION"

    # Check if Java version is 8 or higher
    MAJOR_VERSION=$(echo $JAVA_VERSION | cut -d'.' -f1)
    if [ "$MAJOR_VERSION" = "1" ]; then
        MAJOR_VERSION=$(echo $JAVA_VERSION | cut -d'.' -f2)
    fi

    if [ "$MAJOR_VERSION" -lt 8 ]; then
        print_error "Java 8 or higher is required. Found version: $JAVA_VERSION"
        exit 1
    fi
}

check_gatling() {
    print_status "Checking Gatling installation..."

    if [ -z "$GATLING_HOME" ]; then
        print_error "GATLING_HOME environment variable is not set"
        print_error "Please download Gatling from https://gatling.io/download/"
        print_error "Extract it and set GATLING_HOME to the installation directory"
        print_error "Example: export GATLING_HOME=/path/to/gatling-charts-highcharts-bundle-3.9.5"
        exit 1
    fi

    if [ ! -d "$GATLING_HOME" ]; then
        print_error "GATLING_HOME directory does not exist: $GATLING_HOME"
        exit 1
    fi

    GATLING_BIN="$GATLING_HOME/bin/gatling.sh"
    if [ ! -f "$GATLING_BIN" ]; then
        print_error "Gatling executable not found: $GATLING_BIN"
        print_error "Please ensure GATLING_HOME points to a valid Gatling installation"
        exit 1
    fi

    print_status "Found Gatling installation at: $GATLING_HOME"
}

check_api_health() {
    print_status "Checking API health..."

    # Check if API is responding
    if ! curl -s --max-time 5 "$API_BASE_URL/clientes/1/extrato" > /dev/null 2>&1; then
        print_error "API is not responding at $API_BASE_URL"
        print_error "Please ensure the API is running with: ./run.sh"
        print_error "You can test the API manually with: ./test-api.sh"
        exit 1
    fi

    print_status "API is responding at $API_BASE_URL"

    # Test a few endpoints to ensure they're working properly
    print_status "Testing API endpoints..."

    # Test extrato endpoint
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_BASE_URL/clientes/1/extrato")
    if [ "$HTTP_CODE" != "200" ]; then
        print_error "Extrato endpoint returned HTTP $HTTP_CODE instead of 200"
        exit 1
    fi

    # Test transaction endpoint
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"valor": 1, "tipo": "c", "descricao": "teste"}' \
        "$API_BASE_URL/clientes/1/transacoes")
    if [ "$HTTP_CODE" != "200" ]; then
        print_error "Transaction endpoint returned HTTP $HTTP_CODE instead of 200"
        exit 1
    fi

    # Test 404 for non-existent client
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_BASE_URL/clientes/999/extrato")
    if [ "$HTTP_CODE" != "404" ]; then
        print_error "Non-existent client endpoint returned HTTP $HTTP_CODE instead of 404"
        exit 1
    fi

	# Reset endpoint (POST)
	HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
		-X POST \
		"$API_BASE_URL/reset")
	if [ "$HTTP_CODE" != "200" ]; then
		print_error "Reset endpoint returned HTTP $HTTP_CODE instead of 200"
		exit 1
	fi

    print_status "All API endpoints are working correctly"
}

prepare_directories() {
    print_status "Preparing load test directories..."

    # Ensure results directory exists
    mkdir -p "$RESULTS_DIR"

    # Clean old results if any
    if [ "$(ls -A $RESULTS_DIR 2>/dev/null)" ]; then
        print_warning "Cleaning previous test results..."
        rm -rf "$RESULTS_DIR"/*
    fi

    print_status "Load test directories prepared"
}

run_gatling_test() {
    print_status "Starting Gatling load test..."
    print_status "Simulation class: $SIMULATION_CLASS"
    print_status "User files directory: $USER_FILES_DIR"

    echo ""
    print_warning "This test will run for approximately 5 minutes with high load"
    print_warning "Monitor your system resources during the test"
    echo ""

    # Change to Gatling directory to run the test
    cd "$GATLING_HOME"

    # Run Gatling with our simulation
    print_status "Executing Gatling simulation..."

    if ! ./bin/gatling.sh \
        --simulations-folder "$USER_FILES_DIR/simulations" \
        --resources-folder "$USER_FILES_DIR/resources" \
        --results-folder "$RESULTS_DIR" \
        --simulation "$SIMULATION_CLASS"; then
        print_error "Gatling test execution failed"
        exit 1
    fi

    print_status "Gatling test completed successfully!"
}

show_results() {
    print_status "Load test results:"
    echo ""

    # Find the latest results directory
    LATEST_RESULT=$(ls -1t "$RESULTS_DIR" | head -n 1)

    if [ -n "$LATEST_RESULT" ]; then
        REPORT_PATH="$RESULTS_DIR/$LATEST_RESULT/index.html"

        if [ -f "$REPORT_PATH" ]; then
            print_status "HTML report generated: $REPORT_PATH"
            print_status "Open the report in your browser to view detailed results"

            # Try to open the report automatically (macOS/Linux)
            if command -v open &> /dev/null; then
                print_status "Opening report in default browser..."
                open "$REPORT_PATH"
            elif command -v xdg-open &> /dev/null; then
                print_status "Opening report in default browser..."
                xdg-open "$REPORT_PATH"
            fi
        else
            print_warning "HTML report not found at expected location"
        fi

        # Show summary statistics if available
        SIMULATION_LOG="$RESULTS_DIR/$LATEST_RESULT/simulation.log"
        if [ -f "$SIMULATION_LOG" ]; then
            print_status "Quick statistics from simulation.log:"
            echo ""
            # Extract some basic stats (this is a simplified version)
            echo "Total requests: $(grep -c "REQUEST" "$SIMULATION_LOG" || echo "N/A")"
            echo "Failed requests: $(grep -c "KO" "$SIMULATION_LOG" || echo "0")"
        fi
    else
        print_warning "No test results found in $RESULTS_DIR"
    fi
}

main() {
    print_header

    # Check all prerequisites
    check_java
    check_gatling
    check_api_health

    # Prepare environment
    prepare_directories

    # Run the test
    run_gatling_test

    # Show results
    show_results

    echo ""
    print_status "Load test execution completed!"
    print_status "Check the HTML report for detailed performance analysis"
}

# Handle Ctrl+C gracefully
trap 'echo -e "\n${YELLOW}[WARN]${NC} Test interrupted by user"; exit 130' INT

# Run main function
main "$@"

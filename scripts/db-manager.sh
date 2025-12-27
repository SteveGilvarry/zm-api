#!/bin/bash
# Cross-platform database manager for development and CI/CD
# Supports: Apple Container (macOS), Docker (Linux/CI), Podman (macOS/Linux), Native (macOS Homebrew)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SCHEMA_FILE="$PROJECT_ROOT/zm_create.sql.in"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_debug() { echo -e "${BLUE}[DEBUG]${NC} $1"; }

# Detect container runtime
detect_runtime() {
    local os_type="$(uname -s)"
    
    # macOS - prefer Apple Container
    if [ "$os_type" = "Darwin" ]; then
        if command -v container &> /dev/null; then
            echo "apple-container"
            return 0
        elif command -v podman &> /dev/null; then
            echo "podman"
            return 0
        elif command -v docker &> /dev/null && docker ps &> /dev/null 2>&1; then
            echo "docker"
            return 0
        elif command -v mysql &> /dev/null && command -v psql &> /dev/null; then
            echo "native"
            return 0
        fi
    else
        # Linux - prefer Docker
        if command -v docker &> /dev/null && docker ps &> /dev/null 2>&1; then
            echo "docker"
            return 0
        elif command -v podman &> /dev/null; then
            echo "podman"
            return 0
        fi
    fi
    
    echo "none"
}

RUNTIME=$(detect_runtime)

log_info "Detected runtime: $RUNTIME ($(uname -s))"

# Container runtime abstraction
case "$RUNTIME" in
    apple-container)
        COMPOSE_CMD=""  # Apple Container doesn't use compose
        CONTAINER_CMD="container"
        NEEDS_SUDO="no"
        CONTAINER_NAME_MYSQL="zm-api-mysql"
        CONTAINER_NAME_POSTGRES="zm-api-postgres"
        log_info "Using Apple Container (native macOS containers)"
        
        # Ensure Apple Container system is running
        if ! container system status &>/dev/null || ! container system status 2>&1 | grep -q "apiserver is running"; then
            log_warn "Apple Container system is not running. Starting it now..."
            log_info "This may take a moment on first run..."
            container system start
            sleep 2
        fi
        ;;
    docker)
        COMPOSE_CMD="docker compose"
        CONTAINER_CMD="docker"
        NEEDS_SUDO="no"
        ;;
    podman)
        COMPOSE_CMD="podman-compose"
        CONTAINER_CMD="podman"
        NEEDS_SUDO="no"
        # Check if podman-compose is available
        if ! command -v podman-compose &> /dev/null; then
            log_warn "podman-compose not found. Install: pip3 install podman-compose"
            log_info "Or use: brew install podman-compose"
            COMPOSE_CMD="podman compose"  # Fallback to podman compose plugin
        fi
        ;;
    native)
        log_info "Using native MySQL/PostgreSQL installations"
        ;;
    none)
        log_error "No container runtime or native databases found!"
        log_info "Install one of:"
        log_info "  - Apple Container (macOS): https://github.com/apple/container"
        log_info "  - Docker Desktop (macOS/Windows/Linux): https://docker.com"
        log_info "  - Podman: brew install podman podman-compose"
        log_info "  - Native: brew install mysql postgresql"
        exit 1
        ;;
esac

# MySQL configuration
MYSQL_HOST="${MYSQL_HOST:-127.0.0.1}"
MYSQL_PORT="${MYSQL_PORT:-3307}"
MYSQL_ROOT_PASSWORD="${MYSQL_ROOT_PASSWORD:-test_root_pass}"
MYSQL_USER="${MYSQL_USER:-zmuser}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-zmpass}"
MYSQL_DATABASE="${MYSQL_DATABASE:-zm_test}"

# PostgreSQL configuration
PG_HOST="${PG_HOST:-127.0.0.1}"
PG_PORT="${PG_PORT:-5433}"
PG_USER="${PG_USER:-postgres}"
PG_PASSWORD="${PG_PASSWORD:-test_root_pass}"
PG_DATABASE="${PG_DATABASE:-zm_test_pg}"

# Process schema file
process_schema() {
    local output_file="$PROJECT_ROOT/zm_schema_processed.sql"
    
    if [ ! -f "$SCHEMA_FILE" ]; then
        log_info "Downloading ZoneMinder schema..."
        curl -sL https://raw.githubusercontent.com/ZoneMinder/zoneminder/master/db/zm_create.sql.in -o "$SCHEMA_FILE"
    fi
    
    log_debug "Processing schema file..." >&2
    sed -e 's/@ZM_DB_NAME@/'"$MYSQL_DATABASE"'/g' \
        -e 's/@ZM_MYSQL_ENGINE@/InnoDB/g' \
        -e 's/@PKGDATADIR@/./g' \
        -e '/^source /d' \
        "$SCHEMA_FILE" > "$output_file"
    
    echo "$output_file"
}

# Run container command with sudo if needed
# Apple Container has different command structure than Docker/Podman
run_container_cmd() {
    local cmd="$1"
    shift
    
    # Handle Apple Container's different command structure
    if [ "$RUNTIME" = "apple-container" ]; then
        case "$cmd" in
            pull|push|tag|save|load)
                # Image commands need 'image' prefix
                if [ "$NEEDS_SUDO" = "yes" ]; then
                    sudo "$CONTAINER_CMD" image "$cmd" "$@"
                else
                    "$CONTAINER_CMD" image "$cmd" "$@"
                fi
                ;;
            images)
                # List images command
                if [ "$NEEDS_SUDO" = "yes" ]; then
                    sudo "$CONTAINER_CMD" image list "$@"
                else
                    "$CONTAINER_CMD" image list "$@"
                fi
                ;;
            ps)
                # List containers command
                if [ "$NEEDS_SUDO" = "yes" ]; then
                    sudo "$CONTAINER_CMD" list "$@"
                else
                    "$CONTAINER_CMD" list "$@"
                fi
                ;;
            *)
                # Other commands work as-is
                if [ "$NEEDS_SUDO" = "yes" ]; then
                    sudo "$CONTAINER_CMD" "$cmd" "$@"
                else
                    "$CONTAINER_CMD" "$cmd" "$@"
                fi
                ;;
        esac
    else
        # Docker/Podman use standard command structure
        if [ "$NEEDS_SUDO" = "yes" ]; then
            sudo "$CONTAINER_CMD" "$cmd" "$@"
        else
            "$CONTAINER_CMD" "$cmd" "$@"
        fi
    fi
}

# Start containers
start_containers() {
    local service="${1:-}"
    
    if [ "$RUNTIME" = "native" ]; then
        log_info "Native mode - ensure MySQL/PostgreSQL services are running"
        log_info "  MySQL:      brew services start mysql"
        log_info "  PostgreSQL: brew services start postgresql"
        return 0
    fi
    
    if [ "$RUNTIME" = "apple-container" ]; then
        log_info "Apple Container doesn't use compose files"
        log_info "Use: $0 mysql    # to start MySQL"
        log_info "     $0 postgresql # to start PostgreSQL"
        return 0
    fi
    
    log_info "Starting containers with $RUNTIME..."
    
    if [ -n "$service" ]; then
        $COMPOSE_CMD -f "$PROJECT_ROOT/docker-compose.test.yml" up -d "$service"
    else
        $COMPOSE_CMD -f "$PROJECT_ROOT/docker-compose.test.yml" up -d
    fi
    
    sleep 5
}

# Stop containers
stop_containers() {
    if [ "$RUNTIME" = "native" ]; then
        log_info "Native mode - use brew services to stop if needed"
        return 0
    fi
    
    if [ "$RUNTIME" = "apple-container" ]; then
        log_info "Stopping Apple Container instances..."
        run_container_cmd ps 2>/dev/null | grep -q "$CONTAINER_NAME_MYSQL" && run_container_cmd stop "$CONTAINER_NAME_MYSQL" || true
        run_container_cmd ps 2>/dev/null | grep -q "$CONTAINER_NAME_POSTGRES" && run_container_cmd stop "$CONTAINER_NAME_POSTGRES" || true
        run_container_cmd rm "$CONTAINER_NAME_MYSQL" 2>/dev/null || true
        run_container_cmd rm "$CONTAINER_NAME_POSTGRES" 2>/dev/null || true
        log_info "Stopped"
        return 0
    fi
    
    log_info "Stopping containers..."
    $COMPOSE_CMD -f "$PROJECT_ROOT/docker-compose.test.yml" down
}

# Setup MySQL
setup_mysql() {
    log_info "Setting up MySQL database..."
    local schema_sql
    schema_sql=$(process_schema)
    local schema_basename
    schema_basename="$(basename "$schema_sql")"
    local schema_container_path="/tmp/zm_schema.sql"
    
    if [ "$RUNTIME" = "native" ]; then
        log_info "Using native MySQL on port 3306"
        MYSQL_PORT=3306
        MYSQL_HOST=localhost
        
        # Create database and user if they don't exist
        mysql -uroot -e "CREATE DATABASE IF NOT EXISTS $MYSQL_DATABASE;" 2>/dev/null || {
            log_warn "Could not connect to MySQL. Ensure it's running: brew services start mysql"
            return 1
        }
        mysql -uroot -e "CREATE USER IF NOT EXISTS '$MYSQL_USER'@'localhost' IDENTIFIED BY '$MYSQL_PASSWORD'; GRANT ALL PRIVILEGES ON \`$MYSQL_DATABASE\`.* TO '$MYSQL_USER'@'localhost'; FLUSH PRIVILEGES;" 2>/dev/null || true
    elif [ "$RUNTIME" = "apple-container" ]; then
        # Check if container is already running
        if run_container_cmd ps 2>/dev/null | grep -q "$CONTAINER_NAME_MYSQL"; then
            log_warn "MySQL container already running"
        else
            # Remove stopped container with same name to avoid "already exists"
            run_container_cmd rm "$CONTAINER_NAME_MYSQL" 2>/dev/null || true

            # Pull MariaDB image if not present
            if ! run_container_cmd images 2>/dev/null | grep -q "mariadb.*11.4"; then
                log_info "Pulling MariaDB 11.4 image..."
                run_container_cmd pull docker.io/library/mariadb:11.4
            fi
            
            # Run MariaDB container
            log_info "Starting MariaDB container..."
            if run_container_cmd run \
                --name "$CONTAINER_NAME_MYSQL" \
                -e MYSQL_ROOT_PASSWORD="$MYSQL_ROOT_PASSWORD" \
                -e MYSQL_DATABASE="$MYSQL_DATABASE" \
                -e MYSQL_USER="$MYSQL_USER" \
                -e MYSQL_PASSWORD="$MYSQL_PASSWORD" \
                -p "$MYSQL_PORT:3306" \
                -v "$PROJECT_ROOT:/workspace:ro" \
                -d \
                docker.io/library/mariadb:11.4 \
                --character-set-server=utf8mb4 \
                --collation-server=utf8mb4_unicode_ci; then
                schema_container_path="/workspace/$schema_basename"
            else
                log_warn "Volume mount failed; retrying without mount"
                run_container_cmd run \
                    --name "$CONTAINER_NAME_MYSQL" \
                    -e MYSQL_ROOT_PASSWORD="$MYSQL_ROOT_PASSWORD" \
                    -e MYSQL_DATABASE="$MYSQL_DATABASE" \
                    -e MYSQL_USER="$MYSQL_USER" \
                    -e MYSQL_PASSWORD="$MYSQL_PASSWORD" \
                    -p "$MYSQL_PORT:3306" \
                    -d \
                    docker.io/library/mariadb:11.4 \
                    --character-set-server=utf8mb4 \
                    --collation-server=utf8mb4_unicode_ci
            fi
            
            log_info "Waiting for MySQL to be ready..."
            sleep 15
            
            # Wait for database to be ready (using container's mysql client)
            for i in {1..30}; do
                if run_container_cmd exec "$CONTAINER_NAME_MYSQL" mariadb -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" -e "SELECT 1" &>/dev/null; then
                    break
                fi
                echo "  Waiting... ($i/30)"
                sleep 2
            done
            
            # Grant remote access to zmuser (MariaDB auto-creates the user but only for localhost)
            log_info "Configuring remote access for zmuser..."
            run_container_cmd exec "$CONTAINER_NAME_MYSQL" mariadb -uroot -p"$MYSQL_ROOT_PASSWORD" -e "GRANT ALL PRIVILEGES ON \`$MYSQL_DATABASE\`.* TO '$MYSQL_USER'@'%' IDENTIFIED BY '$MYSQL_PASSWORD'; FLUSH PRIVILEGES;" 2>/dev/null || true
        fi
    else
        start_containers "test-db"
        sleep 10
    fi
    
    # Load schema
    log_info "Loading ZoneMinder schema into MySQL..."
    
    if [ "$RUNTIME" = "native" ]; then
        mysql -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" "$MYSQL_DATABASE" < "$schema_sql" 2>&1 | grep -v "ERROR 1304" || true
    elif [ "$RUNTIME" = "apple-container" ]; then
        # Use mariadb client inside the container
        log_info "Loading schema in container (this may take a few minutes)..."
        heartbeat_pid=""
        cleanup_heartbeat() {
            if [ -n "$heartbeat_pid" ]; then
                kill "$heartbeat_pid" 2>/dev/null || true
                wait "$heartbeat_pid" 2>/dev/null || true
                heartbeat_pid=""
            fi
        }
        trap cleanup_heartbeat RETURN
        ( while true; do
              log_info "Schema load still running..."
              sleep 15
          done ) &
        heartbeat_pid=$!

        if ! run_container_cmd exec "$CONTAINER_NAME_MYSQL" sh -lc "test -f \"$schema_container_path\""; then
            log_info "Schema not mounted in container; copying..."
            schema_container_path="/tmp/zm_schema.sql"
            if run_container_cmd cp "$schema_sql" "$CONTAINER_NAME_MYSQL:$schema_container_path" 2>/dev/null; then
                :
            else
                run_container_cmd exec -i "$CONTAINER_NAME_MYSQL" sh -lc 'cat > /tmp/zm_schema.sql' < "$schema_sql" || true
            fi
        fi

        run_container_cmd exec "$CONTAINER_NAME_MYSQL" sh -lc \
            "mariadb -u\"$MYSQL_USER\" -p\"$MYSQL_PASSWORD\" \"$MYSQL_DATABASE\" < \"$schema_container_path\" 2>&1 | grep -v \"ERROR 1304\" || true"
        cleanup_heartbeat
        trap - RETURN
    else
        $CONTAINER_CMD exec -i zm-test-mysql \
            mysql -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" "$MYSQL_DATABASE" < "$schema_sql" 2>&1 | grep -v "ERROR 1304" || true
    fi
    
    log_info "✅ MySQL database ready!"
    log_info "   Connection: mysql://$MYSQL_USER:****@$MYSQL_HOST:$MYSQL_PORT/$MYSQL_DATABASE"
}

# Setup PostgreSQL
setup_postgresql() {
    log_info "Setting up PostgreSQL database..."
    
    if [ "$RUNTIME" = "native" ]; then
        log_info "Using native PostgreSQL on port 5432"
        PG_PORT=5432
        PG_HOST=localhost
        
        # Create database if it doesn't exist
        PGPASSWORD="$PG_PASSWORD" psql -h "$PG_HOST" -U "$PG_USER" -c "CREATE DATABASE $PG_DATABASE;" 2>/dev/null || {
            log_info "Database may already exist or PostgreSQL not running"
        }
    elif [ "$RUNTIME" = "apple-container" ]; then
        # Check if container is already running
        if run_container_cmd ps 2>/dev/null | grep -q "$CONTAINER_NAME_POSTGRES"; then
            log_warn "PostgreSQL container already running"
        else
            # Pull PostgreSQL image if not present
            if ! run_container_cmd images 2>/dev/null | grep -q "postgres.*16"; then
                log_info "Pulling PostgreSQL 16 image..."
                run_container_cmd pull docker.io/library/postgres:16-alpine
            fi
            
            # Run PostgreSQL container
            log_info "Starting PostgreSQL container..."
            run_container_cmd run \
                --name "$CONTAINER_NAME_POSTGRES" \
                -e POSTGRES_PASSWORD="$PG_PASSWORD" \
                -e POSTGRES_USER="$PG_USER" \
                -e POSTGRES_DB="$PG_DATABASE" \
                -p "$PG_PORT:5432" \
                -d \
                docker.io/library/postgres:16-alpine
            
            log_info "Waiting for PostgreSQL to be ready..."
            sleep 10
            
            # Wait for database to be ready
            for i in {1..30}; do
                if PGPASSWORD="$PG_PASSWORD" psql -h"$PG_HOST" -p"$PG_PORT" -U"$PG_USER" -c "SELECT 1" &>/dev/null; then
                    break
                fi
                echo "  Waiting... ($i/30)"
                sleep 2
            done
        fi
    else
        start_containers "test-db-postgres"
        sleep 10
    fi
    
    log_info "✅ PostgreSQL database ready!"
    log_info "   Connection: postgresql://$PG_USER:****@$PG_HOST:$PG_PORT/$PG_DATABASE"
    log_warn "   PostgreSQL schema migration not yet implemented"
}

# Dump MySQL schema
dump_mysql() {
    log_info "Dumping MySQL schema..."
    
    local output_file="$PROJECT_ROOT/mysql_schema_dump.sql"
    
    if [ "$RUNTIME" = "native" ]; then
        mysqldump -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" \
            --no-data --skip-comments "$MYSQL_DATABASE" > "$output_file"
    elif [ "$RUNTIME" = "apple-container" ]; then
        mysqldump -h"$MYSQL_HOST" -P"$MYSQL_PORT" -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" \
            --no-data --skip-comments "$MYSQL_DATABASE" > "$output_file"
    else
        $CONTAINER_CMD exec zm-test-mysql \
            mysqldump -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" \
            --no-data --skip-comments "$MYSQL_DATABASE" > "$output_file"
    fi
    
    log_info "✅ Schema dumped to: $output_file"
}

# Generate SeaORM entities
generate_entities() {
    log_info "Generating SeaORM entities from MySQL..."
    
    if ! command -v sea-orm-cli &> /dev/null; then
        log_error "sea-orm-cli not found!"
        log_info "Install: cargo install sea-orm-cli"
        exit 1
    fi
    
    cd "$PROJECT_ROOT"
    
    local db_url="mysql://$MYSQL_USER:$MYSQL_PASSWORD@$MYSQL_HOST:$MYSQL_PORT/$MYSQL_DATABASE"
    
    sea-orm-cli generate entity \
        --database-url "$db_url" \
        --output-dir src/entity_from_mysql \
        --with-serde both
    
    log_info "✅ Entities generated in: src/entity_from_mysql/"
}

# Status check
status() {
    log_info "Database Status Check"
    echo ""
    log_info "Runtime: $RUNTIME"
    echo ""
    
    if [ "$RUNTIME" = "native" ]; then
        echo "  MySQL:"
        mysql -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" -e "SELECT VERSION();" 2>/dev/null && \
            log_info "    ✅ Running" || log_warn "    ❌ Not accessible"
        echo "  PostgreSQL:"
        PGPASSWORD="$PG_PASSWORD" psql -h localhost -U "$PG_USER" -c "SELECT version();" 2>/dev/null | head -3 && \
            log_info "    ✅ Running" || log_warn "    ❌ Not accessible"
    elif [ "$RUNTIME" = "apple-container" ]; then
        echo "  MySQL ($CONTAINER_NAME_MYSQL):"
        if run_container_cmd ps 2>/dev/null | grep -q "$CONTAINER_NAME_MYSQL"; then
            log_info "    ✅ Running on port $MYSQL_PORT"
            run_container_cmd ps | grep "$CONTAINER_NAME_MYSQL"
        else
            log_warn "    ❌ Not running"
        fi
        echo ""
        echo "  PostgreSQL ($CONTAINER_NAME_POSTGRES):"
        if run_container_cmd ps 2>/dev/null | grep -q "$CONTAINER_NAME_POSTGRES"; then
            log_info "    ✅ Running on port $PG_PORT"
            run_container_cmd ps | grep "$CONTAINER_NAME_POSTGRES"
        else
            log_warn "    ❌ Not running"
        fi
    else
        $COMPOSE_CMD -f "$PROJECT_ROOT/docker-compose.test.yml" ps
    fi
    
    echo ""
    log_info "Connection strings:"
    echo "  MySQL:      mysql://$MYSQL_USER:****@$MYSQL_HOST:$MYSQL_PORT/$MYSQL_DATABASE"
    echo "  PostgreSQL: postgresql://$PG_USER:****@$PG_HOST:$PG_PORT/$PG_DATABASE"
}

# Main command router
case "${1:-}" in
    start)
        start_containers "${2:-}"
        ;;
    stop)
        stop_containers
        ;;
    mysql)
        setup_mysql
        ;;
    postgresql|postgres)
        setup_postgresql
        ;;
    both)
        setup_mysql
        setup_postgresql
        ;;
    dump)
        dump_mysql
        ;;
    generate|entities)
        generate_entities
        ;;
    status)
        status
        ;;
    full)
        setup_mysql
        dump_mysql
        generate_entities
        log_info "🎉 Full setup complete!"
        ;;
    *)
        echo "Usage: $0 {start|stop|mysql|postgresql|both|dump|generate|status|full}"
        echo ""
        echo "Detected runtime: $RUNTIME"
        echo ""
        echo "Commands:"
        echo "  start [service] - Start containers (or show native status)"
        echo "  stop            - Stop containers"
        echo "  mysql           - Setup MySQL with ZoneMinder schema"
        echo "  postgresql      - Setup PostgreSQL database"
        echo "  both            - Setup both databases"
        echo "  dump            - Dump MySQL schema for comparison"
        echo "  generate        - Generate SeaORM entities from MySQL"
        echo "  status          - Check database status"
        echo "  full            - Complete setup (mysql + dump + generate)"
        echo ""
        echo "Environment variables:"
        echo "  MYSQL_HOST, MYSQL_PORT, MYSQL_USER, MYSQL_PASSWORD, MYSQL_DATABASE"
        echo "  PG_HOST, PG_PORT, PG_USER, PG_PASSWORD, PG_DATABASE"
        exit 1
        ;;
esac

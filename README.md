# Rinha de Backend 2024 Q1 - Rust + Actix-Web

API RESTful de alta performance desenvolvida em Rust com foco em escalabilidade e concorrência para suportar alto volume de requisições simultâneas.

## 🏗️ Arquitetura do Sistema

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Teste Gatling │───▶│ Nginx :9999  │───▶│   API Instance  │
│                 │    │ Load Balancer│    │   (Rust/Actix)  │
└─────────────────┘    │ Round Robin  │    │                 │
                       └──────────────┘    └─────────────────┘
                              │                      │
                              │             ┌─────────────────┐
                              └────────────▶│   API Instance  │
                                            │   (Rust/Actix)  │
                                            │                 │
                                            └─────────────────┘
                                                      │
                                            ┌─────────────────┐
                                            │   PostgreSQL    │
                                            │   Database      │
                                            │   (Optimized)   │
                                            └─────────────────┘
```

### Componentes da Arquitetura

#### 1. Load Balancer (Nginx)

- **CPU**: 0.17 cores
- **Memória**: 50MB
- **Algoritmo**: Round Robin
- **Health Checks**: Monitoramento ativo das instâncias
- **Porta**: 9999 (conforme especificação)

#### 2. API Instances (2x Rust + Actix-Web)

- **CPU**: 0.6 cores cada
- **Memória**: 200MB cada
- **Workers**: 4 por instância (otimizado para concorrência)
- **Pool de Conexões**: 32 conexões por instância

#### 3. Banco de Dados (PostgreSQL 16)

- **CPU**: 0.13 cores
- **Memória**: 100MB
- **Otimizações**: Configurações específicas para alta performance
- **Índices**: Otimizados para consultas frequentes

## 🎯 Decisões Arquiteturais

### **Segurança do Sistema**

**O que foi implementado:**

- Validação rigorosa de entrada de dados
- Sanitização de parâmetros de URL
- Uso de queries parametrizadas (proteção contra SQL injection)
- Usuário não-root no container
- Isolamento por containers
- Limitação de recursos por container

**Justificativa:**
A segurança foi implementada em camadas, priorizando validação de entrada e isolamento. Para um ambiente de produção, adicionaria autenticação/autorização, HTTPS e rate limiting.

### **Integridade dos Dados**

**O que foi implementado:**

- Transações ACID no PostgreSQL
- Constraints de banco de dados (CHECK, FOREIGN KEY)
- Validação de regras de negócio antes da persistência
- Rollback automático em caso de falha
- Verificação de saldo antes de débitos

**Justificativa:**
O PostgreSQL garante ACID, enquanto a validação em camadas (aplicação + banco) assegura consistência. As transações evitam estados inconsistentes em operações concorrentes.

### **Disponibilidade do Sistema**

**O que foi implementado:**

- 2 instâncias da API para redundância
- Health checks no load balancer
- Failover automático via
- Pool de conexões configurado para evitar esgotamento
- Configuração otimizada do PostgreSQL

**Justificativa:**
A redundância de instâncias elimina ponto único de falha. O health check garante que apenas instâncias saudáveis recebam tráfego.

### **Escalabilidade do Sistema**

**O que foi implementado:**

- Arquitetura horizontalmente escalável
- Load balancing por round robin
- Pool de conexões otimizado
- Stateless APIs (facilita scaling)
- Configuração de workers otimizada

**Justificativa:**
A arquitetura permite adicionar mais instâncias facilmente. O design stateless e pool de conexões maximizam o throughput por recursos utilizados.

### **Performance do Sistema**

**O que foi implementado:**

- Rust (zero-cost abstractions + memory safety)
- Actix-Web (um dos frameworks web mais rápidos)
- PostgreSQL com tuning específico
- Índices otimizados para queries frequentes
- Pool de conexões dimensionado
- Multi-stage Docker build

**Justificativa:**
Rust oferece performance próxima ao C sem sacrificar segurança. PostgreSQL tunado e índices estratégicos minimizam latência. O pool de conexões evita overhead de criação/destruição.

### **Manutenibilidade do Sistema**

**O que foi implementado:**

- Código estruturado e tipado (Rust)
- Separação clara de responsabilidades
- Logs estruturados
- Configuração via variáveis de ambiente
- Docker para padronização de ambiente

**Justificativa:**
O sistema de tipos do Rust previne muitos bugs em tempo de compilação. A containerização garante consistência entre ambientes.

### **Testabilidade do Sistema**

**O que foi implementado:**

- Endpoints de health check
- Logs detalhados para debugging
- Configuração flexível via env vars
- Estrutura modular do código
- Docker compose para ambiente de teste

**Justificativa:**
Health checks facilitam monitoramento. Logs e configuração flexível simplificam debugging e testes em diferentes cenários.

## 🚀 Como Executar

### Pré-requisitos

- Docker
- Docker Compose

### Execução

```bash
docker-compose up --build
```

A API estará disponível em `http://localhost:9999`

### Endpoints

#### Criar Transação

```bash
POST /clientes/{id}/transacoes
Content-Type: application/json

{
  "valor": 1000,
  "tipo": "c",
  "descricao": "deposito"
}
```

#### Consultar Extrato

```bash
GET /clientes/{id}/extrato
```

## 📊 Otimizações de Performance

### PostgreSQL

- `max_connections=200`: Suporte a muitas conexões simultâneas
- `shared_buffers=128MB`: Cache otimizado para workload
- `work_mem=4MB`: Memória para operações de ordenação
- `effective_cache_size=256MB`: Hint do cache do SO
- `checkpoint_completion_target=0.9`: Distribuição de I/O

### Aplicação Rust

- Pool de 32 conexões por instância
- 4 workers por instância do Actix
- Queries otimizadas com índices específicos
- Validação eficiente de dados

### Infraestrutura

- Load balancing inteligente com health checks
- Containers otimizados com multi-stage build
- Configuração de rede Docker em modo bridge

## 📈 Limitações e Trade-offs

### Escolhas Realizadas

1. **PostgreSQL vs NoSQL**: Escolhi PostgreSQL pela robustez ACID necessária para transações financeiras, mesmo com possível trade-off de performance em writes extremos.

2. **Pool de Conexões**: 32 conexões por instância balanceia utilização de recursos vs throughput. Muito alto poderia saturar o banco.

3. **Memória Limitada**: Com apenas 550MB total, priorizei a aplicação (400MB) sobre cache de banco, assumindo que o workload é mais CPU-intensive.

4. **Rust sobre Go/Node**: Rust oferece melhor performance e safety, mas com complexidade de desenvolvimento maior.

## 🔧 Monitoramento

- Health checks em `/health`
- Logs estruturados no stdout
- Métricas de conexão do PostgreSQL
- Failover automático do

---

## 📝 Especificações Atendidas

✅ Load balancer na porta 9999
✅ 2 instâncias da API
✅ Banco de dados persistente
✅ Limites de CPU (1.5 total) e Memória (550MB total)
✅ Clientes pré-cadastrados (IDs 1-5)
✅ Endpoints especificados com validações
✅ Códigos HTTP corretos (200, 404, 422)

**Total de recursos utilizados:**

- CPU: 1.5 cores (0.6 + 0.6 + 0.17 + 0.13)
- Memória: 450MB (200 + 200 + 50 + 100) - margem para overhead do Docker

## 🚀 Load Testing com Gatling

Esta implementação inclui testes de carga abrangentes usando Gatling, seguindo as especificações oficiais da Rinha de Backend 2024 Q1.

### 📋 Pré-requisitos para Load Testing

#### 1. Java Development Kit (JDK)

```bash
# Verificar se o Java está instalado
java -version

# Instalar Java 8+ (se necessário)
# macOS:
brew install openjdk@11

# Ubuntu/Debian:
sudo apt update && sudo apt install openjdk-11-jdk

# CentOS/RHEL:
sudo yum install java-11-openjdk-devel
```

#### 2. Gatling Installation

```bash
# Download Gatling (versão 3.9.5 ou superior)
wget https://repo1.maven.org/maven2/io/gatling/highcharts/gatling-charts-highcharts-bundle/3.9.5/gatling-charts-highcharts-bundle-3.9.5-bundle.zip

# Extrair o arquivo
unzip gatling-charts-highcharts-bundle-3.9.5-bundle.zip

# Configurar variável de ambiente
export GATLING_HOME=/path/to/gatling-charts-highcharts-bundle-3.9.5
```

**Adicione ao seu `.bashrc` ou `.zshrc`:**

```bash
export GATLING_HOME=/path/to/gatling-charts-highcharts-bundle-3.9.5
export PATH=$PATH:$GATLING_HOME/bin
```

### 🎯 Cenários de Teste Implementados

O arquivo de simulação `RinhaBackendCrebitosSimulation.scala` inclui:

#### 1. **Cenários Principais**

- **Débitos**: Transações de débito com validação de limite
- **Créditos**: Transações de crédito
- **Extratos**: Consultas de extrato com últimas transações

#### 2. **Validações de Concorrência**

- Teste de 25 transações simultâneas
- Validação de consistência de saldo
- Verificação de integridade ACID

#### 3. **Cenários de Erro**

- HTTP 404 para clientes inexistentes
- HTTP 422 para dados inválidos
- Validação de regras de negócio

#### 4. **Padrões de Carga**

- **Débitos**: 1 → 220 RPS durante 2 min, depois 220 RPS constante por 2 min
- **Créditos**: 1 → 110 RPS durante 2 min, depois 110 RPS constante por 2 min
- **Extratos**: 1 → 10 RPS durante 2 min, depois 10 RPS constante por 2 min

### 🔍 Estrutura dos Testes

```
load-test/
├── user-files/
│   ├── simulations/
│   │   └── rinhabackend/
│   │       └── RinhaBackendCrebitosSimulation.scala
│   └── results/
│       └── [relatórios gerados automaticamente]
├── executar-teste-local.sh      # Script principal de execução
└── check-api-health.sh          # Verificação de saúde da API
```

### 🏃‍♂️ Executando os Testes

#### Passo 1: Verificar Saúde da API

```bash
# Verificar se a API está funcionando corretamente
./check-api-health.sh
```

**Saída esperada:**

```
🔍 Checking API readiness...

[ℹ] Testing API connectivity...
[✓] API is responding at http://localhost:9999
[ℹ] Testing all client endpoints...
[✓] Client 1: OK
[✓] Client 2: OK
[✓] Client 3: OK
[✓] Client 4: OK
[✓] Client 5: OK
[ℹ] Testing transaction creation...
[✓] Transaction endpoint: OK
[ℹ] Testing error handling...
[✓] 404 error handling: OK
[✓] 422/400 error handling: OK

🎉 API is ready for load testing!
```

#### Passo 2: Executar Load Test

```bash
# Executar o teste de carga completo
./executar-teste-local.sh
```

O script irá:

1. ✅ Verificar pré-requisitos (Java, Gatling, API)
2. 🧹 Limpar resultados anteriores
3. 🚀 Executar simulação Gatling (~5 minutos)
4. 📊 Gerar relatório HTML
5. 🌐 Abrir relatório no navegador automaticamente

### 📊 Interpretando os Resultados

#### Métricas Importantes

1. **Response Time Percentiles**
   - P50 (mediana): < 50ms recomendado
   - P95: < 200ms recomendado
   - P99: < 500ms recomendado

2. **Request Rate**
   - Débitos: Deve sustentar 220 RPS
   - Créditos: Deve sustentar 110 RPS
   - Extratos: Deve sustentar 10 RPS

3. **Error Rate**
   - < 1% de erros inesperados
   - 404/422 devem ser tratados corretamente

#### Relatório HTML

O Gatling gera um relatório HTML detalhado com:

- Gráficos de throughput ao longo do tempo
- Distribuição de tempos de resposta
- Análise de percentis
- Estatísticas por cenário
- Timeline de execução

### 🎯 Critérios de Aceitação

Para considerar o teste bem-sucedido:

✅ **Performance**

- Nenhum erro HTTP 5xx
- 99% das requisições < 500ms
- Throughput sustentado conforme especificado

✅ **Consistência**

- Validação de saldo/limite sempre consistente
- Transações ACID funcionando corretamente
- Extratos refletem transações em tempo real

✅ **Escalabilidade**

- Sistema estável durante toda duração do teste
- Sem degradação significativa de performance
- Memory/CPU dentro dos limites especificados

### 🔧 Troubleshooting

#### Erro: "GATLING_HOME não definido"

```bash
export GATLING_HOME=/caminho/para/gatling
```

#### Erro: "API não está respondendo"

```bash
# Verificar se containers estão rodando
docker-compose ps

# Reiniciar se necessário
./run.sh
```

#### Performance baixa

1. Verificar logs do container: `docker-compose logs api1 api2`
2. Monitorar recursos: `docker stats`
3. Verificar conectividade do banco: logs PostgreSQL

### 📝 Customização dos Testes

Para modificar os testes, edite o arquivo:
`load-test/user-files/simulations/rinhabackend/RinhaBackendCrebitosSimulation.scala`

Principais parâmetros configuráveis:

- `rampUsersPerSec()`: Taxa de crescimento de usuários
- `constantUsersPerSec()`: Taxa constante de requisições
- `during()`: Duração de cada fase
- `randomClienteId()`: Range de IDs de clientes testados

---

## 📁 Arquivos e Scripts

### Scripts de Execução

- `run.sh` - Inicia o ambiente Docker completo
- `test-api.sh` - Testa endpoints da API manualmente
- `executar-teste-local.sh` - Executa testes de carga Gatling
- `check-api-health.sh` - Verifica saúde da API antes dos testes
- `validate-setup.sh` - Validação completa do ambiente

### Documentação

- `README.md` - Documentação principal do projeto

### Configuração

- `docker-compose.yml` - Orquestração dos containers
- `Dockerfile` - Build da aplicação Rust
- `file` - Configuração do load balancer
- `script.sql` - Inicialização do banco de dados

### Load Testing

```
load-test/
├── user-files/
│   ├── simulations/
│   │   └── rinhabackend/
│   │       └── RinhaBackendCrebitosSimulation.scala
│   └── results/ (gerado após execução dos testes)
```

### Comandos Rápidos

```bash
# Iniciar ambiente
./run.sh

# Verificar API
./check-api-health.sh

# Validar setup completo
./validate-setup.sh

# Executar load tests
./executar-teste-local.sh

# Ver logs
docker-compose logs -f api01 api02

# Parar ambiente
docker-compose down
```

---

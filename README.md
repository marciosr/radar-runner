radar-runner 0.2
Você

Executa o radar de ativos com modos de bypass ou histórico.

USAGE:
    radar-runner <COMMAND>

COMMANDS:
    bypass      Executa com bypass (força execução)
    historico   Executa em modo histórico
    help        Print this message or the help of the given subcommand(s)

ARGS (para comandos):
    <tipo>  Tipo de ativo a ser monitorado [possible values: acao, fundo]

# Executa radar em modo histórico para ações
radar-runner historico acao

# Executa radar forçando execução para fundos
radar-runner bypass fundo

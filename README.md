# radar-runner 0.4 ✨

**Ferramenta de agendamento e execução periódica para o `radar-fundamentos`.**

O `radar-runner` gerencia o cronograma de coleta de dados, respeitando dias úteis, feriados e horários configurados, chamando o binário `radar-fundamentos` nos momentos apropriados.

---

## ⚙️ Configuração Principal

A frequência de execução, o horário de funcionamento e a lista de ativos são definidos no arquivo de configuração TOML:

* **Localização:** `~/.config/radar/radar-runner.conf`
* **Criação:** O arquivo é criado automaticamente com valores padrão na primeira execução.

### Exemplo de `radar-runner.conf`:

```toml
# Datas de feriados no formato YYYY-MM-DD
feriados = ["2025-01-01", "2025-04-21"]

# Intervalo de horas para rastreio (Fuso de São Paulo, 0..23)
intervalo_inicio = 10
intervalo_fim = 20

# Frequência de execução do modo 'cotacoes' em minutos
frequencia_minutos = 15

# Lista de códigos de ativos de alta frequência (usada pelo modo 'cotacoes')
ativos_codes = ["VALE3", "PRIO3", "KLBN11", "AFHI11"]

# Listas de ativos para o modo 'historico' (tipos específicos)
acao_codes = ["VALE3", "PRIO3", "KLBN11"]
fundo_codes = ["AFHI11", "VGIR11"]
````

-----

## 📦 USO:

```bash
radar-runner <COMANDO>
```

### COMANDOS DISPONÍVEIS:

| Comando | Descrição | Utiliza | Frequência |
| :--- | :--- | :--- | :--- |
| **`cotacoes`** | Executa a coleta de cotações dos **`ativos_codes`** periodicamente, respeitando a `frequencia_minutos` e o `intervalo_inicio`/`fim` do TOML. | `radar-fundamentos cotacoes <ativos> --saida ...` | Periódica (Agendada) |
| **`historico <tipo>`** | Executa a coleta fundamentalista dos ativos (`acao_codes` ou `fundo_codes`) periodicamente. Ideal para rodar a cada 3h ou 4h, dentro do intervalo definido. | `radar-fundamentos export <tipo> <ativos> --saida ...` | Periódica (Agendada) |
| **`cotacoes-agora`** | Executa a coleta de cotações dos **`ativos_codes`** **uma única vez**, ignorando o agendamento de horário e frequência. | `radar-fundamentos cotacoes <ativos> --saida ...` | Única (Bypass) |

#### Exemplos de Execução:

  * **Modo Periódico (Alta Frequência):**

    ```bash
    # Roda a cada 15 minutos (ou conforme TOML), entre 10h e 20h.
    radar-runner cotacoes
    ```

  * **Modo Histórico (Baixa Frequência):**

    ```bash
    # Roda fundos (FIIs) a cada 3 horas, entre 10h e 20h.
    radar-runner historico fundo
    ```

  * **Modo Bypass (Execução Imediata):**

    ```bash
    # Coleta as cotações imediatamente, ignorando o horário atual.
    radar-runner cotacoes-agora
    ```

-----

## ⚙️ Agendamento com systemd (modo usuário)

Para manter o `radar-runner` ativo e executando o modo `cotacoes` (alta frequência) de forma contínua, você deve configurá-lo como um serviço de usuário.

Crie o arquivo `~/.config/systemd/user/radar-runner-cotacoes.service` com o seguinte conteúdo:

```ini
[Unit]
Description=Radar Runner - Coleta Periódica de Cotações

[Service]
# Chama o comando 'cotacoes' que entrará em loop e aguardará a frequência definida no TOML.
ExecStart=%h/.cargo/bin/radar-runner cotacoes
# Reinicia automaticamente em caso de falha (crash).
Restart=on-failure
# Define o diretório de trabalho, útil para o radar-fundamentos.
WorkingDirectory=%h
```

Ative e inicie o serviço:

```bash
systemctl --user daemon-reload
systemctl --user enable --now radar-runner-cotacoes.service
```

Você pode verificar o status e logs com:

```bash
systemctl --user status radar-runner-cotacoes.service
journalctl --user -u radar-runner-cotacoes.service -f
```

-----

## 🧠 Observações

  * **Tipos aceitos no modo `historico`:** `acao`, `fundo`.
  * **Binários:** Requer que o binário **`radar-fundamentos`** esteja no **PATH** (`~/.cargo/bin`).
  * **Uso:** Ideal para automações locais devido ao seu baixo consumo de recursos.

-----

## ✨ Em desenvolvimento

  * Registro automático de falhas
  * Notificações (email, DBus, webhook)

<!-- end list -->

```

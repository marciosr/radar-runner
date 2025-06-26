# radar-runner 0.2

**Executa o radar de ativos com modos agendado, bypass ou histórico.**

---

## 📦 USO:

```bash
radar-runner <COMANDO> [opções]
```

### COMANDOS DISPONÍVEIS:

#### `historico <tipo>`

Executa o radar em modo histórico para o tipo de ativo especificado.

**Exemplo:**

```bash
radar-runner historico acao
```

#### `bypass <tipo>`

Executa o radar imediatamente, forçando a execução mesmo fora do agendamento.

**Exemplo:**

```bash
radar-runner bypass fundo
```

#### `cotacoes`

Executa a coleta de cotações com base no arquivo `ativos.yaml`, chamando o binário `radar-fundamentus`. Este comando pode ser agendado.

**Exemplo:**

```bash
radar-runner cotacoes
```

##### Opções para `cotacoes`:

* `--frequencia <minutos>`: Define o intervalo entre execuções (ex: `15m`)
* `--intervalo <hora_inicial>..<hora_final>`: Restringe a faixa horária de execução (ex: `10..17`)

**Exemplo completo:**

```bash
radar-runner cotacoes --frequencia 15m --intervalo 10..17
```

---

## ⚙️ Configuração com systemd (modo usuário)

Crie o arquivo `~/.config/systemd/user/radar-runner.service` com o seguinte conteúdo:

```ini
[Unit]
Description=Radar Runner - Agendador de Coletas

[Service]
ExecStart=%h/.cargo/bin/radar-runner cotacoes --frequencia 15m --intervalo 10..17
Restart=on-failure
WorkingDirectory=%h/radar-dados

[Install]
WantedBy=default.target
```

Ative com:

```bash
systemctl --user daemon-reexec
systemctl --user enable --now radar-runner.service
```

---

## 🧠 Observações

* Tipos aceitos: `acao`, `fundo`
* Uso mínimo de memória (\~1,2 MB)
* Ideal para sistemas leves e automações locais
* Requer que o binário `radar-fundamentus` esteja no PATH (`~/.cargo/bin`)

---

## ✨ Em desenvolvimento

* Registro automático de falhas
* Notificações (email, DBus, webhook)
* Dry-run para testes

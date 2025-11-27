use chrono::{Datelike, Local, NaiveDate, Timelike};
use chrono_tz::America::Sao_Paulo;
use clap::{Parser, Subcommand, ValueEnum};
use dirs;
use serde::Deserialize;
use std::{
	fs::{File, create_dir_all},
	io::{Read, Write},
	path::PathBuf,
	process::Command,
	thread::sleep,
	time::Duration,
};
use toml;

// --- Funções Auxiliares de Configuração ---

fn default_feriados() -> Vec<String> {
	Vec::new()
}

// NOVO: Função padrão para a lista COMPLETA de ativos (alta frequência/cotações)
fn default_ativos_codes() -> Vec<String> {
	vec![
		"SNEL11".to_string(),
		"AFHI11".to_string(),
		"RELG11".to_string(),
		"VGIR11".to_string(),
		"VALE3".to_string(),
		"PRIO3".to_string(),
		"BRAV3".to_string(),
		"KLBN11".to_string(),
		"ITSA4".to_string(),
	]
}

fn default_acao_codes() -> Vec<String> {
	vec![
		"VALE3".to_string(),
		"PRIO3".to_string(),
		"BRAV3".to_string(),
		"KLBN11".to_string(),
		"ITSA4".to_string(),
	]
}
fn default_fundo_codes() -> Vec<String> {
	vec![
		"SNEL11".to_string(),
		"AFHI11".to_string(),
		"RELG11".to_string(),
		"VGIR11".to_string(),
	]
}
fn default_intervalo_inicio() -> u32 {
	10
}
fn default_intervalo_fim() -> u32 {
	20
}
fn default_frequencia_minutos() -> u64 {
	15
}

/// Estrutura para as configurações do radar-runner lidas de radar-runner.conf (TOML)
#[derive(Deserialize)]
struct RunnerConfig {
	#[serde(default = "default_feriados")]
	feriados: Vec<String>,

	#[serde(default = "default_intervalo_inicio")]
	intervalo_inicio: u32,
	#[serde(default = "default_intervalo_fim")]
	intervalo_fim: u32,

	#[serde(default = "default_frequencia_minutos")]
	frequencia_minutos: u64,

	// NOVO: Lista de ativos de alta frequência (cotacoes)
	#[serde(default = "default_ativos_codes")]
	ativos_codes: Vec<String>,

	// Listas de ativos (baixa frequência / fundamentalista)
	#[serde(default = "default_acao_codes")]
	acao_codes: Vec<String>,
	#[serde(default = "default_fundo_codes")]
	fundo_codes: Vec<String>,
}

/// Retorna o caminho para o arquivo de configuração TOML: ~/.config/radar/radar-runner.conf
fn get_config_path() -> PathBuf {
	let mut config_path = dirs::config_dir().unwrap_or_else(|| {
		eprintln!(
			"[runner] Aviso: Não foi possível determinar o diretório de configuração. Usando ./"
		);
		PathBuf::from(".")
	});
	config_path.push("radar");
	let _ = create_dir_all(&config_path);
	config_path.push("radar-runner.conf");
	config_path
}

/// Carrega a configuração principal do radar-runner a partir do arquivo TOML.
/// Cria o arquivo padrão caso não exista.
fn carregar_config() -> RunnerConfig {
	let caminho = get_config_path();

	// Conteúdo padrão a ser escrito se o arquivo não existir (Atualizado com ativos_codes)
	let padrao_toml = r#"
# Datas de feriados no formato YYYY-MM-DD
feriados = [
    "2025-01-01",
    "2025-03-03",
    "2025-03-04",
    "2025-04-18",
    "2025-04-21",
    "2025-05-01",
    "2025-06-19",
    "2025-11-20",
    "2025-12-24",
    "2025-12-25",
    "2025-12-31"
]

# Intervalo de horas para rastreio periódico (0..23)
intervalo_inicio = 10
intervalo_fim = 20

# Frequência de execução do modo 'cotacoes' em minutos
frequencia_minutos = 15

# Lista de códigos de ativos de alta frequência (cotacoes)
ativos_codes = ["VALE3", "PRIO3", "BRAV3", "KLBN11", "ITSA4", "SNEL11", "AFHI11", "RELG11", "VGIR11"]

# Lista de códigos de ativos de Ações (fundamentalista/histórico)
acao_codes = ["VALE3", "PRIO3", "BRAV3", "KLBN11", "ITSA4"]

# Lista de códigos de Fundos (fundamentalista/histórico)
fundo_codes = ["SNEL11", "AFHI11", "RELG11", "VGIR11"]
"#;

	// Cria o arquivo padrão se não existir
	if !caminho.exists() {
		if let Ok(mut arquivo) = File::create(&caminho) {
			if arquivo.write_all(padrao_toml.as_bytes()).is_ok() {
				println!(
					"[runner] Arquivo de configuração padrão criado: {}",
					caminho.display()
				);
			} else {
				eprintln!(
					"[runner] Falha ao escrever arquivo padrão: {}",
					caminho.display()
				);
			}
		} else {
			eprintln!(
				"[runner] Falha ao criar arquivo padrão: {}",
				caminho.display()
			);
		}
	}

	// Tenta ler e parsear o arquivo
	if let Ok(mut arquivo) = File::open(&caminho) {
		let mut conteudo = String::new();
		if arquivo.read_to_string(&mut conteudo).is_ok() {
			match toml::from_str::<RunnerConfig>(&conteudo) {
				Ok(cfg) => return cfg,
				Err(e) => eprintln!(
					"[runner] Falha ao parsear {}: {} => usando padrão.",
					caminho.display(),
					e
				),
			}
		} else {
			eprintln!(
				"[runner] Falha ao ler {}: usando padrão.",
				caminho.display()
			);
		}
	} else {
		eprintln!(
			"[runner] Aviso: Arquivo de configuração não encontrado: {}. Usando padrão.",
			caminho.display()
		);
	}

	// Retorna a configuração padrão em caso de falha
	RunnerConfig {
		feriados: default_feriados(),
		intervalo_inicio: default_intervalo_inicio(),
		intervalo_fim: default_intervalo_fim(),
		frequencia_minutos: default_frequencia_minutos(),
		ativos_codes: default_ativos_codes(),
		acao_codes: default_acao_codes(),
		fundo_codes: default_fundo_codes(),
	}
}

/// Retorna a lista de ativos a partir da configuração carregada.
fn get_ativos_from_config(config: &RunnerConfig, tipo: TipoAtivo) -> &[String] {
	match tipo {
		TipoAtivo::Acao => &config.acao_codes,
		TipoAtivo::Fundo => &config.fundo_codes,
	}
}

// --- Funções de Lógica ---

/// Verifica se a data é dia útil (Segunda a Sexta) e não está em feriados
fn e_dia_util(data: NaiveDate, feriados: &[String]) -> bool {
	let dia_semana = data.weekday().number_from_monday();
	let data_str = data.format("%Y-%m-%d").to_string();
	(1..=5).contains(&dia_semana) && !feriados.contains(&data_str)
}

/// Retorna o caminho base para os arquivos de dados (e.g., para o modo histórico)
fn get_data_base_path() -> PathBuf {
	let mut data_path = dirs::data_dir().unwrap_or_else(|| {
		eprintln!("[runner] Aviso: Não foi possível determinar o diretório de dados. Usando ./");
		PathBuf::from(".")
	});
	data_path.push("radar");
	let _ = create_dir_all(&data_path);
	data_path
}

/// Executa o comando `radar-fundamentos` coma as opções adequadas.
fn executar_radar(
	tipo: &str,
	codes: &[String],
	historico: bool,
	feriados: &[String],
	intervalo_inicio: u32,
	intervalo_fim: u32,
	executar_agora: bool,
) {
	let agora = Local::now().with_timezone(&Sao_Paulo);
	let hoje = agora.date_naive();
	let hora = agora.hour();

	let cond_exec_periodica =
		e_dia_util(hoje, feriados) && (intervalo_inicio..=intervalo_fim).contains(&hora);

	let cond_exec = executar_agora || cond_exec_periodica;

	println!(
		"[runner] [{}] Hora atual: {}h, data {} => Executável? {}",
		tipo,
		hora,
		hoje.format("%Y-%m-%d"),
		cond_exec
	);

	if cond_exec {
		println!("[runner] [{}] Iniciando `radar-fundamentos`...", tipo);
		let mut cmd = Command::new("radar-fundamentos");
		let subcomando = if historico { "export" } else { "cotacoes" };

		cmd.arg(subcomando);

		if historico {
			cmd.arg(tipo);
		}

		for code in codes {
			cmd.arg(code);
		}

		if historico {
			let mut dir = get_data_base_path();
			dir.push("dados/historico");
			let _ = create_dir_all(&dir);
			let timestamp = agora.format("%Y-%m-%d_%Hh-%Mm-%Ss");
			let arquivo = dir.join(format!("{}_{}.csv", tipo, timestamp));
			cmd.arg("--saida").arg(&arquivo);
			println!(
				"[runner] [{}] Modo histórico, saída em {}",
				tipo,
				arquivo.display()
			);
		} else if subcomando == "cotacoes" {
			let saida_padrao = get_data_base_path().join("cotacoes.csv");
			cmd.arg("--saida").arg(&saida_padrao);
			println!("[runner] [{}] Saída em {}", tipo, saida_padrao.display());
		}

		println!("[runner] [{}] Comando completo: {:?}", tipo, cmd);
		match cmd.status() {
			Ok(status) => println!("[runner] [{}] Finalizado com: {}", tipo, status),
			Err(e) => eprintln!("[runner] [{}] Erro ao executar: {}", tipo, e),
		}
	} else {
		println!(
			"[runner] [{}] Fora do horário ({}..{}) ou feriado. Aguardando a próxima janela.",
			tipo, intervalo_inicio, intervalo_fim
		);
	}
}

// --- Definições de CLI Simplificadas ---

#[derive(Parser)]
#[command(name = "radar-runner", version = "0.4", author = "Author")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Executa a coleta de cotações uma única vez, ignorando agendamento.
	CotacoesAgora,

	/// Executa em modo histórico (periódico, ativos e horários do TOML).
	Historico {
		#[arg(value_enum)]
		tipo: TipoAtivo,
	},

	/// Executa a coleta de cotações (periódico, ativos, horários e frequência do TOML).
	Cotacoes,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
enum TipoAtivo {
	Acao,
	Fundo,
}

// --- Função Principal Atualizada ---

fn main() {
	let runner_config = carregar_config();
	let feriados = &runner_config.feriados;
	let cli = Cli::parse();

	let config_inicio = runner_config.intervalo_inicio;
	let config_fim = runner_config.intervalo_fim;
	let frequencia_minutos = runner_config.frequencia_minutos;

	// NOVO: Usaremos esta lista para as cotações
	let ativos_de_cotacoes = &runner_config.ativos_codes;

	// Lista de tipos e ativos a serem iterados (Usada apenas para o modo Historico)
	let _ativos_e_tipos_fundamentalista = [
		(
			TipoAtivo::Acao,
			get_ativos_from_config(&runner_config, TipoAtivo::Acao),
		),
		(
			TipoAtivo::Fundo,
			get_ativos_from_config(&runner_config, TipoAtivo::Fundo),
		),
	];

	match cli.command {
		// MODO SOB DEMANDA (COTAÇÕES AGORA)
		Commands::CotacoesAgora => {
			println!(
				"[runner] Execução única de cotações (bypass) para {} ativos.",
				ativos_de_cotacoes.len()
			);

			executar_radar(
				"geral",
				ativos_de_cotacoes, // Passa a lista COMPLETA de ativos
				false,
				feriados,
				config_inicio,
				config_fim,
				true,
			);
		}

		// MODO HISTÓRICO (PERIÓDICO)
		Commands::Historico { tipo } => {
			let codes = get_ativos_from_config(&runner_config, tipo);
			let tipo_str = format!("{:?}", tipo).to_lowercase();
			let intervalo_secs = 3 * 3600;

			println!(
				"[runner] Iniciando modo histórico (tipo={}). Checagem a cada 3 horas, no intervalo {}:00 - {}:00.",
				tipo_str, config_inicio, config_fim
			);
			loop {
				executar_radar(
					&tipo_str,
					codes,
					true,
					feriados,
					config_inicio,
					config_fim,
					false,
				);
				sleep(Duration::from_secs(intervalo_secs));
			}
		}

		// MODO COTAÇÕES (PERIÓDICO)
		Commands::Cotacoes => loop {
			println!(
				"[runner] Iniciando coleta periódica de {} cotações.",
				ativos_de_cotacoes.len()
			);

			executar_radar(
				"geral",
				ativos_de_cotacoes, // Passa a lista COMPLETA de ativos
				false,
				feriados,
				config_inicio,
				config_fim,
				false,
			);

			println!(
				"[runner] Aguardando {} minutos (frequência do TOML)...",
				frequencia_minutos
			);
			sleep(Duration::from_secs(frequencia_minutos * 60));
		},
	}
}

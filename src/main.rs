use chrono::{Datelike, Local, NaiveDate, Timelike};
use chrono_tz::America::Sao_Paulo;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_yml;
use std::{
	fs::{File, create_dir_all},
	io::{Read, Write},
	path::PathBuf,
	process::Command,
	thread::sleep,
	time::Duration,
};

/// Configuração de feriados lida de um arquivo YAML `feriados.yaml`
#[derive(Deserialize)]
struct FeriadosConfig {
	feriados: Vec<String>,
}

/// Configuração de códigos lida de um arquivo YAML `acoes.yaml` ou `fundos.yaml`
#[derive(Serialize, Deserialize)]
struct CodigosConfig {
	codes: Vec<String>,
}

/// Cria o conteúdo padrão e salva o arquivo YAML se ele não existir
fn salvar_lista_de_ativos(_tipo: TipoAtivo, caminho: &PathBuf, padrao: &[&str]) {
	if !caminho.exists() {
		let config = CodigosConfig {
			codes: padrao.iter().map(|s| s.to_string()).collect(),
		};

		if let Ok(serializado) = serde_yml::to_string(&config) {
			if let Ok(mut arquivo) = File::create(&caminho) {
				let _ = arquivo.write_all(serializado.as_bytes());
				println!("[runner] Arquivo padrão criado: {}", caminho.display());
			}
		}
	}
}

/// Carrega os códigos de ativos a partir de um arquivo YAML conforme o tipo.
/// Cria o arquivo com padrão caso não exista.
fn carregar_ativos(tipo: TipoAtivo) -> Vec<String> {
	let (nome_arquivo, padrao): (&str, &[&str]) = match tipo {
		TipoAtivo::Acao => (
			"/var/lib/radar-runner/acoes.yaml",
			&["VALE3", "PRIO3", "BRAV3", "KLBN11", "ITSA4"],
		),
		TipoAtivo::Fundo => (
			"/var/lib/radar-runner/fundos.yaml",
			&["SNEL11", "AFHI11", "RELG11", "VGIR11"],
		),
	};

	let caminho = PathBuf::from(nome_arquivo);
	salvar_lista_de_ativos(tipo, &caminho, padrao);

	if let Ok(mut arquivo) = File::open(&caminho) {
		let mut conteudo = String::new();
		if arquivo.read_to_string(&mut conteudo).is_ok() {
			if let Ok(cfg) = serde_yml::from_str::<CodigosConfig>(&conteudo) {
				return cfg.codes;
			}
		}
		eprintln!(
			"[runner] Falha ao ler ou parsear {}: usando padrão",
			caminho.display()
		);
	}

	// Fallback
	padrao.iter().map(|s| s.to_string()).collect()
}

/// Carrega os feriados a partir de um arquivo YAML localizado no diretório de trabalho do aplicativo
fn carregar_feriados_yaml() -> Vec<String> {
	let caminho = PathBuf::from("feriados.yaml");
	if let Ok(mut arquivo) = File::open(&caminho) {
		let mut conteudo = String::new();
		if arquivo.read_to_string(&mut conteudo).is_ok() {
			if let Ok(cfg) = serde_yml::from_str::<FeriadosConfig>(&conteudo) {
				return cfg.feriados;
			}
		}
	}
	eprintln!("[runner] Aviso: feriados.yaml não encontrado ou inválido.");
	Vec::new()
}

/// Verifica se a data é dia útil (Segunda a Sexta) e não está em feriados
fn is_dia_util(data: NaiveDate, feriados: &[String]) -> bool {
	let dia_semana = data.weekday().number_from_monday();
	let data_str = data.format("%Y-%m-%d").to_string();
	(1..=5).contains(&dia_semana) && !feriados.contains(&data_str)
}

/// Executa o comando `radar-fundamentos` conforme flags, usando feriados e códigos
fn executar_radar(
	tipo: &str,
	codes: &[String],
	bypass: bool,
	historico: bool,
	feriados: &[String],
) {
	let agora = Local::now().with_timezone(&Sao_Paulo);
	let hoje = agora.date_naive();
	let hora = agora.hour();

	let cond_exec = bypass || (is_dia_util(hoje, feriados) && (17..=21).contains(&hora));
	println!(
		"[runner] [{}] Hora atual: {}h, data {} => executável? {}",
		tipo,
		hora,
		hoje.format("%Y-%m-%d"),
		cond_exec
	);

	if cond_exec {
		println!("[runner] [{}] Iniciando `radar-fundamentos`...", tipo);
		let mut cmd = Command::new("radar-fundamentos");
		cmd.arg("export").arg(tipo);
		for code in codes {
			cmd.arg(code);
		}

		if historico {
			let dir = PathBuf::from("/var/home/marcio/.local/share/radar-runner/dados/historico");
			let _ = create_dir_all(&dir);
			let timestamp = agora.format("%Y-%m-%d_%Hh-%Mm-%Ss");
			let arquivo = dir.join(format!("{}_{}.csv", tipo, timestamp));
			cmd.arg("--saida").arg(&arquivo);
			println!(
				"[runner] [{}] Modo histórico, saída em {}",
				tipo,
				arquivo.display()
			);
		}

		println!("[runner] [{}] Comando completo: {:?}", tipo, cmd);
		match cmd.status() {
			Ok(status) => println!("[runner] [{}] Finalizado com: {}", tipo, status),
			Err(e) => eprintln!("[runner] [{}] Erro ao executar: {}", tipo, e),
		}
	} else {
		println!(
			"[runner] [{}] Fora do horário ou feriado. Bypass={}",
			tipo, bypass
		);
	}
}

#[derive(Parser)]
#[command(name = "radar-runner", version = "0.2", author = "Você")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Executa com bypass (força execução)
	Bypass {
		#[arg(value_enum)]
		tipo: TipoAtivo,
	},
	/// Executa em modo histórico
	Historico {
		#[arg(value_enum)]
		tipo: TipoAtivo,
	},
	/// Executa o radar-fundamentus para atualizar cotações via YAML
	Cotacoes {
		/// Caminho para o arquivo YAML com os ativos
		#[arg(long, default_value = "ativos.yaml")]
		yaml: String,

		/// Caminho para o arquivo CSV de saída
		#[arg(long, default_value = "cotacoes.csv")]
		saida: String,

		/// Frequência de execução em minutos
		#[arg(long, default_value = "15")]
		frequencia: u64,

		/// Hora inicial da janela permitida (ex: 10)
		#[arg(long, default_value = "10")]
		intervalo_inicio: u32,

		/// Hora final da janela permitida (ex: 17)
		#[arg(long, default_value = "17")]
		intervalo_fim: u32,
	},
}

#[derive(Copy, Clone, ValueEnum, Debug)]
enum TipoAtivo {
	Acao,
	Fundo,
}

fn main() {
	let intervalo = 3;
	let feriados = carregar_feriados_yaml();
	let cli = Cli::parse();

	match cli.command {
		Commands::Bypass { tipo } => {
			let codes = carregar_ativos(tipo);
			let tipo_str = format!("{:?}", tipo).to_lowercase();
			println!(
				"[runner] Iniciando com bypass=true historico=false (tipo={})",
				tipo_str
			);
			loop {
				executar_radar(&tipo_str, &codes, true, false, &feriados);
				sleep(Duration::from_secs(intervalo * 3600));
			}
		}
		Commands::Historico { tipo } => {
			let codes = carregar_ativos(tipo);
			let tipo_str = format!("{:?}", tipo).to_lowercase();
			println!(
				"[runner] Iniciando com bypass=false historico=true (tipo={})",
				tipo_str
			);
			loop {
				executar_radar(&tipo_str, &codes, false, true, &feriados);
				sleep(Duration::from_secs(intervalo * 3600));
			}
		}
		//
		Commands::Cotacoes {
			yaml,
			saida,
			frequencia,
			intervalo_inicio,
			intervalo_fim,
		} => {
			//let feriados = carregar_feriados_yaml();

			loop {
				let agora = Local::now().with_timezone(&Sao_Paulo);
				let hora = agora.hour();
				let hoje = agora.date_naive();

				let dentro_da_janela = (intervalo_inicio..=intervalo_fim).contains(&hora);
				let eh_dia_util = is_dia_util(hoje, &feriados);

				println!(
					"[runner] Checando: hora={} ∈ {}..{}, dia útil?={}, feriado={}, frequência={} min",
					hora, intervalo_inicio, intervalo_fim, eh_dia_util, !eh_dia_util, frequencia
				);

				if dentro_da_janela && eh_dia_util {
					println!("[runner] Executando radar-fundamentos...");
					let status = Command::new("radar-fundamentos")
						.args(&["cotacoes", "--yaml", &yaml, "--saida", &saida])
						.status();

					match status {
						Ok(s) if s.success() => {
							println!("[runner] Execução bem-sucedida.");
						}
						Ok(s) => {
							eprintln!("[runner] radar-fundamentus retornou código {}", s);
						}
						Err(e) => {
							eprintln!("[runner] Erro ao executar radar-fundamentus: {}", e);
						}
					}
				} else {
					println!("[runner] Aguardando próxima janela...");
				}

				sleep(Duration::from_secs(frequencia * 60));
			}
		}
	}
}

// Функция для получения бесплатных SOL через airdrop в Devnet
async fn request_airdrops_for_source_wallets(config: &Config, rpc_client: &RpcClient) -> Result<()> {
    info!("Requesting airdrops for source wallets...");
    
    // Создаем прогресс-бар для отслеживания airdrops
    let pb = ProgressBar::new(config.source_wallets.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} Airdrops ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    
    for (_i, source_wallet) in config.source_wallets.iter().enumerate() {
        // Создаем keypair из секретного ключа
        let keypair = create_keypair_from_secret(&source_wallet.secret_key)?;
        let pubkey = keypair.pubkey();
        
        // Определяем сумму для airdrop (1 SOL должно быть достаточно для тестов)
        // В Devnet обычно ограничение 2 SOL на запрос, но часто бывают rate limits
        let airdrop_amount = 1.0;
        let airdrop_lamports = (airdrop_amount * 1_000_000_000.0) as u64;
        
        // Проверяем текущий баланс
        let balance = rpc_client.get_balance(&pubkey)
            .with_context(|| format!("Failed to get balance for wallet {}", pubkey))?;
        
        if balance < 10_000_000 {  // Если меньше 0.01 SOL
            info!("Requesting airdrop of {} SOL for wallet {}", airdrop_amount, pubkey);
            
            // Создаем новый клиент для каждого запроса airdrop
            let cluster_url = config.network.cluster_url.clone();
            let commitment = CommitmentConfig::from_str(&config.network.commitment)
                .with_context(|| format!("Invalid commitment level: {}", config.network.commitment))?;
            
            // Выполняем запрос на airdrop
            let task_rpc_client = RpcClient::new_with_commitment(cluster_url, commitment);
            
            // Максимальное количество попыток для airdrop
            let max_airdrop_attempts = 3;
            let mut airdrop_succeeded = false;
            
            for attempt in 1..=max_airdrop_attempts {
                match task_rpc_client.request_airdrop(&pubkey, airdrop_lamports) {
                    Ok(signature) => {
                        info!("Airdrop requested for wallet {} (attempt {}/{}), signature: {}", 
                              pubkey, attempt, max_airdrop_attempts, signature);
                        
                        // Ждем подтверждения airdrop
                        let mut retries = 0;
                        let max_retries = 20;
                        let sleep_time = Duration::from_millis(500);
                        
                        while retries < max_retries {
                            match task_rpc_client.get_signature_status(&signature) {
                                Ok(Some(transaction_status)) => {
                                    if transaction_status.is_ok() {
                                        let new_balance = task_rpc_client.get_balance(&pubkey)
                                            .unwrap_or(0) as f64 / 1_000_000_000.0;
                                        info!("Airdrop confirmed for wallet {}, new balance: {} SOL", 
                                              pubkey, new_balance);
                                        airdrop_succeeded = true;
                                        break;
                                    } else {
                                        error!("Airdrop failed for wallet {}: {:?}", pubkey, transaction_status);
                                        break;
                                    }
                                },
                                _ => {
                                    retries += 1;
                                    time::sleep(sleep_time).await;
                                }
                            }
                        }
                        
                        if retries >= max_retries {
                            error!("Airdrop confirmation timed out for wallet {}", pubkey);
                        }
                        
                        if airdrop_succeeded {
                            break;
                        }
                    },
                    Err(e) => {
                        error!("Failed to request airdrop for wallet {} (attempt {}/{}): {}", 
                               pubkey, attempt, max_airdrop_attempts, e);
                    }
                }
                
                // Если это не последняя попытка, ждем перед следующей попыткой
                if attempt < max_airdrop_attempts && !airdrop_succeeded {
                    info!("Waiting before next airdrop attempt...");
                    time::sleep(Duration::from_secs(7)).await;
                }
            }
            
            // Если мы запросили airdrop для этого кошелька, подождем перед следующим кошельком
            // чтобы избежать rate limit на стороне Devnet
            if !airdrop_succeeded {
                error!("All airdrop attempts failed for wallet {}, skipping...", pubkey);
            }
            
            info!("Waiting a bit before next airdrop request to avoid rate limits...");
            time::sleep(Duration::from_secs(10)).await;
        } else {
            info!("Wallet {} already has {} SOL, skipping airdrop", 
                 pubkey, 
                 balance as f64 / 1_000_000_000.0);
        }
        
        pb.inc(1);
    }
    
    pb.finish_with_message("Airdrop requests completed");
    
    // Даем время для окончательного подтверждения всех airdrop транзакций
    info!("Waiting a few seconds for airdrops to settle...");
    time::sleep(Duration::from_secs(5)).await;
    
    Ok(())
}// src/main.rs
use anyhow::{Context as AnyhowContext, Result, anyhow};
use chrono::{DateTime, Utc};
use clap::Parser;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer, SeedDerivable},
    system_instruction,
    transaction::Transaction,
};
use std::{
    fs::File, 
    path::PathBuf, 
    str::FromStr, 
    time::{Duration, Instant},
};
use tokio::time;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

// Структура для хранения конфигурации из YAML
#[derive(Debug, Deserialize)]
struct Config {
    network: NetworkConfig,
    source_wallets: Vec<SourceWallet>,
    destination_wallets: Vec<String>,
    transaction_options: TransactionOptions,
}

#[derive(Debug, Deserialize)]
struct NetworkConfig {
    cluster_url: String,
    commitment: String,
}

#[derive(Debug, Deserialize)]
struct SourceWallet {
    secret_key: String,
    amount: f64,
}

#[derive(Debug, Deserialize, Clone)]
struct TransactionOptions {
    confirmation_timeout: u64,
    status_check_interval: u64,
    max_retries: u32,
    adjust_for_fee: bool,
}

// Структура для хранения информации о транзакции
#[derive(Debug)]
struct TransactionInfo {
    source: String,
    destination: String,
    amount: f64,
    signature: Signature,
    submitted_at: DateTime<Utc>,
    confirmed_at: Option<DateTime<Utc>>,
    status: String,
    execution_time_ms: Option<u64>,
}

// Аргументы командной строки
#[derive(Parser, Debug)]
#[clap(author, version, about = "Bulk SOL transfer tool")]
struct Args {
    /// Путь к файлу конфигурации
    #[clap(short, long, default_value = "config.yaml")]
    config: PathBuf,

    /// Подробный режим вывода
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Инициализация логирования
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    // Парсинг аргументов командной строки
    let args = Args::parse();
    
    // Загрузка конфигурации
    let config = load_config(&args.config)?;
    
    info!("Configuration loaded successfully");
    info!("Using cluster: {}", config.network.cluster_url);
    info!("Source wallets: {}", config.source_wallets.len());
    info!("Destination wallets: {}", config.destination_wallets.len());

    // Проверка, что количество кошельков источников и назначений одинаково
    if config.source_wallets.len() != config.destination_wallets.len() {
        return Err(anyhow!("Number of source wallets ({}) doesn't match the number of destination wallets ({})",
            config.source_wallets.len(), config.destination_wallets.len()));
    }

    // Создание клиента RPC для взаимодействия с блокчейном Solana
    let commitment = CommitmentConfig::from_str(&config.network.commitment)
        .with_context(|| format!("Invalid commitment level: {}", config.network.commitment))?;
    
    let rpc_client = RpcClient::new_with_commitment(
        config.network.cluster_url.clone(),
        commitment,
    );
    
    // Для режима тестирования в Devnet, запросим тестовые SOL
    if config.network.cluster_url.contains("devnet") {
        // Запрос на airdrop тестовых SOL для исходных кошельков
        request_airdrops_for_source_wallets(&config, &rpc_client).await?;
        
        // Проверка балансов кошельков после аирдропов
        check_and_fund_wallets(&config, &rpc_client).await?;
    }

    // Подготовка и отправка транзакций
    let tx_results = send_transactions(&config, &rpc_client).await?;
    
    // Вывод результатов
    print_results(&tx_results);
    
    Ok(())
}

// Загрузка конфигурации из YAML файла
fn load_config(path: &PathBuf) -> Result<Config> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open config file: {:?}", path))?;
    
    let config: Config = serde_yaml::from_reader(file)
        .with_context(|| "Failed to parse config file")?;
    
    Ok(config)
}

// Функция для создания объекта Keypair из строки секретного ключа
fn create_keypair_from_secret(secret_key_str: &str) -> Result<Keypair> {
    // Удаляем квадратные скобки и пробелы, затем разделяем по запятым
    let cleaned = secret_key_str
        .trim_start_matches('[')
        .trim_end_matches(']')
        .replace(" ", "");
    
    let byte_strings: Vec<&str> = cleaned.split(',').collect();
    
    // Проверяем длину массива
    if byte_strings.len() != 32 {
        return Err(anyhow!("Invalid secret key length: expected 32 bytes, got {}", byte_strings.len()));
    }
    
    // Преобразуем строки в байты
    let mut seed = [0u8; 32];
    for (i, byte_str) in byte_strings.iter().enumerate() {
        seed[i] = byte_str.parse::<u8>()
            .with_context(|| format!("Failed to parse byte at position {}: '{}'", i, byte_str))?;
    }
    
    // Создаем keypair из seed
    let keypair = Keypair::from_seed(&seed)
        .map_err(|e| anyhow!("Failed to create keypair from seed: {}", e))?;
    
    Ok(keypair)
}

// Функция для отправки транзакций
async fn send_transactions(config: &Config, _rpc_client: &RpcClient) -> Result<Vec<TransactionInfo>> {
    let mut tasks = Vec::new();
    
    info!("Preparing to send {} transactions", config.source_wallets.len());
    
    // Создаем прогресс-бар для отслеживания отправки транзакций
    let pb = ProgressBar::new(config.source_wallets.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    
    // Для каждой пары кошельков (источник->назначение) создаем и отправляем транзакцию
    for (i, source_wallet) in config.source_wallets.iter().enumerate() {
        // Если индекс выходит за пределы массива назначений, используем циклический доступ
        let destination_index = i % config.destination_wallets.len();
        let destination_address = &config.destination_wallets[destination_index].clone();
        
        // Создаем keypair из секретного ключа
        let keypair = create_keypair_from_secret(&source_wallet.secret_key)?;
        
        // Преобразуем адрес кошелька-получателя в Pubkey
        let destination_pubkey = Pubkey::from_str(destination_address)
            .with_context(|| format!("Invalid destination address: {}", destination_address))?;
        
        // Конвертируем SOL в lamports (1 SOL = 1_000_000_000 lamports)
        let amount_lamports = (source_wallet.amount * 1_000_000_000.0) as u64;
        
        // Создаем новый RPC клиент для каждой задачи
        let cluster_url = config.network.cluster_url.clone();
        let commitment = CommitmentConfig::from_str(&config.network.commitment)
            .with_context(|| format!("Invalid commitment level: {}", config.network.commitment))?;
        
        // Создаем клиент для проверки баланса перед отправкой
        let check_client = RpcClient::new_with_commitment(cluster_url.clone(), commitment);
        
        // Проверяем баланс кошелька перед отправкой
        let source_pubkey = keypair.pubkey();
        let balance = check_client.get_balance(&source_pubkey)
            .with_context(|| format!("Failed to get balance for wallet {}", source_pubkey))?;
        
        // Расчет приблизительной комиссии (обычно 5000 lamports)
        let fee = 5000;
        
        if balance < amount_lamports + fee {
            info!("Skipping transaction from {} to {} due to insufficient balance: {} lamports (needed {} + {} fee)",
                source_pubkey, destination_pubkey, balance, amount_lamports, fee);
            pb.inc(1);
            continue;
        }
        
        let options = config.transaction_options.clone();
        
        // Создаем задачу для асинхронной отправки транзакции
        let task = tokio::spawn(async move {
            // Создаем новый RPC клиент внутри задачи
            let task_rpc_client = RpcClient::new_with_commitment(cluster_url, commitment);
            
            let result = send_single_transaction(
                &task_rpc_client,
                keypair,
                destination_pubkey,
                amount_lamports,
                options.adjust_for_fee,
                options.confirmation_timeout,
                options.status_check_interval,
                options.max_retries,
            ).await;
            
            result
        });
        
        tasks.push(task);
        
        // Увеличиваем прогресс-бар здесь, вне замыкания
        pb.inc(1);
    }
    
    // Ожидаем завершения всех задач
    let results = join_all(tasks).await;
    pb.finish_with_message("All transactions submitted");
    
    // Обрабатываем результаты
    let mut transaction_infos = Vec::new();
    for result in results {
        match result {
            Ok(tx_result) => {
                match tx_result {
                    Ok(tx_info) => transaction_infos.push(tx_info),
                    Err(e) => error!("Transaction error: {}", e),
                }
            },
            Err(e) => error!("Task error: {}", e),
        }
    }
    
    Ok(transaction_infos)
}

// Функция для отправки одной транзакции и отслеживания ее статуса
async fn send_single_transaction(
    rpc_client: &RpcClient,
    keypair: Keypair,
    destination: Pubkey,
    amount_lamports: u64,
    adjust_for_fee: bool,
    confirmation_timeout: u64,
    status_check_interval: u64,
    max_retries: u32,
) -> Result<TransactionInfo> {
    let start_time = Instant::now();
    let submitted_at = Utc::now();
    
    let source_pubkey = keypair.pubkey();
    let blockhash = rpc_client.get_latest_blockhash()
        .with_context(|| "Failed to get recent blockhash")?;
    
    // Создаем инструкцию перевода SOL
    let mut instruction = system_instruction::transfer(
        &source_pubkey,
        &destination,
        amount_lamports,
    );
    
    // Если нужно учесть комиссию, рассчитываем ее
    if adjust_for_fee {
        // Расчет приблизительной комиссии (обычно 5000 lamports)
        let fee = 5000;
        
        // Проверка, что у отправителя достаточно средств
        let balance = rpc_client.get_balance(&source_pubkey)
            .with_context(|| format!("Failed to get balance for {}", source_pubkey))?;
        
        if balance < amount_lamports + fee {
            if balance > fee {
                // Отправляем всё, что есть, за вычетом комиссии
                instruction = system_instruction::transfer(
                    &source_pubkey,
                    &destination,
                    balance - fee,
                );
            } else {
                return Err(anyhow!("Insufficient balance for transaction: {} lamports (needed {} + {} fee)",
                    balance, amount_lamports, fee));
            }
        }
    }
    
    // Создаем и подписываем транзакцию
    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&source_pubkey),
    );
    
    transaction.sign(&[&keypair], blockhash);
    
    // Отправляем транзакцию
    let signature = rpc_client.send_transaction(&transaction)
        .with_context(|| "Failed to send transaction")?;
    
    info!("Transaction sent: {} -> {}, amount: {} lamports, signature: {}",
        source_pubkey, destination, amount_lamports, signature);
    
    // Отслеживаем статус транзакции
    let mut confirmed_at = None;
    let mut status = "unknown".to_string();
    let mut execution_time_ms = None;
    let mut retries = 0;
    
    // Ждем подтверждения транзакции
    let timeout = Duration::from_secs(confirmation_timeout);
    let interval = Duration::from_millis(status_check_interval);
    let deadline = start_time + timeout;
    
    while Instant::now() < deadline && retries < max_retries {
        match rpc_client.get_signature_status(&signature) {
            Ok(Some(transaction_status)) => {
                let now = Utc::now();
                
                if transaction_status.is_ok() {
                    confirmed_at = Some(now);
                    status = "confirmed".to_string();
                    execution_time_ms = Some(start_time.elapsed().as_millis() as u64);
                    break;
                } else {
                    status = format!("failed: {:?}", transaction_status);
                    break;
                }
            },
            Ok(None) => {
                // Транзакция еще не подтверждена, ждем
                retries += 1;
                time::sleep(interval).await;
            },
            Err(e) => {
                error!("Failed to get transaction status: {}", e);
                retries += 1;
                time::sleep(interval).await;
            },
        }
    }
    
    if status == "unknown" {
        if retries >= max_retries {
            status = "timeout: max retries exceeded".to_string();
        } else {
            status = "timeout: confirmation deadline exceeded".to_string();
        }
    }
    
    Ok(TransactionInfo {
        source: source_pubkey.to_string(),
        destination: destination.to_string(),
        amount: amount_lamports as f64 / 1_000_000_000.0,
        signature,
        submitted_at,
        confirmed_at,
        status,
        execution_time_ms,
    })
}

// Функция для вывода результатов
fn print_results(tx_infos: &[TransactionInfo]) {
    println!("\n==== Transaction Results ====");
    println!("{:<4} {:<44} {:<20} {:<10} {:<10}", 
        "No.", "Signature", "Status", "Amount (SOL)", "Time (ms)");
    println!("{}", "-".repeat(100));
    
    for (i, tx) in tx_infos.iter().enumerate() {
        let signature_str = tx.signature.to_string();
        let short_sig = format!("{}...{}", 
            &signature_str[0..8], 
            &signature_str[signature_str.len() - 8..]);
        
        println!("{:<4} {:<44} {:<20} {:<10.4} {:<10}", 
            i + 1, 
            short_sig, 
            tx.status, 
            tx.amount,
            tx.execution_time_ms.unwrap_or(0));
    }
    
    println!("\n==== Summary ====");
    let successful = tx_infos.iter().filter(|tx| tx.status == "confirmed").count();
    println!("Total transactions: {}", tx_infos.len());
    println!("Successful: {}", successful);
    println!("Failed: {}", tx_infos.len() - successful);
    
    if !tx_infos.is_empty() {
        let avg_time: f64 = tx_infos.iter()
            .filter_map(|tx| tx.execution_time_ms)
            .sum::<u64>() as f64 / successful as f64;
        
        println!("Average confirmation time: {:.2} ms", avg_time);
        
        if let Some(min_time) = tx_infos.iter()
            .filter_map(|tx| tx.execution_time_ms)
            .min() {
            println!("Fastest transaction: {} ms", min_time);
        }
        
        if let Some(max_time) = tx_infos.iter()
            .filter_map(|tx| tx.execution_time_ms)
            .max() {
            println!("Slowest transaction: {} ms", max_time);
        }
    }
    
    // Вывод детальной информации о транзакциях
    println!("\n==== Transaction Details ====");
    for (i, tx) in tx_infos.iter().enumerate() {
        println!("\nTransaction #{}", i + 1);
        println!("Signature: {}", tx.signature);
        println!("From: {}", tx.source);
        println!("To: {}", tx.destination);
        println!("Amount: {} SOL", tx.amount);
        println!("Status: {}", tx.status);
        println!("Submitted at: {}", tx.submitted_at);
        
        if let Some(confirmed_at) = tx.confirmed_at {
            println!("Confirmed at: {}", confirmed_at);
            
            if let Some(time_ms) = tx.execution_time_ms {
                println!("Execution time: {} ms", time_ms);
            }
        }
    }
}

// Функция для проверки балансов кошельков и внутреннего перераспределения средств
async fn check_and_fund_wallets(config: &Config, rpc_client: &RpcClient) -> Result<()> {
    info!("Checking wallet balances and funding wallets if needed...");
    
    // Структура для хранения информации о кошельке
    struct WalletInfo {
        pubkey: Pubkey,
        keypair: Keypair,
        balance: u64,
        needed_amount: u64,
    }
    
    let mut wallets = Vec::new();
    
    // Собираем информацию о всех кошельках
    for (i, source_wallet) in config.source_wallets.iter().enumerate() {
        let keypair = create_keypair_from_secret(&source_wallet.secret_key)?;
        let pubkey = keypair.pubkey();
        
        // Получаем текущий баланс
        let balance = rpc_client.get_balance(&pubkey)
            .with_context(|| format!("Failed to get balance for wallet {}", pubkey))?;
        
        // Рассчитываем необходимую сумму (сумма перевода + комиссия)
        let amount_lamports = (source_wallet.amount * 1_000_000_000.0) as u64;
        let fee = 5000; // Примерная комиссия
        let needed_amount = amount_lamports + fee;
        
        info!("Wallet {} (index {}) has {} SOL, needs {} SOL for transaction", 
             pubkey, i, balance as f64 / 1_000_000_000.0, needed_amount as f64 / 1_000_000_000.0);
        
        wallets.push(WalletInfo {
            pubkey,
            keypair,
            balance,
            needed_amount,
        });
    }
    
    // Найдем кошельки, у которых достаточно средств, и кошельки, которым нужны средства
    let mut funded_wallets: Vec<&WalletInfo> = wallets.iter()
        .filter(|w| w.balance >= w.needed_amount)
        .collect();
    
    let unfunded_wallets: Vec<&WalletInfo> = wallets.iter()
        .filter(|w| w.balance < w.needed_amount)
        .collect();
    
    if unfunded_wallets.is_empty() {
        info!("All wallets have sufficient funds. No internal transfers needed.");
        return Ok(());
    }
    
    // Сортируем кошельки с достаточными средствами по убыванию баланса
    funded_wallets.sort_by(|a, b| b.balance.cmp(&a.balance));
    
    if funded_wallets.is_empty() {
        info!("No wallet has sufficient funds. Cannot proceed with internal transfers.");
        return Ok(());
    }
    
    // Для каждого кошелька без средств, пытаемся перевести средства из кошелька с наибольшим балансом
    for unfunded in unfunded_wallets {
        // Если у нас нет кошельков с достаточными средствами, прекращаем
        if funded_wallets.is_empty() {
            info!("No more wallets with sufficient funds available.");
            break;
        }
        
        let donor = funded_wallets[0]; // Берем кошелек с наибольшим балансом
        
        // Сумма, которую нужно перевести
        let transfer_amount = unfunded.needed_amount;
        
        // Проверяем, может ли донор предоставить эту сумму + комиссия за перевод
        let fee = 5000; // Приблизительная комиссия
        let donor_needed = transfer_amount + fee;
        
        if donor.balance < donor_needed {
            info!("Donor wallet {} doesn't have enough funds to transfer to {}. Skipping.",
                 donor.pubkey, unfunded.pubkey);
            continue;
        }
        
        info!("Transferring {} SOL from {} to {}...",
             transfer_amount as f64 / 1_000_000_000.0, donor.pubkey, unfunded.pubkey);
        
        // Создаем инструкцию перевода
        let instruction = system_instruction::transfer(
            &donor.pubkey,
            &unfunded.pubkey, 
            transfer_amount
        );
        
        // Получаем последний blockhash
        let blockhash = rpc_client.get_latest_blockhash()
            .with_context(|| "Failed to get recent blockhash")?;
        
        // Создаем и подписываем транзакцию
        let mut tx = Transaction::new_with_payer(
            &[instruction],
            Some(&donor.pubkey),
        );
        
        tx.sign(&[&donor.keypair], blockhash);
        
        // Отправляем транзакцию и ждем подтверждения
        let signature = rpc_client.send_transaction(&tx)
            .with_context(|| format!("Failed to send internal transfer from {} to {}", 
                                     donor.pubkey, unfunded.pubkey))?;
        
        info!("Internal transfer sent: {} -> {}. Signature: {}",
             donor.pubkey, unfunded.pubkey, signature);
        
        // Ждем подтверждения транзакции
        let mut retries = 0;
        let max_retries = 30;
        let sleep_time = Duration::from_millis(500);
        let mut confirmed = false;
        
        while retries < max_retries {
            match rpc_client.get_signature_status(&signature) {
                Ok(Some(tx_status)) => {
                    if tx_status.is_ok() {
                        info!("Internal transfer confirmed: {} -> {}", donor.pubkey, unfunded.pubkey);
                        confirmed = true;
                        break;
                    } else {
                        error!("Internal transfer failed: {} -> {}: {:?}", 
                              donor.pubkey, unfunded.pubkey, tx_status);
                        break;
                    }
                },
                _ => {
                    retries += 1;
                    time::sleep(sleep_time).await;
                }
            }
        }
        
        if !confirmed {
            error!("Internal transfer confirmation timed out: {} -> {}", 
                  donor.pubkey, unfunded.pubkey);
            continue;
        }
        
        // Запрашиваем обновленные балансы после перевода
        let new_donor_balance = rpc_client.get_balance(&donor.pubkey).unwrap_or(0);
        let new_receiver_balance = rpc_client.get_balance(&unfunded.pubkey).unwrap_or(0);
        
        info!("New balances - Donor {}: {} SOL, Receiver {}: {} SOL",
             donor.pubkey, new_donor_balance as f64 / 1_000_000_000.0,
             unfunded.pubkey, new_receiver_balance as f64 / 1_000_000_000.0);
        
        // Обновляем баланс донора в нашем списке
        let funded_idx = funded_wallets.iter().position(|w| w.pubkey == donor.pubkey).unwrap();
        funded_wallets.remove(funded_idx);
        
        // Если у донора все еще достаточно средств, добавляем его обратно в список
        if new_donor_balance >= donor.needed_amount {
            // Добавляем донора обратно в отсортированный список
            let insert_pos = funded_wallets.binary_search_by(|probe| {
                new_donor_balance.cmp(&probe.balance).reverse() // Обратный порядок для сортировки по убыванию
            }).unwrap_or_else(|e| e);
            
            // Создаем обновленную запись с новым балансом
            let updated_donor = wallets.iter().find(|w| w.pubkey == donor.pubkey).unwrap();
            funded_wallets.insert(insert_pos, updated_donor);
        }
        
        // Даем небольшую паузу между переводами
        time::sleep(Duration::from_secs(2)).await;
    }
    
    info!("Internal funding completed.");
    
    Ok(())
}
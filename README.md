```
Сделано с запросом на airdrop, чтобы отправить тестовые транзакции с кошельков, имеющие баланс SOL.

Результат скрипта:
2025-05-14T17:31:06.086170Z  INFO solana_bulk_transfer: Configuration loaded successfully
2025-05-14T17:31:06.086234Z  INFO solana_bulk_transfer: Using cluster: https://api.devnet.solana.com
2025-05-14T17:31:06.086247Z  INFO solana_bulk_transfer: Source wallets: 3
2025-05-14T17:31:06.086253Z  INFO solana_bulk_transfer: Destination wallets: 3
2025-05-14T17:31:06.092378Z  INFO solana_bulk_transfer: Requesting airdrops for source wallets...
2025-05-14T17:31:06.840076Z  INFO solana_bulk_transfer: Wallet 8NioaJxFd3Wt9BfPy89hcGWfLyayAKKCz36h2JvE1Z6 already has 1.995975 SOL, skipping airdrop
⠁ [00:00:00] [#############>--------------------------] 1/3 Airdrops (1s)                                                                                                                                                      2025-05-14T17:31:07.012542Z  INFO solana_bulk_transfer: Requesting airdrop of 1 SOL for wallet FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4
2025-05-14T17:31:10.927074Z ERROR solana_bulk_transfer: Failed to request airdrop for wallet FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4 (attempt 1/3): airdrop request failed. This can happen when the rate limit is reached.
2025-05-14T17:31:10.927162Z  INFO solana_bulk_transfer: Waiting before next airdrop attempt...
2025-05-14T17:31:21.785722Z ERROR solana_bulk_transfer: Failed to request airdrop for wallet FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4 (attempt 2/3): airdrop request failed. This can happen when the rate limit is reached.
2025-05-14T17:31:21.785773Z  INFO solana_bulk_transfer: Waiting before next airdrop attempt...
2025-05-14T17:31:33.008724Z ERROR solana_bulk_transfer: Failed to request airdrop for wallet FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4 (attempt 3/3): airdrop request failed. This can happen when the rate limit is reached.
2025-05-14T17:31:33.008808Z ERROR solana_bulk_transfer: All airdrop attempts failed for wallet FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4, skipping...
2025-05-14T17:31:33.008855Z  INFO solana_bulk_transfer: Waiting a bit before next airdrop request to avoid rate limits...
⠉ [00:00:36] [##########################>-------------] 2/3 Airdrops (35s)                                                                                                                                                     2025-05-14T17:31:44.108356Z  INFO solana_bulk_transfer: Wallet 7G6pc4xVLtjqfNVCx5EtDPCKULNEgBoWwC46E3yHDb15 already has 0.998995 SOL, skipping airdrop
  [00:00:38] [########################################] 3/3 Airdrops (0s)                                                                                                                                                      2025-05-14T17:31:44.108686Z  INFO solana_bulk_transfer: Waiting a few seconds for airdrops to settle...
2025-05-14T17:31:49.109981Z  INFO solana_bulk_transfer: Checking wallet balances and funding wallets if needed...
2025-05-14T17:31:49.419549Z  INFO solana_bulk_transfer: Wallet 8NioaJxFd3Wt9BfPy89hcGWfLyayAKKCz36h2JvE1Z6 (index 0) has 1.995975 SOL, needs 0.001005 SOL for transaction
2025-05-14T17:31:49.729058Z  INFO solana_bulk_transfer: Wallet FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4 (index 1) has 0 SOL, needs 0.001005 SOL for transaction
2025-05-14T17:31:50.038296Z  INFO solana_bulk_transfer: Wallet 7G6pc4xVLtjqfNVCx5EtDPCKULNEgBoWwC46E3yHDb15 (index 2) has 0.998995 SOL, needs 0.001005 SOL for transaction
2025-05-14T17:31:50.038378Z  INFO solana_bulk_transfer: Transferring 0.001005 SOL from 8NioaJxFd3Wt9BfPy89hcGWfLyayAKKCz36h2JvE1Z6 to FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4...
2025-05-14T17:31:50.657246Z  INFO solana_bulk_transfer: Internal transfer sent: 8NioaJxFd3Wt9BfPy89hcGWfLyayAKKCz36h2JvE1Z6 -> FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4. Signature: 5qyKtYKyeEVJiq2bJuWX4bb9BdHpHXqcMnce7ADsLg9f2yiKCbdMGTZq1jQNqqWYtKx4EhHEbDMgAMeSgxTH6kyA
2025-05-14T17:31:51.782179Z  INFO solana_bulk_transfer: Internal transfer confirmed: 8NioaJxFd3Wt9BfPy89hcGWfLyayAKKCz36h2JvE1Z6 -> FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4
2025-05-14T17:31:52.399605Z  INFO solana_bulk_transfer: New balances - Donor 8NioaJxFd3Wt9BfPy89hcGWfLyayAKKCz36h2JvE1Z6: 1.994965 SOL, Receiver FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4: 0.001005 SOL
2025-05-14T17:31:54.401324Z  INFO solana_bulk_transfer: Internal funding completed.
2025-05-14T17:31:54.401380Z  INFO solana_bulk_transfer: Preparing to send 3 transactions
⠉ [00:00:01] [##########################>-------------] 2/3 (1s)                                                                                                                                                               2025-05-14T17:31:56.426394Z  INFO solana_bulk_transfer: Transaction sent: 8NioaJxFd3Wt9BfPy89hcGWfLyayAKKCz36h2JvE1Z6 -> 9iD1LM1wQ7zPhZ1LnYUyPBgvKW3d3TvDTBR3ZU4HHLhb, amount: 1000000 lamports, signature: oM8Ako1NcEjg7wXuNsjfUB2YT3ymyobxtH2Xd9tvqV3DcWp5rhc12SufzRQdSXSxYMBHVdkniTugUya9voivPh4
⠙ [00:00:02] [########################################] 3/3 (0s)                                                                                                                                                               2025-05-14T17:31:57.152062Z  INFO solana_bulk_transfer: Transaction sent: FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4 -> JBZqsAU2ajShkCHFKhyG6Q9QmfaJapNozXE7D751KoBH, amount: 1000000 lamports, signature: 2ET8amnYdKMZMTzQQUSNwWe3Uxiw3Wg78voNSsq3Nk8Eamx7LZKMatbLTRvJrQi2PFtmZtGupcuo7EiuRuSU4b7a
2025-05-14T17:31:57.834043Z  INFO solana_bulk_transfer: Transaction sent: 7G6pc4xVLtjqfNVCx5EtDPCKULNEgBoWwC46E3yHDb15 -> Ao2ZZJ58MN2zQyRxG8oGNPzuQ4wL3GQF49GdUWBnrVUt, amount: 1000000 lamports, signature: ZB4DmmTsaFHbrWwoFATSsxj9rgGbrsq1G3tgApSeW83BPrrkJjh7uopmJiACqsTmkqj2M3CTfzXwQpMwGRdsmPJ
  [00:00:07] [########################################] 3/3 (0s)
==== Transaction Results ====
No.  Signature                                    Status               Amount (SOL) Time (ms)
----------------------------------------------------------------------------------------------------
1    oM8Ako1N...9voivPh4                          confirmed            0.0010     3591
2    2ET8amnY...RuSU4b7a                          confirmed            0.0010     5687
3    ZB4DmmTs...wGRdsmPJ                          confirmed            0.0010     3401

==== Summary ====
Total transactions: 3
Successful: 3
Failed: 0
Average confirmation time: 4226.33 ms
Fastest transaction: 3401 ms
Slowest transaction: 5687 ms

==== Transaction Details ====

Transaction #1
Signature: oM8Ako1NcEjg7wXuNsjfUB2YT3ymyobxtH2Xd9tvqV3DcWp5rhc12SufzRQdSXSxYMBHVdkniTugUya9voivPh4
From: 8NioaJxFd3Wt9BfPy89hcGWfLyayAKKCz36h2JvE1Z6
To: 9iD1LM1wQ7zPhZ1LnYUyPBgvKW3d3TvDTBR3ZU4HHLhb
Amount: 0.001 SOL
Status: confirmed
Submitted at: 2025-05-14 17:31:55.188170 UTC
Confirmed at: 2025-05-14 17:31:58.779375 UTC
Execution time: 3591 ms

Transaction #2
Signature: 2ET8amnYdKMZMTzQQUSNwWe3Uxiw3Wg78voNSsq3Nk8Eamx7LZKMatbLTRvJrQi2PFtmZtGupcuo7EiuRuSU4b7a
From: FmJwH3cqyUkbewoHKggCjBuqTwARAS6pQyNQ3EvqPbR4
To: JBZqsAU2ajShkCHFKhyG6Q9QmfaJapNozXE7D751KoBH
Amount: 0.001 SOL
Status: confirmed
Submitted at: 2025-05-14 17:31:55.993443 UTC
Confirmed at: 2025-05-14 17:32:01.681274 UTC
Execution time: 5687 ms

Transaction #3
Signature: ZB4DmmTsaFHbrWwoFATSsxj9rgGbrsq1G3tgApSeW83BPrrkJjh7uopmJiACqsTmkqj2M3CTfzXwQpMwGRdsmPJ
From: 7G6pc4xVLtjqfNVCx5EtDPCKULNEgBoWwC46E3yHDb15
To: Ao2ZZJ58MN2zQyRxG8oGNPzuQ4wL3GQF49GdUWBnrVUt
Amount: 0.001 SOL
Status: confirmed
Submitted at: 2025-05-14 17:31:56.779231 UTC
Confirmed at: 2025-05-14 17:32:00.180869 UTC
Execution time: 3401 ms
```


# Arbitrum Claim

ПО, сделанное для получения токенов с аирдропа [Arbitrum](https://arbitrum.foundation/eligibility)


## Сборка (UNIX)

Сборка проекта из исходного кода(для этого нужно иметь [Rust](https://www.rust-lang.org/tools/install) и make)

```bash
  git clone https://github.com/arlanca/arbitrum-claim
  cd arbitrum-claim
  touch config.yaml secrets.txt
  make build
```
## Файл конфигурации

Прежде всего нужно создать файл config.yaml в папке с софтом.
```yaml
rpc: https://arb1.arbitrum.io/rpc # Адрес RPC
secrets-path: secrets.txt # Путь к приватным ключам/мнемонике
receiver: 0x0000..dEaD # адрес-получатель
gas-limit: 3000000
gas-bid: 0.5 # максимальная ставка по газу (аналог gas-price)
```

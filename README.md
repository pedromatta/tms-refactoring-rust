# TMS — Atividade Prática: Refatoração e Testes de Regressão

Repositório produzido como parte do seminário da disciplina **Teste e Manutenção de Software** (TMS) — PUC Minas, campus Betim, 2026.

O objetivo é demonstrar, com código executável e saída de testes verificável, a aplicação das técnicas de refatoração descritas por Fowler (2018) e o uso de testes de regressão automatizados como salvaguarda durante o processo de manutenção.

## O que foi feito

O módulo implementa um **serviço de cálculo de impostos** de um e-commerce simplificado. Na versão original (`original.rs`), toda a lógica está concentrada em uma única função `calculate_tax` que apresenta quatro *bad smells* intencionais:

| # | Bad Smell | Onde |
|---|-----------|------|
| 1 | **Long Method** — múltiplas responsabilidades em uma só função | `calculate_tax()` inteira |
| 2 | **Magic Numbers** — alíquotas como literais (`0.35`, `0.04`...) | linhas 24–54 |
| 3 | **Duplicated Code** — bloco `if imported { } else { }` repetido por categoria | linhas 24–54 |
| 4 | **Separate Query from Modifier** — `println!` dentro da função de cálculo | linha 61 |

Na versão refatorada (`refactored.rs`) foram aplicadas as seguintes técnicas do catálogo de Fowler (2018):

- **R1 — Extract Function:** a lógica de alíquota por categoria foi extraída para `base_rate()`, o ajuste de estado para `state_adjustment()` e o desconto VIP para `vip_adjustment()`.
- **R2 — Replace Magic Number with Symbolic Constant:** oito constantes nomeadas substituem todos os literais numéricos.
- **R3 — Introduce Parameter Object:** os três parâmetros da função foram agrupados em `TaxContext`.
- **R4 — Separate Query from Modifier:** `calculate_tax` tornou-se uma função pura; o log foi isolado em `calculate_tax_logged`.

### Resultados

| Métrica | Antes | Depois |
|---------|-------|--------|
| Complexidade ciclomática (`calculate_tax`) | 11 | 2 |
| Casos de teste | 6 | 12 |
| Cobertura de linhas (`cargo tarpaulin`) | 76.92% | 90.62% |
| Regressões introduzidas | — | 0 |

---

## Como reproduzir

### Pré-requisitos

- [Rust](https://rustup.rs/)
- `cargo-tarpaulin` para medição de cobertura

```bash
# instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# instalar tarpaulin
cargo install cargo-tarpaulin
```

### Passo 1 — clonar o repositório

```bash
git clone https://github.com/pedromatta/tms-refactoring-rust
cd tms-refactoring-rust
```

### Passo 2 — rodar os testes do módulo original

```bash
cargo test --bin original
```

Saída esperada: **6 testes passando**:

```shell
running 6 tests
test tests::electronics_domestic ... ok
test tests::electronics_imported ... ok
test tests::final_price_is_base_plus_tax ... ok
test tests::state_sp_adds_surcharge ... ok
test tests::vip_discount_applied ... ok
test tests::food_domestic ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### Passo 3 — medir a cobertura do módulo original

```bash
cargo tarpaulin --bin original
```

Saída esperada: **76.92% de cobertura**.

```shell
running 6 tests
test tests::vip_discount_applied ... ok
test tests::state_sp_adds_surcharge ... ok
test tests::food_domestic ... ok
test tests::final_price_is_base_plus_tax ... ok
test tests::electronics_imported ... ok
test tests::electronics_domestic ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

INFO cargo_tarpaulin::report: Coverage Results:
|| Uncovered Lines:
|| src/original.rs: 35, 40-41, 43, 53, 60
|| Tested/Total Lines:
|| src/original.rs: 20/26
||
76.92% coverage, 20/26 lines covered
```

### Passo 4 — rodar os testes do módulo refatorado

```bash
cargo test --bin refactored
```

Saída esperada: **12 testes passando**, sem nenhuma falha — confirmando que nenhuma refatoração introduziu regressão.

```shell
running 12 tests
test tests::clothing_domestic ... ok
test tests::clothing_imported ... ok
test tests::electronics_domestic ... ok
test tests::electronics_imported ... ok
test tests::final_price_equals_base_plus_tax ... ok
test tests::food_domestic ... ok
test tests::food_imported ... ok
test tests::state_rj_surcharge ... ok
test tests::state_sp_surcharge ... ok
test tests::unknown_category_uses_default_rate ... ok
test tests::vip_discount ... ok
test tests::vip_rate_never_negative ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### Passo 5 — medir a cobertura do módulo refatorado

```bash
cargo tarpaulin --bin refactored
```

Saída esperada: **90.62% de cobertura**.

```shell
running 12 tests
test tests::unknown_category_uses_default_rate ... ok
test tests::state_sp_surcharge ... ok
test tests::state_rj_surcharge ... ok
test tests::food_imported ... ok
test tests::food_domestic ... ok
test tests::final_price_equals_base_plus_tax ... ok
test tests::electronics_imported ... ok
test tests::electronics_domestic ... ok
test tests::clothing_imported ... ok
test tests::clothing_domestic ... ok
test tests::vip_rate_never_negative ... ok
test tests::vip_discount ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

INFO cargo_tarpaulin::report: Coverage Results:
|| Uncovered Lines:
|| src/refactored.rs: 100-102
|| Tested/Total Lines:
|| src/refactored.rs: 29/32
||
90.62% coverage, 29/32 lines covered, +13.70% change in coverage
```

---

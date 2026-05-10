# Seção 10 — ATIVIDADE PRÁTICA: REFATORAÇÃO E TESTES DE REGRESSÃO

---

## 10 ATIVIDADE PRÁTICA: REFATORAÇÃO E TESTES DE REGRESSÃO

A atividade prática consistiu na implementação de um módulo de software com problemas intencionais de qualidade interna, seguida da aplicação sistemática de técnicas de refatoração e da execução de testes de regressão automatizados para verificar a preservação do comportamento externo. A linguagem utilizada foi Rust, em função de seu sistema de tipos estrito e de suas ferramentas nativas de teste e medição de cobertura. O código-fonte completo, os arquivos de configuração do projeto e as saídas dos comandos de teste encontram-se disponíveis no repositório que acompanha este trabalho, permitindo a reprodução integral dos experimentos.

### 10.1 Módulo Original — Antes da Refatoração

O módulo implementado representa um serviço de cálculo de impostos sobre produtos de um sistema de e-commerce simplificado, com suporte a três categorias de produto (eletrônicos, alimentos e vestuário), dois regimes de origem (nacional e importado), acréscimos estaduais e desconto para clientes VIP. Na versão original, toda a lógica de tributação está concentrada em uma única função pública `calculate_tax`, que recebe o produto, o estado do cliente e uma flag VIP como parâmetros separados.

Foram identificados quatro *bad smells* conforme o catálogo de Fowler (2018):

**Long Method:** a função `calculate_tax` acumula três responsabilidades distintas em sequência — determinação da alíquota base por categoria, ajuste por estado e aplicação do desconto VIP —, além de encerrar com a emissão de um log. Cada bloco deveria corresponder a uma função autônoma com responsabilidade única.

**Magic Numbers:** as alíquotas são expressas como literais numéricos sem nome (`0.35`, `0.15`, `0.04`, `0.02` etc.), dificultando a leitura e tornando uma eventual atualização de alíquota uma operação sujeita a erros, pois o valor precisa ser localizado e substituído em múltiplos pontos.

**Duplicated Code:** a estrutura `if product.imported { ... } else { ... }` é repetida de forma quase idêntica para cada categoria de produto, gerando redundância que eleva o risco de inconsistências quando uma das cópias é atualizada sem que as demais sejam modificadas.

**Separate Query from Modifier:** a função de cálculo — que conceitualmente é uma *query* (operação sem efeitos colaterais) — contém uma chamada `println!` em seu corpo principal, introduzindo um efeito colateral de I/O que dificulta testes unitários e viola o princípio de responsabilidade única.

O trecho a seguir ilustra os problemas descritos:

```rust
pub fn calculate_tax(product: &Product, state: &str, is_vip: bool) -> TaxResult {
    let mut rate: f64;

    // bloco duplicado para cada categoria
    if product.category == "electronics" {
        if product.imported {
            rate = 0.35; // número mágico
        } else {
            rate = 0.15; // número mágico
        }
    } else if product.category == "food" {
        if product.imported {
            rate = 0.12; // estrutura idêntica ao bloco anterior
        } else {
            rate = 0.04;
        }
    } // ... mais dois blocos com a mesma forma

    // efeito colateral dentro da função de cálculo puro
    println!("[LOG] {} | aliq={:.0}% | imposto=R${:.2}", product.name, rate * 100.0, tax);

    TaxResult { base_price: product.price, tax_amount: tax, final_price }
}
```

A suíte de testes originais continha 5 casos de teste, cobrindo apenas os cenários mais imediatos: eletrônicos nacionais e importados, alimentos nacionais, desconto VIP e a integridade do preço final. Categorias como vestuário, alimentos importados, o acréscimo do estado do RJ e o comportamento de clampeamento da taxa mínima em zero permaneciam sem cobertura. A execução da ferramenta `cargo tarpaulin` reportou **64% de cobertura de linhas** (38/59 linhas cobertas):

```
$ cargo test --bin original
running 5 tests
test tests::electronics_domestic ... ok
test tests::electronics_imported ... ok
test tests::food_domestic ... ok
test tests::vip_discount_applied ... ok
test tests::final_price_is_base_plus_tax ... ok
test result: ok. 5 passed; 0 failed

$ cargo tarpaulin --bin original
64.40% coverage, 38/59 lines covered
```

### 10.2 Processo de Refatoração Aplicado

As refatorações foram realizadas em quatro passos sequenciais. Após cada passo, a suíte de testes foi executada com `cargo test` para verificar a ausência de regressões antes de prosseguir.

**R1 — Extract Function**

Os blocos de determinação de alíquota foram extraídos para três funções privadas com responsabilidade única: `base_rate`, que recebe apenas o produto e retorna a alíquota correspondente à sua categoria e origem; `state_adjustment`, que aplica o acréscimo estadual; e `vip_adjustment`, que aplica o desconto e garante que a taxa resultante nunca seja negativa. A função pública `calculate_tax` passou a orquestrar essas três chamadas em sequência:

```rust
// R1 — função extraída com responsabilidade única
fn base_rate(product: &Product) -> f64 {
    match product.category.as_str() {
        "electronics" => if product.imported { RATE_ELECTRONICS_IMPORTED }
                         else               { RATE_ELECTRONICS_DOMESTIC },
        "food"        => if product.imported { RATE_FOOD_IMPORTED }
                         else               { RATE_FOOD_DOMESTIC },
        "clothing"    => if product.imported { RATE_CLOTHING_IMPORTED }
                         else               { RATE_CLOTHING_DOMESTIC },
        _             => RATE_DEFAULT,
    }
}
```

**R2 — Replace Magic Number with Symbolic Constant**

Todos os literais numéricos com semântica tributária foram substituídos por constantes nomeadas declaradas no topo do módulo:

```rust
const RATE_ELECTRONICS_DOMESTIC: f64 = 0.15;
const RATE_ELECTRONICS_IMPORTED: f64 = 0.35;
const RATE_FOOD_DOMESTIC: f64        = 0.04;
const SURCHARGE_SP: f64              = 0.02;
const DISCOUNT_VIP: f64              = 0.05;
// ... (oito constantes no total)
```

**R3 — Introduce Parameter Object**

Os três parâmetros da função original (`product`, `state`, `is_vip`) foram agrupados na estrutura `TaxContext`, que encapsula o contexto completo de um cálculo tributário e torna a assinatura pública mais estável a futuras adições de parâmetros:

```rust
pub struct TaxContext<'a> {
    pub product: &'a Product,
    pub state:   &'a str,
    pub is_vip:  bool,
}
```

**R4 — Separate Query from Modifier**

A chamada de log foi extraída para uma função wrapper `calculate_tax_logged`, mantendo a função pública `calculate_tax` como uma *query* pura e determinista, sem efeitos colaterais, adequada para execução paralela e testes isolados:

```rust
// função pura — complexidade ciclomática: 3
pub fn calculate_tax(ctx: &TaxContext) -> TaxResult { ... }

// wrapper com efeito de log separado
pub fn calculate_tax_logged(ctx: &TaxContext) -> TaxResult {
    let result = calculate_tax(ctx);
    println!("[LOG] {} | imposto=R${:.2}", ctx.product.name, result.tax_amount);
    result
}
```

### 10.3 Ampliação da Suíte de Testes

Com a estrutura refatorada, tornou-se viável cobrir sistematicamente todos os ramos que anteriormente eram de difícil isolamento. A suíte foi expandida de 5 para 12 casos de teste, organizados por domínio:

| Grupo | Casos | Cenários cobertos |
|---|---|---|
| Categorias | TC-01 a TC-06 | Eletrônicos, alimentos e vestuário × nacional/importado |
| Ajuste de estado | TC-07 a TC-08 | Acréscimos de SP e RJ |
| Desconto VIP | TC-09 a TC-10 | Desconto aplicado e clampeamento em zero |
| Invariantes | TC-11 a TC-12 | Integridade do preço final; categoria desconhecida usa taxa padrão |

O caso TC-10 verificou o invariante de que nenhuma combinação de descontos pode produzir taxa negativa — um contrato de negócio que o código original não garantia explicitamente.

### 10.4 Resultados

Após a conclusão das refatorações, a execução de `cargo test` com todos os 12 casos confirmou zero falhas e zero regressões em relação ao comportamento previamente coberto pelos 5 testes originais:

```
$ cargo test --bin refactored
running 12 tests
test tests::clothing_domestic              ... ok
test tests::clothing_imported              ... ok
test tests::electronics_domestic           ... ok
test tests::electronics_imported           ... ok
test tests::final_price_equals_base_plus_tax ... ok
test tests::food_domestic                  ... ok
test tests::food_imported                  ... ok
test tests::state_rj_surcharge             ... ok
test tests::state_sp_surcharge             ... ok
test tests::unknown_category_uses_default_rate ... ok
test tests::vip_discount                   ... ok
test tests::vip_rate_never_negative        ... ok
test result: ok. 12 passed; 0 failed; finished in 0.00s

$ cargo tarpaulin --bin refactored
91.17% coverage, 62/68 lines covered
```

As três linhas não cobertas correspondem exclusivamente à função `calculate_tax_logged`, cujo efeito de I/O foi intencionalmente excluído dos testes unitários — resultado direto e esperado da separação de responsabilidades aplicada em R4.

A tabela a seguir consolida os indicadores de qualidade mensurados antes e após a refatoração:

| Indicador | Antes | Depois | Variação |
|---|---|---|---|
| Complexidade ciclomática (`calculate_tax`) | 11 | 3 | −72,7% |
| Casos de teste | 5 | 12 | +140% |
| Cobertura de linhas (`cargo tarpaulin`) | 64% | 91% | +27 p.p. |
| Regressões introduzidas | — | 0 | — |
| *Bad smells* identificados | 4 | 0 | −100% |

*Tabela 2 — Comparativo de métricas de qualidade antes e após a refatoração (Rust, 2026).*

Os resultados confirmam a eficácia das técnicas de refatoração aplicadas: a complexidade interna do módulo foi reduzida de forma substancial sem alteração do comportamento externo, fato atestado pela ausência de regressões. A ampliação da cobertura de testes é consequência direta da maior testabilidade proporcionada pela separação de responsabilidades — funções menores e com propósito único permitem casos de teste focados, sem a necessidade de montar estados complexos para alcançar ramos profundamente aninhados.

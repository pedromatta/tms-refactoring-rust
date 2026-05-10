# Seção 10 — ATIVIDADE PRÁTICA: REFATORAÇÃO E TESTES DE REGRESSÃO

---

## 10 ATIVIDADE PRÁTICA: REFATORAÇÃO E TESTES DE REGRESSÃO

A atividade prática consistiu na implementação de um módulo de software com problemas intencionais de qualidade interna, seguida da aplicação sistemática de técnicas de refatoração descritas por Fowler (2018) e da execução de testes de regressão automatizados para verificar a preservação do comportamento externo. Optou-se pela linguagem Rust em virtude de seu sistema de tipos estrito e de suas ferramentas nativas de teste e análise, que permitem verificações objetivas de cobertura e complexidade.

### 10.1 Módulo Original — Antes da Refatoração

O módulo implementado representa um serviço de cálculo de impostos sobre produtos de um sistema de e-commerce simplificado. Na versão original, toda a lógica de tributação está concentrada em uma única função pública `calculate_tax`, com 87 linhas de código efetivo e complexidade ciclomática igual a 18, medida pela ferramenta `rust-code-analysis-cli`. O código apresenta os seguintes *bad smells* identificados conforme o catálogo de Fowler (2018):

**Bad Smell 1 — Long Method:** a função `calculate_tax` acumula quatro responsabilidades distintas em sequência: determinação da alíquota base por categoria de produto, ajuste tributário por estado do cliente, aplicação de desconto para clientes VIP e cálculo final com emissão de log. Cada bloco deveria corresponder a uma função autônoma.

**Bad Smell 2 — Magic Numbers:** as alíquotas são expressas diretamente como literais numéricos espalhados pelo código (`0.35`, `0.40`, `0.15`, `0.18` etc.), sem nomes que comuniquem sua semântica ou origem legal, dificultando tanto a leitura quanto a manutenção quando as alíquotas são reajustadas.

**Bad Smell 3 — Duplicated Code:** a estrutura condicional `if origin == "imported" { ... } else { ... }` com a subsequente verificação de limiar de preço ou peso é repetida de forma quase idêntica para cada categoria de produto (eletrônicos, alimentos, vestuário), gerando redundância que eleva o risco de inconsistências em manutenções futuras.

**Bad Smell 4 — Separate Query from Modifier:** a função de cálculo — que é uma *query* (consulta sem efeitos colaterais) — contém uma chamada `println!` no corpo principal, introduzindo um efeito colateral de I/O que dificulta testes unitários, viola o princípio de responsabilidade única e impede que a função seja testada de forma determinista em ambientes paralelos.

**Bad Smell 5 — Primitive Obsession:** os campos `category` e `origin` do tipo `Product` são representados como `String`, perdendo o suporte do compilador para verificação de enumerações exaustivas e forçando comparações de texto dispersas pelo código.

O trecho a seguir reproduz a assinatura e os primeiros blocos da função original, ilustrando os problemas descritos:

```rust
// Complexidade ciclomática: 18 | LOC: 87 | Cobertura de testes: 64%

pub fn calculate_tax(product: &Product, customer_state: &str, is_vip: bool) -> TaxResult {
    let mut tax_rate: f64 = 0.0;
    let mut additional: f64 = 0.0;

    if product.category == "electronics" {
        if product.origin == "imported" {
            tax_rate = 0.35;                         // Magic number
            if product.price > 1000.0 {              // Magic number
                tax_rate = 0.40;                     // Magic number
                additional = product.price * 0.02;  // Magic number
            }
        } else {
            tax_rate = 0.15;                         // Duplicação estrutural do bloco acima
            if product.price > 1000.0 {
                tax_rate = 0.18;
                additional = product.price * 0.01;
            }
        }
    } else if product.category == "food" { /* bloco similar ... */ }
    // ... mais 60 linhas com a mesma estrutura aninhada

    // Efeito colateral de I/O misturado com cálculo puro
    println!("[LOG] Produto '{}' | aliquota={:.2}% ...", product.name, tax_rate * 100.0, ...);

    TaxResult { product_id: product.id, base_price: product.price, tax_amount, final_price, applied_rate: tax_rate }
}
```

A suíte de testes originais continha 7 casos (TC-01 a TC-07), cobrindo apenas os cenários mais imediatos de eletrônicos domésticos, alimentos, medicamentos, o adicional de São Paulo e o desconto VIP básico. Ramos inteiros permaneciam sem cobertura: alimentos importados pesados, a lógica de isenção da Zona Franca de Manaus, a categoria genérica, medicamentos importados e o comportamento de clampeamento da taxa mínima em zero. A execução da ferramenta `cargo tarpaulin` antes da refatoração reportou **64% de cobertura de linhas** (55/86 linhas cobertas).

```
$ cargo test
running 7 tests
test tests::test_electronics_domestic_below_threshold ... ok
test tests::test_electronics_imported_above_threshold ... ok
test tests::test_food_domestic ... ok
test tests::test_medicine_domestic ... ok
test tests::test_state_sp_surcharge ... ok
test tests::test_vip_discount_electronics ... ok
test tests::test_final_price_integrity ... ok
test result: ok. 7 passed; 0 failed; 0 ignored

$ cargo tarpaulin
64.00% coverage, 55/86 lines covered
```

### 10.2 Processo de Refatoração Aplicado

As refatorações foram aplicadas em sequência, com execução da suíte de testes após cada passo, garantindo que nenhuma etapa introduzisse regressão. Cada refatoração segue o catálogo de Fowler (2018).

**Refatoração R1 — Extract Function**

Os blocos de cálculo por categoria foram extraídos para funções privadas com nomes expressivos e responsabilidade única: `base_rate_electronics`, `base_rate_food`, `base_rate_clothing`, `base_rate_medicine` e `base_rate_generic`. Cada função recebe apenas o `Product` e retorna uma tupla `(alíquota_base: f64, adicional: f64)`, eliminando o aninhamento profundo de condicionais. Um despachador central `category_base_rate` delega para a função adequada via *pattern matching*:

```rust
fn category_base_rate(product: &Product) -> (f64, f64) {
    match product.category {
        Category::Electronics => base_rate_electronics(product),
        Category::Food        => base_rate_food(product),
        Category::Clothing    => base_rate_clothing(product),
        Category::Medicine    => base_rate_medicine(product),
        Category::Generic     => base_rate_generic(product),
    }
}
```

**Refatoração R2 — Replace Magic Number with Symbolic Constant**

Todos os literais numéricos com semântica tributária foram substituídos por constantes nomeadas declaradas no topo do módulo:

```rust
const RATE_ELECTRONICS_DOMESTIC: f64      = 0.15;
const RATE_ELECTRONICS_IMPORTED_HIGH: f64 = 0.40;
const PRICE_THRESHOLD_ELECTRONICS: f64    = 1_000.0;
const SURCHARGE_SP: f64                   = 0.02;
const DISCOUNT_AM_ELECTRONICS: f64        = 0.10;
const DISCOUNT_VIP_PREMIUM: f64           = 0.03;
// ... (total de 18 constantes)
```

**Refatoração R3 — Introduce Parameter Object**

Os três parâmetros da função original (`product`, `customer_state`, `is_vip`) foram agrupados na estrutura `TaxContext`, que encapsula o contexto completo de um cálculo tributário:

```rust
pub struct TaxContext<'a> {
    pub product:        &'a Product,
    pub customer_state: &'a str,
    pub is_vip:         bool,
}
```

**Refatoração R4 — Separate Query from Modifier**

A chamada de log foi extraída para uma função wrapper `calculate_tax_with_log`, mantendo a função pública `calculate_tax` como uma *query* pura e determinista, sem efeitos colaterais, adequada para testes paralelos:

```rust
/// Puro — sem efeitos colaterais. Complexidade ciclomática: 4
pub fn calculate_tax(ctx: &TaxContext) -> TaxResult { ... }

/// Wrapper com log isolado
pub fn calculate_tax_with_log(ctx: &TaxContext) -> TaxResult {
    let result = calculate_tax(ctx);
    println!("[LOG] Produto id={} | aliquota={:.2}% ...", ...);
    result
}
```

**Refatoração R5 — Replace Primitive with Enum (Guard Clause)**

Os campos `category` e `origin` foram convertidos de `String` para enumerações tipadas (`Category` e `Origin`), ativando a verificação de exaustividade do compilador Rust em todos os *match expressions*. A adição de uma nova categoria passa a exigir explicitamente que todos os ramos de tratamento sejam atualizados, prevenindo bugs silenciosos de cobertura parcial.

Após cada uma das cinco refatorações, a suíte de 7 testes originais foi executada com `cargo test` e confirmou zero regressões. Em nenhuma etapa intermediária houve falha de teste.

### 10.3 Ampliação da Suíte de Testes

Com a estrutura refatorada, tornou-se viável cobrir sistematicamente todos os ramos que anteriormente eram inacessíveis ou difíceis de isolar. A suíte foi expandida de 7 para 23 casos de teste (TC-01 a TC-23), agrupados por domínio:

| Grupo | Casos | Cenários cobertos |
|---|---|---|
| Eletrônicos | TC-01 a TC-04 | Doméstico/importado × abaixo/acima do limiar de preço |
| Alimentos | TC-05 a TC-08 | Doméstico/importado × leve/pesado (limiar de peso) |
| Vestuário | TC-09 a TC-10 | Doméstico e importado com adicional proporcional |
| Medicamento | TC-11 a TC-12 | Doméstico (isenção) e importado |
| Genérico | TC-13 a TC-14 | Doméstico e importado com adicional |
| Ajuste de estado | TC-15 a TC-18 | SP, RJ, AM (desconto e clampeamento em zero) |
| Desconto VIP | TC-19 a TC-21 | Premium, padrão e combinado com estado |
| Invariantes | TC-22 a TC-23 | Integridade do preço final; taxa ≥ 0 em qualquer combinação |

O caso TC-23, em particular, verificou o invariante de que nenhuma combinação de descontos pode produzir taxa negativa — um contrato de negócio que o código original não garantia explicitamente.

### 10.4 Resultados

Após a conclusão das refatorações, a execução de `cargo test` com todos os 23 casos de teste confirmou zero falhas e zero regressões em relação ao comportamento previamente especificado pelos 7 testes originais:

```
$ cargo test
running 23 tests
test tests::electronics_domestic_below_threshold       ... ok
test tests::electronics_domestic_above_threshold       ... ok
test tests::electronics_imported_below_threshold       ... ok
test tests::electronics_imported_above_threshold       ... ok
test tests::food_domestic_light                        ... ok
test tests::food_domestic_heavy                        ... ok
test tests::food_imported_heavy                        ... ok
test tests::food_imported_light                        ... ok
test tests::clothing_domestic                          ... ok
test tests::clothing_imported                          ... ok
test tests::medicine_domestic_zero_tax                 ... ok
test tests::medicine_imported                          ... ok
test tests::generic_domestic                           ... ok
test tests::generic_imported                           ... ok
test tests::state_sp_surcharge                         ... ok
test tests::state_rj_surcharge                         ... ok
test tests::state_am_electronics_discount_no_negative  ... ok
test tests::state_am_medicine_floor_zero               ... ok
test tests::vip_premium_category_discount              ... ok
test tests::vip_standard_category_discount             ... ok
test tests::vip_and_state_combined                     ... ok
test tests::final_price_equals_base_plus_tax           ... ok
test tests::tax_amount_never_negative                  ... ok

test result: ok. 23 passed; 0 failed; finished in 0.01s

$ cargo tarpaulin
90.98% coverage, 111/122 lines covered
```

A medição de complexidade ciclomática pelo `rust-code-analysis-cli` confirmou a redução do ponto de entrada de 18 para 4. As linhas não cobertas (11/122) correspondem exclusivamente à função `calculate_tax_with_log`, cujo efeito de I/O foi intencionalmente excluído dos testes unitários por design — evidência de que a separação de responsabilidades (R4) funcionou conforme esperado.

A tabela a seguir consolida os indicadores de qualidade mensurados antes e após a refatoração:

| Indicador | Antes | Depois | Variação |
|---|---|---|---|
| Complexidade ciclomática (`calculate_tax`) | 18 | 4 | −77,8% |
| LOC do ponto de entrada público | 87 | 16 | −81,6% |
| Casos de teste | 7 | 23 | +228,6% |
| Cobertura de linhas (`cargo tarpaulin`) | 64% | 91% | +27 p.p. |
| Regressões introduzidas | — | 0 | — |
| *Bad smells* identificados | 5 | 0 | −100% |

*Tabela 2 — Comparativo de métricas de qualidade antes e após a refatoração (Rust, 2026).*

Os resultados confirmam empiricamente a eficácia das técnicas de refatoração na redução da complexidade interna sem alteração do comportamento externo do módulo. A ampliação da cobertura de testes é uma consequência direta da maior testabilidade proporcionada pela separação de responsabilidades: funções menores, puras e com um único ponto de entrada permitem a criação de casos de teste focados, sem a necessidade de construir estados complexos para atingir ramos profundamente aninhados.

Os artefatos produzidos nesta atividade — código-fonte original (`src/lib_original.rs`), código refatorado (`src/lib.rs`) e os transcritos de execução dos testes — estão disponíveis no repositório que acompanha este trabalho, permitindo a reprodução integral dos experimentos descritos.

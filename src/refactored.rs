// Módulo refatorado — após aplicação das técnicas de Fowler (2018)
//
// Refatorações aplicadas:
//   R1 - Extract Function: alíquota por categoria isolada em função própria
//   R2 - Replace Magic Number with Symbolic Constant
//   R3 - Introduce Parameter Object (TaxContext)
//   R4 - Separate Query from Modifier: println! removido da função de cálculo

// R2 — constantes nomeadas no lugar de números mágicos
const RATE_ELECTRONICS_DOMESTIC: f64 = 0.15;
const RATE_ELECTRONICS_IMPORTED: f64 = 0.35;
const RATE_FOOD_DOMESTIC: f64 = 0.04;
const RATE_FOOD_IMPORTED: f64 = 0.12;
const RATE_CLOTHING_DOMESTIC: f64 = 0.12;
const RATE_CLOTHING_IMPORTED: f64 = 0.28;
const RATE_DEFAULT: f64 = 0.10;
const SURCHARGE_SP: f64 = 0.02;
const SURCHARGE_RJ: f64 = 0.015;
const DISCOUNT_VIP: f64 = 0.05;

#[derive(Debug, Clone)]
pub struct Product {
    pub name: String,
    pub price: f64,
    pub category: String,
    pub imported: bool,
}

#[derive(Debug, PartialEq)]
pub struct TaxResult {
    pub base_price: f64,
    pub tax_amount: f64,
    pub final_price: f64,
}

// R3 — objeto de contexto agrupa os parâmetros da operação
pub struct TaxContext<'a> {
    pub product: &'a Product,
    pub state: &'a str,
    pub is_vip: bool,
}

// R1 — função extraída: responsabilidade única de determinar alíquota base
fn base_rate(product: &Product) -> f64 {
    match product.category.as_str() {
        "eletronicos" => {
            if product.imported {
                RATE_ELECTRONICS_IMPORTED
            } else {
                RATE_ELECTRONICS_DOMESTIC
            }
        }
        "alimentos" => {
            if product.imported {
                RATE_FOOD_IMPORTED
            } else {
                RATE_FOOD_DOMESTIC
            }
        }
        "vestuario" => {
            if product.imported {
                RATE_CLOTHING_IMPORTED
            } else {
                RATE_CLOTHING_DOMESTIC
            }
        }
        _ => RATE_DEFAULT,
    }
}

fn state_adjustment(rate: f64, state: &str) -> f64 {
    match state {
        "SP" => rate + SURCHARGE_SP,
        "RJ" => rate + SURCHARGE_RJ,
        _ => rate,
    }
}

fn vip_adjustment(rate: f64) -> f64 {
    (rate - DISCOUNT_VIP).max(0.0)
}

// R4 — função pura, sem efeitos colaterais (println! removido)
// Complexidade ciclomática: 2
pub fn calculate_tax(ctx: &TaxContext) -> TaxResult {
    let mut rate = base_rate(ctx.product);
    rate = state_adjustment(rate, ctx.state);
    if ctx.is_vip {
        rate = vip_adjustment(rate);
    }
    let tax = ctx.product.price * rate;
    TaxResult {
        base_price: ctx.product.price,
        tax_amount: tax,
        final_price: ctx.product.price + tax,
    }
}

// Wrapper separado com efeito de log (R4)
pub fn calculate_tax_logged(ctx: &TaxContext) -> TaxResult {
    let result = calculate_tax(ctx);
    println!(
        "[LOG] {} | imposto=R${:.2}",
        ctx.product.name, result.tax_amount
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prod(category: &str, imported: bool, price: f64) -> Product {
        Product {
            name: "Teste".into(),
            price,
            category: category.into(),
            imported,
        }
    }

    fn ctx<'a>(p: &'a Product, state: &'a str, vip: bool) -> TaxContext<'a> {
        TaxContext {
            product: p,
            state,
            is_vip: vip,
        }
    }

    // TC-01
    #[test]
    fn electronics_domestic() {
        let p = prod("eletronicos", false, 500.0);
        let r = calculate_tax(&ctx(&p, "MG", false));
        assert!((r.tax_amount - 75.0).abs() < 1e-6);
    }

    // TC-02
    #[test]
    fn electronics_imported() {
        let p = prod("eletronicos", true, 500.0);
        let r = calculate_tax(&ctx(&p, "MG", false));
        assert!((r.tax_amount - 175.0).abs() < 1e-6);
    }

    // TC-03
    #[test]
    fn food_domestic() {
        let p = prod("alimentos", false, 100.0);
        let r = calculate_tax(&ctx(&p, "MG", false));
        assert!((r.tax_amount - 4.0).abs() < 1e-6);
    }

    // TC-04
    #[test]
    fn food_imported() {
        let p = prod("alimentos", true, 100.0);
        let r = calculate_tax(&ctx(&p, "MG", false));
        assert!((r.tax_amount - 12.0).abs() < 1e-6);
    }

    // TC-05
    #[test]
    fn clothing_domestic() {
        let p = prod("vestuario", false, 200.0);
        let r = calculate_tax(&ctx(&p, "MG", false));
        assert!((r.tax_amount - 24.0).abs() < 1e-6);
    }

    // TC-06
    #[test]
    fn clothing_imported() {
        let p = prod("vestuario", true, 200.0);
        let r = calculate_tax(&ctx(&p, "MG", false));
        assert!((r.tax_amount - 56.0).abs() < 1e-6);
    }

    // TC-07
    #[test]
    fn state_sp_surcharge() {
        let p = prod("eletronicos", false, 500.0);
        let r = calculate_tax(&ctx(&p, "SP", false));
        // (0.15 + 0.02) * 500 = 85.0
        assert!((r.tax_amount - 85.0).abs() < 1e-6);
    }

    // TC-08
    #[test]
    fn state_rj_surcharge() {
        let p = prod("eletronicos", false, 500.0);
        let r = calculate_tax(&ctx(&p, "RJ", false));
        // (0.15 + 0.015) * 500 = 82.5
        assert!((r.tax_amount - 82.5).abs() < 1e-6);
    }

    // TC-09
    #[test]
    fn vip_discount() {
        let p = prod("eletronicos", false, 500.0);
        let r = calculate_tax(&ctx(&p, "MG", true));
        // (0.15 - 0.05) * 500 = 50.0
        assert!((r.tax_amount - 50.0).abs() < 1e-6);
    }

    // TC-10
    #[test]
    fn vip_rate_never_negative() {
        // food domestic 0.04 - vip 0.05 => clamp a 0.0
        let p = prod("alimentos", false, 100.0);
        let r = calculate_tax(&ctx(&p, "MG", true));
        assert!(r.tax_amount >= 0.0);
    }

    // TC-11
    #[test]
    fn final_price_equals_base_plus_tax() {
        let p = prod("vestuario", false, 200.0);
        let r = calculate_tax(&ctx(&p, "MG", false));
        assert!((r.final_price - (r.base_price + r.tax_amount)).abs() < 1e-6);
    }

    // TC-12
    #[test]
    fn unknown_category_uses_default_rate() {
        let p = prod("brinquedos", false, 100.0);
        let r = calculate_tax(&ctx(&p, "MG", false));
        assert!((r.tax_amount - 10.0).abs() < 1e-6);
    }
}

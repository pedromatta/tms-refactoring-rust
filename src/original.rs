// Módulo original — antes da refatoração
// Bad smells presentes:
//   - função longa com múltiplas responsabilidades
//   - números mágicos espalhados
//   - código duplicado nos ramos de categoria
//   - println! misturado com lógica de cálculo (efeito colateral)

#[derive(Debug, Clone)]
pub struct Product {
    pub name: String,
    pub price: f64,
    pub category: String, // "eletronicos" | "alimentos" | "vestuario"
    pub imported: bool,
}

#[derive(Debug)]
pub struct TaxResult {
    pub base_price: f64,
    pub tax_amount: f64,
    pub final_price: f64,
}

pub fn calculate_tax(product: &Product, state: &str, is_vip: bool) -> TaxResult {
    let mut rate: f64;

    // bloco de categoria
    if product.category == "eletronicos" {
        if product.imported {
            rate = 0.35;
        } else {
            rate = 0.15;
        }
    } else if product.category == "alimentos" {
        if product.imported {
            rate = 0.12;
        } else {
            rate = 0.04;
        }
    } else if product.category == "vestuario" {
        if product.imported {
            rate = 0.28;
        } else {
            rate = 0.12;
        }
    } else {
        rate = 0.10;
    }

    // ajuste por estado
    if state == "SP" {
        rate += 0.02;
    } else if state == "RJ" {
        rate += 0.015;
    }

    // desconto VIP
    if is_vip {
        rate -= 0.05;
        if rate < 0.0 {
            rate = 0.0;
        }
    }

    let tax = product.price * rate;
    let final_price = product.price + tax;

    // imprime o resultado do cálculo
    println!(
        "[LOG] {} | aliq={:.0}% | imposto=R${:.2}",
        product.name,
        rate * 100.0,
        tax
    );

    TaxResult {
        base_price: product.price,
        tax_amount: tax,
        final_price,
    }
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

    // TC-01
    #[test]
    fn electronics_domestic() {
        let r = calculate_tax(&prod("eletronicos", false, 500.0), "MG", false);
        assert!((r.tax_amount - 75.0).abs() < 1e-6);
    }

    // TC-02
    #[test]
    fn electronics_imported() {
        let r = calculate_tax(&prod("eletronicos", true, 500.0), "MG", false);
        assert!((r.tax_amount - 175.0).abs() < 1e-6);
    }

    // TC-03
    #[test]
    fn food_domestic() {
        let r = calculate_tax(&prod("alimentos", false, 100.0), "MG", false);
        assert!((r.tax_amount - 4.0).abs() < 1e-6);
    }

    // TC-04
    #[test]
    fn state_sp_adds_surcharge() {
        let r = calculate_tax(&prod("eletronicos", false, 500.0), "SP", false);
        // 0.15 + 0.02 = 0.17 => 85.0
        assert!((r.tax_amount - 85.0).abs() < 1e-6);
    }

    // TC-05
    #[test]
    fn vip_discount_applied() {
        let r = calculate_tax(&prod("eletronicos", false, 500.0), "MG", true);
        // 0.15 - 0.05 = 0.10 => 50.0
        assert!((r.tax_amount - 50.0).abs() < 1e-6);
    }

    // TC-06
    #[test]
    fn final_price_is_base_plus_tax() {
        let r = calculate_tax(&prod("vetuario", false, 200.0), "MG", false);
        assert!((r.final_price - (r.base_price + r.tax_amount)).abs() < 1e-6);
    }
}

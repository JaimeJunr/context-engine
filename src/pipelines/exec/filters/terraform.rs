// Filtros para Terraform / OpenTofu (`tofu`) — outputs gigantes que dominam contexto.

use crate::pipelines::exec::types::FilterConfig;

/// `terraform plan` / `tofu plan` — preserva resumo ("Plan: N to add, M to change..."),
/// linhas com `+`, `-`, `~` (mudanças), e remove preâmbulos de refresh/lock.
pub fn plan() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        // Remove ruído típico de provider download/refresh/lock.
        strip_lines_matching: vec![
            r"^Initializing the backend\.\.\.".to_string(),
            r"^Initializing provider plugins\.\.\.".to_string(),
            r"^- Finding ".to_string(),
            r"^- Installing ".to_string(),
            r"^- Installed ".to_string(),
            r"^- Using previously-installed ".to_string(),
            r"^Refreshing state\.\.\.".to_string(),
            r"^.* Refreshing state\.\.\.".to_string(),
            r"^.* Reading\.\.\.".to_string(),
            r"^.* Read complete after".to_string(),
            r"^Acquiring state lock".to_string(),
            r"^Releasing state lock".to_string(),
            r"^Terraform has compared".to_string(),
            r"^Note: Objects have changed outside".to_string(),
            r"^Unless you have made equivalent".to_string(),
            r"^Terraform will perform the following actions:".to_string(),
        ],
        truncate_lines_at: Some(200),
        max_lines: Some(200),
        on_empty: Some("(no changes)".to_string()),
        ..Default::default()
    }
}

/// `terraform apply` / `tofu apply` — preserva resumo final e erros.
pub fn apply() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        // Mantém só linhas relevantes ao resultado.
        keep_lines_matching: vec![
            r"Apply complete".to_string(),
            r"Error:".to_string(),
            r"Warning:".to_string(),
            r"Resources:".to_string(),
            r"Outputs:".to_string(),
            r": Creating\.\.\.".to_string(),
            r": Destroying\.\.\.".to_string(),
            r": Modifying\.\.\.".to_string(),
            r": Creation complete".to_string(),
            r": Destruction complete".to_string(),
            r": Modifications complete".to_string(),
        ],
        max_lines: Some(80),
        ..Default::default()
    }
}

/// `terraform init` / `tofu init` — apenas resultado.
pub fn init() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"Terraform has been successfully initialized".to_string(),
            r"OpenTofu has been successfully initialized".to_string(),
            r"Error:".to_string(),
            r"Warning:".to_string(),
        ],
        max_lines: Some(10),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

/// `terraform validate` / `tofu validate` — ok ou erros.
pub fn validate() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"Success!".to_string(),
            r"Error:".to_string(),
            r"Warning:".to_string(),
        ],
        max_lines: Some(20),
        on_empty: Some("ok".to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipelines::exec::pipeline::apply_pipeline;

    #[test]
    fn terraform_plan_remove_refresh_e_preserva_mudancas() {
        let input = r#"Initializing the backend...
Initializing provider plugins...
- Finding hashicorp/aws versions matching "~> 5.0"...
- Installed hashicorp/aws v5.42.0
aws_instance.example: Refreshing state... [id=i-abc1234567890]
aws_s3_bucket.logs: Refreshing state... [id=my-logs-bucket]

Terraform used the selected providers to generate the following execution plan.
Resource actions are indicated with the following symbols:
  + create
  ~ update in-place
  - destroy

Terraform will perform the following actions:

  # aws_instance.example will be updated in-place
  ~ resource "aws_instance" "example" {
        id = "i-abc1234567890"
      ~ instance_type = "t3.micro" -> "t3.small"
    }

Plan: 0 to add, 1 to change, 0 to destroy."#;
        let out = apply_pipeline(input, &plan());
        assert!(!out.contains("Refreshing state"), "refresh deve sumir");
        assert!(!out.contains("Installed hashicorp"), "install deve sumir");
        assert!(out.contains("Plan: 0 to add"), "resumo deve permanecer");
        assert!(
            out.contains("instance_type"),
            "diff de campo deve permanecer"
        );
    }

    #[test]
    fn terraform_init_so_resultado() {
        let input = "Initializing the backend...\nLots of stuff...\nTerraform has been successfully initialized!";
        let out = apply_pipeline(input, &init());
        assert!(out.contains("successfully initialized"));
        assert!(out.lines().count() <= 2);
    }
}

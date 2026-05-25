use crate::pipelines::exec::dedup::group_repeated;
use crate::pipelines::exec::types::FilterConfig;

pub fn generic() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(80),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

/// `aws logs tail/filter-log-events` — dedup em eventos repetidos.
pub fn logs() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        preprocess: Some(group_repeated),
        tail_lines: Some(100),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

/// `aws sts get-caller-identity` — output sempre pequeno, mantém quase íntegro.
pub fn sts() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(20),
        ..Default::default()
    }
}

/// `aws s3 ls/cp/sync` — comprime listagens longas.
pub fn s3() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        max_lines: Some(80),
        truncate_lines_at: Some(160),
        ..Default::default()
    }
}

/// `aws ec2 describe-instances` — JSON inflado, comprime.
pub fn ec2() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        // Remove campos verbosos do JSON que raramente importam
        strip_lines_matching: vec![
            r#"^\s*"AmiLaunchIndex":"#.to_string(),
            r#"^\s*"ProductCodes":"#.to_string(),
            r#"^\s*"ClientToken":"#.to_string(),
            r#"^\s*"Hypervisor":"#.to_string(),
            r#"^\s*"VirtualizationType":"#.to_string(),
            r#"^\s*"EnaSupport":"#.to_string(),
            r#"^\s*"SriovNetSupport":"#.to_string(),
        ],
        max_lines: Some(120),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

/// `aws lambda list-functions/invoke` — strip de campos verbosos.
pub fn lambda() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        strip_lines_matching: vec![
            r#"^\s*"CodeSha256":"#.to_string(),
            r#"^\s*"RevisionId":"#.to_string(),
            r#"^\s*"MasterArn":"#.to_string(),
        ],
        max_lines: Some(120),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

/// `aws iam` — strip de policy documents inline (geralmente enormes).
pub fn iam() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        preprocess: Some(strip_policy_documents),
        max_lines: Some(80),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

/// `aws dynamodb` — unwrap dos type annotations DynamoDB ({"S": "x"} → "x").
pub fn dynamodb() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        preprocess: Some(unwrap_dynamodb_types),
        max_lines: Some(120),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

/// `aws cloudformation` — comprime listagens e remove campos meta.
pub fn cloudformation() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        strip_lines_matching: vec![
            r#"^\s*"StackId":"#.to_string(),
            r#"^\s*"RoleARN":"#.to_string(),
            r#"^\s*"ChangeSetId":"#.to_string(),
        ],
        max_lines: Some(120),
        truncate_lines_at: Some(200),
        ..Default::default()
    }
}

/// Remove blocos "PolicyDocument": { ... } substituindo por "<policy>".
fn strip_policy_documents(input: &str) -> String {
    // Heurística simples: linha começa com "PolicyDocument" → ignora até o `}` matching.
    let mut out = String::new();
    let mut in_policy = false;
    let mut depth = 0i32;
    for line in input.lines() {
        let trimmed = line.trim();
        if !in_policy && trimmed.starts_with("\"PolicyDocument\"") {
            in_policy = true;
            depth = trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
            out.push_str(&line[..line.find('"').unwrap_or(0)]);
            out.push_str("\"PolicyDocument\": \"<policy>\",\n");
            if depth == 0 {
                in_policy = false;
            }
            continue;
        }
        if in_policy {
            depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
            if depth <= 0 {
                in_policy = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Reescreve type-annotations DynamoDB: `{"S": "valor"}` → `"valor"`,
/// `{"N": "42"}` → `42`, `{"BOOL": true}` → `true`.
fn unwrap_dynamodb_types(input: &str) -> String {
    use std::sync::OnceLock;
    static PATTERNS: OnceLock<Vec<(regex::Regex, &'static str)>> = OnceLock::new();
    let patterns = PATTERNS.get_or_init(|| {
        vec![
            // {"S": "valor"} → "valor"
            (
                regex::Regex::new(r#"\{\s*"S"\s*:\s*("(?:[^"\\]|\\.)*")\s*\}"#).unwrap(),
                "$1",
            ),
            // {"N": "42"} → 42 (sem aspas)
            (
                regex::Regex::new(r#"\{\s*"N"\s*:\s*"(-?[0-9.]+)"\s*\}"#).unwrap(),
                "$1",
            ),
            // {"BOOL": true|false} → true|false
            (
                regex::Regex::new(r#"\{\s*"BOOL"\s*:\s*(true|false)\s*\}"#).unwrap(),
                "$1",
            ),
            // {"NULL": true} → null
            (
                regex::Regex::new(r#"\{\s*"NULL"\s*:\s*true\s*\}"#).unwrap(),
                "null",
            ),
        ]
    });

    let mut result = input.to_string();
    for (re, replacement) in patterns {
        result = re.replace_all(&result, *replacement).to_string();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipelines::exec::pipeline::apply_pipeline;

    #[test]
    fn aws_logs_agrupa_eventos_repetidos() {
        let mut input = String::new();
        for i in 0..6 {
            input.push_str(&format!(
                "2024-01-15T12:00:{:02}Z [INFO] processing batch\n",
                i
            ));
        }
        let out = apply_pipeline(&input, &logs());
        assert!(out.contains("(×6)"), "esperava agrupamento: {}", out);
    }

    #[test]
    fn iam_remove_policy_document_inline() {
        let input = r#"{
    "RoleName": "MyRole",
    "PolicyDocument": {
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": "*",
            "Resource": "*"
        }]
    },
    "Arn": "arn:aws:iam::123:role/MyRole"
}"#;
        let out = apply_pipeline(input, &iam());
        assert!(out.contains("<policy>"), "esperava placeholder: {}", out);
        assert!(!out.contains("Version"), "policy detail deve ser removido");
        assert!(out.contains("MyRole"), "demais campos preservados");
    }

    #[test]
    fn dynamodb_unwrap_string_value() {
        let input =
            r#"{"Item": {"name": {"S": "Maria"}, "age": {"N": "30"}, "active": {"BOOL": true}}}"#;
        let out = apply_pipeline(input, &dynamodb());
        assert!(out.contains("\"Maria\""), "esperava string unwrap: {}", out);
        assert!(out.contains("30"), "esperava number unwrap");
        assert!(out.contains("true"), "esperava bool unwrap");
        assert!(!out.contains("\"S\":"), "type tag deve sumir: {}", out);
    }

    #[test]
    fn ec2_remove_campos_verbosos() {
        let input = r#"{
    "InstanceId": "i-abc123",
    "AmiLaunchIndex": 0,
    "ProductCodes": [],
    "Hypervisor": "xen",
    "State": {"Name": "running"}
}"#;
        let out = apply_pipeline(input, &ec2());
        assert!(out.contains("InstanceId"));
        assert!(!out.contains("AmiLaunchIndex"));
        assert!(!out.contains("Hypervisor"));
    }

    #[test]
    fn sts_passthrough_pequeno() {
        let input = r#"{"UserId": "AIDA...", "Account": "123456789012", "Arn": "arn:aws:iam::123:user/foo"}"#;
        let out = apply_pipeline(input, &sts());
        assert!(out.contains("Account"));
    }
}

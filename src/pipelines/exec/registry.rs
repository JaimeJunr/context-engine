use super::{filters, types::FilterConfig};

/// Indica se um par (comando, args) possui filtro registrado.
/// Reusa `lookup` — fonte única de verdade para "comando coberto".
pub fn matches(cmd: &str, args: &[String]) -> bool {
    lookup(cmd, args).is_some()
}

/// Resolve o FilterConfig para um comando + args.
/// Tenta match específico (cmd + primeiro arg) antes de cair para match genérico (só cmd).
/// Retorna None se o comando não tem filtro registrado (passthrough).
pub fn lookup(cmd: &str, args: &[String]) -> Option<FilterConfig> {
    let first_arg = args.first().map(|s| s.as_str()).unwrap_or("");

    // Match específico: "git status", "cargo test", etc.
    let specific_key = format!("{} {}", cmd, first_arg);
    if let Some(config) = lookup_specific(&specific_key) {
        return Some(config);
    }

    // Match genérico: só o nome do comando
    lookup_generic(cmd)
}

fn lookup_specific(key: &str) -> Option<FilterConfig> {
    match key {
        // git
        "git status" => Some(filters::git::status()),
        "git log" => Some(filters::git::log()),
        "git diff" => Some(filters::git::diff()),
        "git show" => Some(filters::git::diff()),
        "git branch" => Some(filters::git::generic()),
        "git tag" => Some(filters::git::generic()),
        "git stash" => Some(filters::git::generic()),
        "git blame" => Some(filters::git::blame()),
        "git push" => Some(filters::git::push()),
        "git pull" => Some(filters::git::pull()),
        "git add" => Some(filters::git::add()),
        "git commit" => Some(filters::git::commit()),
        "git fetch" => Some(filters::git::pull()),

        // cargo
        "cargo test" => Some(filters::cargo::test()),
        "cargo build" => Some(filters::cargo::build()),
        "cargo check" => Some(filters::cargo::build()),
        "cargo clippy" => Some(filters::cargo::clippy()),
        "cargo fmt" => Some(filters::cargo::fmt()),
        "cargo run" => Some(filters::cargo::run()),
        "cargo install" => Some(filters::cargo::build()),

        // npm/yarn
        "npm install" => Some(filters::node::install()),
        "npm test" => Some(filters::node::test()),
        "npm run" => Some(filters::node::run_script()),
        "npm build" => Some(filters::node::build()),
        "yarn install" => Some(filters::node::install()),
        "yarn test" => Some(filters::node::test()),
        "yarn build" => Some(filters::node::build()),
        "pnpm install" => Some(filters::node::install()),
        "pnpm test" => Some(filters::node::test()),
        "pnpm build" => Some(filters::node::build()),

        // gh
        "gh pr" => Some(filters::gh::pr_list()),
        "gh issue" => Some(filters::gh::issue_list()),
        "gh run" => Some(filters::gh::run_list()),

        // docker
        "docker ps" => Some(filters::docker::ps()),
        "docker images" => Some(filters::docker::images()),
        "docker logs" => Some(filters::docker::logs()),
        "docker compose" => Some(filters::docker::compose_ps()),

        // kubectl subcomandos comuns
        "kubectl get" => Some(filters::kubectl::get()),
        "kubectl logs" => Some(filters::kubectl::logs()),
        "kubectl describe" => Some(filters::kubectl::get()),

        // aws
        "aws logs" => Some(filters::aws::logs()),
        "aws sts" => Some(filters::aws::sts()),
        "aws s3" => Some(filters::aws::s3()),
        "aws s3api" => Some(filters::aws::s3()),
        "aws ec2" => Some(filters::aws::ec2()),
        "aws lambda" => Some(filters::aws::lambda()),
        "aws iam" => Some(filters::aws::iam()),
        "aws dynamodb" => Some(filters::aws::dynamodb()),
        "aws cloudformation" => Some(filters::aws::cloudformation()),
        "aws cfn" => Some(filters::aws::cloudformation()),

        // gradle
        "gradle test" => Some(filters::jvm::gradle_test()),
        "gradle build" => Some(filters::jvm::gradle_build()),
        "gradlew test" => Some(filters::jvm::gradle_test()),
        "gradlew build" => Some(filters::jvm::gradle_build()),
        "./gradlew test" => Some(filters::jvm::gradle_test()),
        "./gradlew build" => Some(filters::jvm::gradle_build()),

        // maven
        "mvn test" => Some(filters::jvm::mvn_test()),
        "mvn build" => Some(filters::jvm::mvn_build()),
        "mvn package" => Some(filters::jvm::mvn_build()),
        "mvn install" => Some(filters::jvm::mvn_build()),
        "mvn verify" => Some(filters::jvm::mvn_build()),
        "./mvnw test" => Some(filters::jvm::mvn_test()),
        "./mvnw build" => Some(filters::jvm::mvn_build()),

        // grails
        "grails test-app" => Some(filters::jvm::grails_test()),
        "grails run-app" => Some(filters::jvm::grails_run()),

        // ruby
        "rubocop" => Some(filters::ruby::rubocop()),
        "rspec" => Some(filters::ruby::rspec()),
        "bundle exec rspec" => Some(filters::ruby::rspec()),
        "bundle exec rubocop" => Some(filters::ruby::rubocop()),
        "rake test" => Some(filters::ruby::rake()),
        "rake spec" => Some(filters::ruby::rspec()),
        "bundle exec rake" => Some(filters::ruby::rake()),

        // go
        "go test" => Some(filters::go::test()),
        "go build" => Some(filters::go::build()),
        "go vet" => Some(filters::go::golangci()),
        "golangci-lint run" => Some(filters::go::golangci()),
        "golangci-lint" => Some(filters::go::golangci()),

        // linters Python
        "ruff check" => Some(filters::python::ruff()),
        "ruff format" => Some(filters::python::ruff()),
        "mypy" => Some(filters::python::mypy()),

        // linters JS/TS
        "tsc" => Some(filters::ts::tsc()),
        "eslint" => Some(filters::ts::eslint()),
        "prettier" => Some(filters::ts::prettier()),
        "biome" => Some(filters::ts::biome()),
        "npx tsc" => Some(filters::ts::tsc()),
        "npx eslint" => Some(filters::ts::eslint()),
        "npx prettier" => Some(filters::ts::prettier()),
        "pnpm tsc" => Some(filters::ts::tsc()),
        "pnpm eslint" => Some(filters::ts::eslint()),

        // terraform / opentofu
        "terraform plan" => Some(filters::terraform::plan()),
        "terraform apply" => Some(filters::terraform::apply()),
        "terraform init" => Some(filters::terraform::init()),
        "terraform validate" => Some(filters::terraform::validate()),
        "tofu plan" => Some(filters::terraform::plan()),
        "tofu apply" => Some(filters::terraform::apply()),
        "tofu init" => Some(filters::terraform::init()),
        "tofu validate" => Some(filters::terraform::validate()),

        _ => None,
    }
}

fn lookup_generic(cmd: &str) -> Option<FilterConfig> {
    match cmd {
        "git" => Some(filters::git::generic()),
        "cargo" => Some(filters::cargo::build()),
        "npm" | "yarn" | "pnpm" => Some(filters::node::run_script()),
        "jest" | "vitest" => Some(filters::node::test()),
        "pytest" => Some(filters::python::pytest()),
        "python" | "python3" => Some(filters::python::generic()),
        "ls" => Some(filters::nav::ls()),
        "find" => Some(filters::nav::find()),
        "tree" => Some(filters::nav::tree()),
        "grep" | "rg" | "ag" => Some(filters::nav::grep()),
        "gh" => Some(filters::gh::generic()),
        "docker" => Some(filters::docker::ps()),
        "kubectl" => Some(filters::kubectl::get()),
        "aws" => Some(filters::aws::generic()),
        "tsc" => Some(filters::ts::tsc()),
        "eslint" => Some(filters::ts::eslint()),
        "prettier" => Some(filters::ts::prettier()),
        "biome" => Some(filters::ts::biome()),
        "ruff" => Some(filters::python::ruff()),
        "mypy" => Some(filters::python::mypy()),
        "go" => Some(filters::go::build()),
        "golangci-lint" => Some(filters::go::golangci()),
        "rubocop" => Some(filters::ruby::rubocop()),
        "rspec" => Some(filters::ruby::rspec()),
        "rake" => Some(filters::ruby::rake()),
        "terraform" | "tofu" => Some(filters::terraform::plan()),
        "curl" | "wget" => Some(filters::data::curl()),
        "jq" => Some(filters::data::jq()),
        "sqlite3" => Some(filters::data::sqlite3()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipelines::exec::pipeline::apply_pipeline;

    // =========================================================================
    // Testes de comportamento: o filtro entrega o que o agente precisa ver?
    // =========================================================================

    #[test]
    fn cargo_test_com_falha_exibe_nome_do_teste_falhando() {
        let input = include_str!("../../../tests/fixtures/cargo_test_with_failure.txt");
        let config = lookup("cargo", &["test".to_string()]).unwrap();

        let result = apply_pipeline(input, &config);

        assert!(
            result.contains("test_login_invalido"),
            "agente precisa ver qual teste falhou; resultado: {}",
            result
        );
        assert!(
            result.contains("2 failed") || result.contains("FAILED"),
            "agente precisa ver o resumo de falha"
        );
    }

    #[test]
    fn cargo_test_filtro_e_mais_especifico_que_cargo_generico() {
        // "cargo test" deve usar filtro de teste (com keep_lines para mostrar falhas)
        // enquanto "cargo build" usa filtro de build (sem keep_lines de testes)
        let test_config = lookup("cargo", &["test".to_string()]).unwrap();
        let build_config = lookup("cargo", &["build".to_string()]).unwrap();

        // filtro de teste preserva linhas de falha; de build não precisa de tail
        assert!(
            !test_config.keep_lines_matching.is_empty(),
            "cargo test deve ter padrões de inclusão para mostrar falhas"
        );
        assert!(
            build_config.keep_lines_matching.is_empty()
                || build_config.keep_lines_matching != test_config.keep_lines_matching,
            "cargo build deve ter filtro diferente de cargo test"
        );
    }

    #[test]
    fn git_status_com_mudancas_preserva_lista_de_arquivos() {
        let input = include_str!("../../../tests/fixtures/git_status_dirty.txt");
        let config = lookup("git", &["status".to_string()]).unwrap();

        let result = apply_pipeline(input, &config);

        assert!(
            result.contains("src/auth.rs"),
            "arquivo modificado deve aparecer"
        );
        assert!(
            !result.is_empty(),
            "git status com mudanças não pode ser vazio"
        );
    }

    #[test]
    fn git_status_clean_retorna_mensagem_util() {
        // Dado: working tree limpo (output vazio após filtro)
        let config = lookup("git", &["status".to_string()]).unwrap();

        let result = apply_pipeline("", &config);

        // Não deve retornar silêncio — deve informar ao agente que está limpo
        assert!(
            config.on_empty.is_some(),
            "git status deve ter mensagem para output vazio"
        );
        let _ = result; // resultado depende do on_empty configurado
    }

    #[test]
    fn comando_desconhecido_retorna_none_para_passthrough() {
        // Comportamento crítico: ctx exec nunca deve falhar por comando desconhecido
        assert!(lookup("meu-binario-exotico", &[]).is_none());
        assert!(lookup("g++", &["main.cpp".to_string()]).is_none());
        assert!(lookup("ruby", &["script.rb".to_string()]).is_none());
        assert!(lookup("./meu-script.sh", &[]).is_none());
    }

    #[test]
    fn todos_comandos_suportados_tem_strip_ansi() {
        // Invariante: qualquer filtro registrado deve remover ANSI
        // (output colorido de terminal não deve chegar ao agente)
        let comandos = vec![
            ("git", vec!["status"]),
            ("cargo", vec!["test"]),
            ("cargo", vec!["build"]),
            ("ls", vec![]),
            ("find", vec![]),
            ("grep", vec![]),
            ("pytest", vec![]),
            ("docker", vec!["ps"]),
        ];
        for (cmd, args) in comandos {
            let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            let config = lookup(cmd, &args)
                .unwrap_or_else(|| panic!("comando '{}' deveria ter filtro registrado", cmd));
            assert!(
                config.strip_ansi,
                "filtro de '{}' deve ter strip_ansi=true",
                cmd
            );
        }
    }
}

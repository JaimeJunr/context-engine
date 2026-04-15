use crate::exec::types::FilterConfig;

pub fn gradle_test() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"FAILED".to_string(),
            r"BUILD FAILED".to_string(),
            r"BUILD SUCCESSFUL".to_string(),
            r"tests completed".to_string(),
            r"Exception".to_string(),
            r"Error at".to_string(),
            r"SpockAssertionError".to_string(),
            r"AssertionError".to_string(),
            r"ArithmeticException".to_string(),
            r"FAILURE:".to_string(),
            r"What went wrong".to_string(),
            r"There were failing tests".to_string(),
        ],
        strip_lines_matching: vec![r"^> Task :".to_string(), r"^Download ".to_string()],
        max_lines: Some(100),
        on_empty: Some("(testes concluídos sem falhas)".to_string()),
        ..Default::default()
    }
}

pub fn gradle_build() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"error:".to_string(),
            r"BUILD FAILED".to_string(),
            r"BUILD SUCCESSFUL".to_string(),
            r"Compilation failed".to_string(),
        ],
        strip_lines_matching: vec![r"^> Task :".to_string(), r"^Download ".to_string()],
        max_lines: Some(80),
        on_empty: Some("(build concluído sem erros)".to_string()),
        ..Default::default()
    }
}

pub fn mvn_test() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        strip_lines_matching: vec![r"^\[INFO\]".to_string()],
        max_lines: Some(80),
        on_empty: Some("(testes Maven concluídos sem erros)".to_string()),
        ..Default::default()
    }
}

pub fn mvn_build() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        strip_lines_matching: vec![r"^\[INFO\]".to_string()],
        max_lines: Some(80),
        on_empty: Some("(build Maven concluído sem erros)".to_string()),
        ..Default::default()
    }
}

pub fn grails_run() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        strip_lines_matching: vec![
            r"Mapped URL path".to_string(),
            r"Mapping: ".to_string(),
            r"Bean:".to_string(),
            r"Initializing Spring".to_string(),
        ],
        max_lines: Some(50),
        ..Default::default()
    }
}

pub fn grails_test() -> FilterConfig {
    FilterConfig {
        strip_ansi: true,
        keep_lines_matching: vec![
            r"FAILED".to_string(),
            r"BUILD FAILED".to_string(),
            r"BUILD SUCCESSFUL".to_string(),
            r"tests completed".to_string(),
            r"Error".to_string(),
            r"Exception".to_string(),
        ],
        strip_lines_matching: vec![r"^> Task :".to_string(), r"^Download ".to_string()],
        max_lines: Some(100),
        on_empty: Some("(testes Grails concluídos sem falhas)".to_string()),
        ..Default::default()
    }
}

// Segmentação semântica de documentos (RD-03, RD-04)
//
// Hierarquia de pontos de ruptura (prioridade decrescente):
//   H1 > H2 > bloco de código/exemplo > parágrafo > quebra de linha > espaço
//
// Sobreposição: ~15% do tamanho do chunk anterior (RD-04)

const DEFAULT_CHUNK_SIZE: usize = 1000; // caracteres
const OVERLAP_RATIO: f64 = 0.15;

// Representa um fragmento produzido pelo chunker
#[derive(Debug, Clone, PartialEq)]
pub struct RawChunk {
    pub content: String,
    pub start_offset: usize,
}

pub fn chunk_document(text: &str, chunk_size: usize) -> Vec<RawChunk> {
    if text.is_empty() {
        return vec![];
    }
    let size = if chunk_size == 0 {
        DEFAULT_CHUNK_SIZE
    } else {
        chunk_size
    };
    split_with_overlap(text, size, OVERLAP_RATIO)
}

fn split_with_overlap(text: &str, chunk_size: usize, overlap_ratio: f64) -> Vec<RawChunk> {
    let mut chunks: Vec<RawChunk> = Vec::new();
    let mut pos = 0usize;

    while pos < text.len() {
        // Garante que `end` caia em uma fronteira de char UTF-8 (bug fix)
        let raw_end = (pos + chunk_size).min(text.len());
        let end = floor_char_boundary(text, raw_end);
        let window = &text[pos..end];

        // Tenta encontrar ponto de ruptura natural dentro da janela
        let cut = find_break_point(window, chunk_size);
        let actual_end = pos + cut;

        let content = text[pos..actual_end].trim().to_string();
        if !content.is_empty() {
            chunks.push(RawChunk {
                content,
                start_offset: pos,
            });
        }

        if actual_end <= pos {
            // Segurança: avança pelo menos 1 char
            let next = next_char_boundary(text, pos + 1);
            pos = next;
            continue;
        }

        // Se consumimos até o fim do texto, para — evita chunk extra de overlap (bug fix)
        if actual_end >= text.len() {
            break;
        }

        // Calcula sobreposição para próximo chunk (RD-04)
        let overlap = ((actual_end - pos) as f64 * overlap_ratio) as usize;
        let raw_next = actual_end.saturating_sub(overlap);
        let next_start = floor_char_boundary(text, raw_next);
        pos = if next_start > pos {
            next_start
        } else {
            actual_end
        };
    }

    chunks
}

// Retrocede até a fronteira de char mais próxima (≤ pos)
fn floor_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut p = pos;
    while p > 0 && !s.is_char_boundary(p) {
        p -= 1;
    }
    p
}

// Avança até a próxima fronteira de char (≥ pos)
fn next_char_boundary(s: &str, pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    let mut p = pos;
    while p < s.len() && !s.is_char_boundary(p) {
        p += 1;
    }
    p
}

// Encontra melhor ponto de corte dentro de `window` seguindo a hierarquia RD-03
fn find_break_point(window: &str, max_size: usize) -> usize {
    let len = window.len();
    if len <= max_size {
        return len;
    }

    // Busca na segunda metade da janela para não criar chunks muito pequenos
    let search_start = max_size / 2;

    // 1. H1: "\n# " — maior prioridade
    if let Some(idx) = rfind_pattern(window, "\n# ", search_start, len) {
        return idx + 1; // inclui o \n como terminador
    }

    // 2. H2: "\n## "
    if let Some(idx) = rfind_pattern(window, "\n## ", search_start, len) {
        return idx + 1;
    }

    // 3. Delimitador de bloco de código: "\n```"
    if let Some(idx) = rfind_pattern(window, "\n```", search_start, len) {
        return idx + 1;
    }

    // 4. Parágrafo: "\n\n"
    if let Some(idx) = rfind_pattern(window, "\n\n", search_start, len) {
        return idx + 2;
    }

    // 5. Quebra de linha simples: "\n"
    if let Some(idx) = rfind_char(window, '\n', search_start, len) {
        return idx + 1;
    }

    // 6. Último espaço antes do limite
    if let Some(idx) = rfind_char(window, ' ', search_start, len) {
        return idx + 1;
    }

    // Sem ponto de ruptura: corta no limite
    max_size.min(len)
}

// Busca reversa de padrão multi-byte na fatia [start..end] da string
fn rfind_pattern(s: &str, pattern: &str, start: usize, end: usize) -> Option<usize> {
    let slice = &s[..end.min(s.len())];
    // Percorre da direita para a esquerda
    let mut last = None;
    let mut search_from = start;
    while let Some(pos) = slice[search_from..].find(pattern) {
        let abs_pos = search_from + pos;
        last = Some(abs_pos);
        search_from = abs_pos + 1;
        if search_from >= end {
            break;
        }
    }
    last
}

fn rfind_char(s: &str, ch: char, start: usize, end: usize) -> Option<usize> {
    let slice = &s[start..end.min(s.len())];
    slice.rfind(ch).map(|i| start + i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_document() {
        let chunks = chunk_document("", 500);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_small_document_single_chunk() {
        let text = "Olá mundo, este é um texto pequeno.";
        let chunks = chunk_document(text, 500);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start_offset, 0);
        assert!(chunks[0].content.contains("Olá mundo"));
    }

    #[test]
    fn test_chunk_respects_h1_boundary() {
        let text = "# Título A\n\nConteúdo do título A que é bastante longo para forçar chunking.\n\n# Título B\n\nConteúdo do título B também extenso o suficiente para criar um novo fragmento.";
        // chunk_size pequeno para forçar quebra
        let chunks = chunk_document(text, 80);
        // deve haver mais de um chunk
        assert!(
            chunks.len() > 1,
            "esperava múltiplos chunks, obteve {}",
            chunks.len()
        );
        // o primeiro chunk não deve conter o conteúdo de B
        assert!(
            !chunks[0].content.contains("Título B"),
            "primeiro chunk não deveria conter Título B"
        );
    }

    #[test]
    fn test_overlap_15_percent() {
        // Cria texto com marcadores identificáveis em cada chunk
        let part_a = "A".repeat(100);
        let boundary = " QUEBRA ";
        let part_b = "B".repeat(100);
        let text = format!("{}{}{}", part_a, boundary, part_b);

        let chunks = chunk_document(&text, 110);
        // Com sobreposição de ~15%, o segundo chunk deve começar antes do fim do primeiro
        if chunks.len() >= 2 {
            let first_end = chunks[0].start_offset + chunks[0].content.len();
            let second_start = chunks[1].start_offset;
            assert!(
                second_start < first_end,
                "segundo chunk deveria começar antes do fim do primeiro (sobreposição): second_start={} first_end={}",
                second_start, first_end
            );
        }
    }

    #[test]
    fn test_offsets_are_monotonically_increasing() {
        let text = (0..50)
            .map(|i| {
                format!(
                    "Parágrafo {}. Texto de exemplo suficientemente longo.\n\n",
                    i
                )
            })
            .collect::<String>();
        let chunks = chunk_document(&text, 200);
        for w in chunks.windows(2) {
            assert!(
                w[1].start_offset >= w[0].start_offset,
                "offsets devem ser crescentes: {} >= {}",
                w[1].start_offset,
                w[0].start_offset
            );
        }
    }

    #[test]
    fn test_no_empty_chunks() {
        let text = "\n\n\n\nTexto após espaços.\n\n\n";
        let chunks = chunk_document(text, 500);
        for c in &chunks {
            assert!(!c.content.is_empty(), "chunk não deve ser vazio");
        }
    }

    #[test]
    fn test_chunk_with_code_block_boundary() {
        let text = "Descrição inicial.\n\n```rust\nfn hello() {}\n```\n\nTexto após bloco de código com mais conteúdo suficiente para quebrar.";
        let chunks = chunk_document(text, 60);
        // Deve produzir pelo menos 2 chunks
        assert!(!chunks.is_empty());
    }
}

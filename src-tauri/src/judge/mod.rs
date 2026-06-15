/// 테스트케이스 텍스트 정규화.
/// 저장 시(write_tc_files)와 채점 시(실제 출력 비교) 양쪽에서 동일하게 사용한다.
///
/// 규칙:
/// - 각 줄의 trailing whitespace 제거
/// - 전체 앞뒤 빈 줄 제거
pub fn normalize_tc_text(s: &str) -> String {
    s.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_spaces_per_line() {
        assert_eq!(normalize_tc_text("hello   \nworld  "), "hello\nworld");
    }

    #[test]
    fn strips_leading_and_trailing_blank_lines() {
        assert_eq!(normalize_tc_text("\n\nhello\n\n"), "hello");
    }

    #[test]
    fn preserves_internal_blank_lines() {
        assert_eq!(normalize_tc_text("a\n\nb"), "a\n\nb");
    }

    #[test]
    fn empty_input() {
        assert_eq!(normalize_tc_text("   \n  \n"), "");
    }
}

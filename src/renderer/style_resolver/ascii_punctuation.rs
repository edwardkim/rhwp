//! 구두점의 측정과 표시가 같은 영문 프로그램을 사용하도록 슬롯을 정한다.

fn primary(chain: &str) -> &str {
    chain
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(|ch| ch == '\'' || ch == '"')
}

pub(super) fn uses_latin_slot(
    families: &[String],
    latin_metric_face: Option<&str>,
    legacy_latin: bool,
    hangul_hft: bool,
    declared_latin_hft: bool,
    bold: bool,
    italic: bool,
) -> bool {
    let hangul = primary(&families[0]);
    let latin = primary(&families[1]);
    if hangul == latin {
        return false;
    }
    // 표시 대체 face에 한글이 있다고 원본 영문 프로그램까지 한글 글꼴로 보지 않는다.
    let latin_only_substitute = legacy_latin
        && crate::renderer::font_metrics_data::find_metric(
            latin_metric_face.unwrap_or(latin),
            bold,
            italic,
        )
        .is_some_and(|found| found.metric.hangul.is_none());
    // 한글 HFT의 표시용 치환이 영문에 선언한 별도 HFT 프로그램을 덮지 않는다.
    // 영문을 치환하지 않았고 등록된 ASCII 폭 표가 있을 때만 그 슬롯을 쓴다.
    // 메트릭 이름의 기존 별칭도 해소한다. 한양신명조의 실측 ASCII 표는
    // 영문 이름 HanyangSinMyeongJo에 등록돼 있어 별칭 자체를 거부하면 놓친다.
    // HCI Hollyhock 같은 영문 치환이나 미확인 폭은 이 경로로 수용하지 않는다.
    let declared_hft_program = hangul_hft
        && declared_latin_hft
        && crate::renderer::font_metrics_data::find_metric(latin, bold, italic)
            .is_some_and(|found| (' '..='~').all(|ch| found.metric.get_width(ch).is_some()));
    latin_only_substitute || declared_hft_program
}

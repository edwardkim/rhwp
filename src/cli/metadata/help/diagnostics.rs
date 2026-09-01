use super::sink::println;

/// 기존 통짜 help에 절이 없어서 capabilities 요약만 보이던 diagnostic 명령의 실제 사용법.
/// parser와 같은 옵션만 적어 도움말이 존재하지 않는 문법을 만들지 않는다.
pub(super) fn print() {
    println!("  dump-extents <파일.hwp> [-p <쪽번호>] [--min-h <px>] [--outside] [--gaps]");
    println!(
        "      페이지 render tree의 항목별 실제 extent를 출력해 쪽 밖 배치와 빈 공간을 조사한다"
    );
    println!();
    println!("      -p, --page <쪽번호>      0부터 시작하는 한 페이지만 조사 (생략 시 전체)");
    println!("      --min-h <px>             이 높이 미만 노드 생략 (기본: 0)");
    println!("      --outside                쪽 경계를 넘는 노드만 출력");
    println!("      --gaps                   콘텐츠 사이의 세로 빈 구간만 출력");
    println!("      출력의 페이지 번호는 사람이 읽는 1부터 시작하며, -p 입력만 0부터 시작한다");
    println!();
    println!(
        "  measure-width --size <pt> [--font <이름>] [--ratio <백분율>] [--repeat <N>] <text>..."
    );
    println!("      렌더러의 텍스트 폭 추정값을 TSV로 출력하는 글꼴 폭 프로브");
    println!();
    println!("      --size <pt>              양수 글자 크기 (필수)");
    println!("      --font <이름>            글꼴 family (기본: 함초롬바탕)");
    println!("      --ratio <백분율>         양수 폭 비율 (기본: 100)");
    println!("      --repeat <N>             각 text 반복 횟수 (양의 정수, 기본: 1)");
    println!("      --                       뒤의 -로 시작하는 text를 옵션이 아닌 값으로 해석");
    println!("      출력: text, chars, width_px, per_char_px TSV 열");
    println!();
    println!("  core-pages <파일.hwp>");
    println!("      DocumentCore 페이지 수와 각 페이지의 첫 텍스트를 출력해 뷰어 코어와 비교한다");
    println!();
    println!("      출력: core pages:<N> 다음에 p001 형식의 페이지별 첫 텍스트");
    println!("      읽기·파싱 실패는 stderr와 nonzero exit으로 보고한다");
    println!();
}

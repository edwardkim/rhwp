# 누적 renderer 보정의 lint와 비공개 버퍼 보관

## 수정 전 근거와 범위

차트 숫자 회귀를 고정한 `7ad99f00cf`에서 focused Clippy는 root lib 오류 두 건으로 실패했다.
`logs/chart-axis-regression-1029-clippy.log`의 실제 진단을 기준으로 한다. 원 기여자의 주장을
검증한 결과나 새 차트 회귀의 실행 실패와 구분하며, 기대값·골든·경고 허용으로 우회하지 않는다.

- `pua_glyph.rs:193`: 후보·fallback 순서에서 처음 지원하는 글리프를 고르는 수동 반복문이다.
  같은 iterator의 `find`로 바꿔 반환 순서와 미지원 `None` 계약을 보존한다.
- `render_tree.rs:400`: `TextRun` 672바이트와 다음 `Path` 464바이트의 차이가 208바이트다.
  이 브랜치가 추가한 `BidiLayout`의 비공개 소유 버퍼 세 개가 `TextStyle`에 inline으로 들어간다.
  렌더 노드 전체나 무관한 기존 개체를 box하지 않고 완료된 방향 결과의 보관을 줄인다.

## 생산자와 소비자

`text_direction::resolve_page`에서 원문·논리 전진·표시 구간을 가변 버퍼로 구성한 뒤
`BidiLayout`에 넘긴다. 이후에는 `VisualTextPositions::for_style`이 원문·전진의 동일성을
확인하고 같은 표시 구간을 빌려 쓰며 변경하지 않는다. 따라서 완료된 비공개 `String`·
`Vec<f64>`·`Vec<BidiVisualRun>`을 `Box<str>`·`Box<[f64]>`·`Box<[BidiVisualRun]>`으로 보관한다.
생산 중 계산, 소유·순서·좌표, 외부의 `Option<BidiLayout>`, 직렬화 필드는 그대로 유지한다.
문자열·전진이 바뀌었을 때 국소 방향 해석으로 되돌리는 반례도 보존한다.

## 검증 경계

기존 방향 진단·PUA 공개 API 회귀와 같은 chart 숫자 회귀를 묶어 실행한다. 완료 결과의
클론·직렬화·이후 표시 소비를 확인하며 새 수치·절대 px·골든을 추가하지 않는다.
같은 공유 `target/pr-review`, 빌드 잡 8개를 사용한다. fmt와 Clippy를 다시 확인한다.
현재 진행 중인 Native075/fresh028의 결과는 앞선 코드 `a19d7c3d16`의 증거로 유지하고,
이 lint 보정 뒤 출력의 최종 Native/fresh gate로 재귀속하지 않는다. 전체 제출 전 재검증은 남아 있다.

## 실행 결과

1035에서 실제 수정 전 lib로 방향 진단 2건을 실행해 모두 통과했다. 보관 변경은 동작을
바꾸는 결함 수정이 아니므로 수정 전 실패를 주장하지 않는다. 수정 뒤 focused Clippy는
exit 0, 기존 PUA 공개 API 회귀 3건과 새 차트 관계형 회귀 6건도 9PASS였다.

1039에서는 해당 lib의 실제 Cargo artifact에서 의존성을 골라 방향 진단을 다시 실행했고
2PASS였다. 초기 별도 진단의 컴파일 오류는 서로 다른 serde 의존성 조합 때문이며 제품의
결함 재현으로 세지 않는다. mixed Arabic·Hebrew·짧은 Arabic 문서 3개에서 RenderTree JSON과
SVG 6개 모두 전후 바이트·SHA-256이 같았다. clone과 같은 길이·폭의 표시 문자열 교체도
검사했다. 실제 크기는 `BidiLayout` 88→64, `TextRunNode`와 `RenderNodeType` 672→648바이트다.

새 차트 검사 파일도 정식 수집 뒤 포맷을 적용했다. 소스 바이트 크기에 따른 suite 배정이
바뀌어 초기 manifest 검사에서 drift를 확인했고, 실행 중 Cargo가 종료된 뒤 생성기로
다시 준비했다. 생성 harness를 직접 편집하거나 커밋하지 않았다. 고정 base
`2e3b4f880bd11d35398e6ea0100846611f13a72d`의 manifest 검사와 fmt check는 모두 exit 0이다.
manifest는 1,659 source, 정적 test attribute 7,185개, 28 suite+20 exception=48 target을 확인했다.
9건 실행과 최종 생성 상태의 정합 검사를 구분하며 전체 nextest와 최종 Native/fresh는 남아 있다.

근거는 ignored output의 `renderer-lint-1035-results.json`,
`renderer-lint-1035-wire-equivalence.json`, `renderer-lint-1039-results.json`이다.
로그는 `logs/renderer-lint-1035-{clippy,tests}.log`,
`logs/renderer-lint-1039-{metadata,compile-after,bidi-after,manifest,fmt-check}.log`다.

같은 시점의 원격 읽기 전용 감사 1036은 열린 semanticist21 PR 45개·Draft 10개를 확인했다.
기존 1002 대비 추가·제거·head 변경은 없고 devel도 같은 `2e3b4f880b`라 리베이스하지 않았다.
시각 미달·숫자 생략의 추가 반례와 모든 PR의 최종 검증을 계속하며 원격 변경은 수행하지 않았다.

# Task M100 #3739 2단계 — HWP3 IR 차이 보정 완료

## 목표

1단계 커밋 `cd09580fb` 뒤에도 `HWP3-password-123456.hwp`는 HWPX 산출과 24쪽 재열기는
성공했지만 `--verify`에서 15건의 IR 차이로 exit 3이었다. 이 단계는 각 차이를
원인별로 분석하고, 실제 HWPX 표현을 보존한 뒤 검증 가능한 차이만 최소 범위로
정규화한다.

## 분석과 구현

| 원인 | 관측 | 조치 |
|---|---|---|
| HWP3 개체 위치 | `U+FFFC`는 text에는 1단위지만 암호 표본의 offset에는 8단위 슬롯으로 기록됨 | 해당 표식을 `<hp:t>`로 쓰지 않고 control 슬롯으로 치환하고 슬롯 수 추정을 보정 |
| HWP3 하이퍼텍스트 | 종전 HWPX serializer가 `Control::Hyperlink`를 버려 이후 `char_shapes`가 8단위 밀림 | `fieldBegin type="HYPERLINK"`와 `Command` parameter로 HWPX field를 방출 |
| URL 누락 링크 | HWP3 추가정보에 URL이 없으면 `url`이 빈 문자열 | 표시 문자열을 Command fallback으로 보존 |
| 그림 영역 | HWP3 `[0,0,0,0]`는 crop 정보 없음 센티널, HWPX는 실제 사각형을 요구 | 정확한 canonical 물질화 형태만 HWP3→HWPX 검증에서 제외 |

개체 슬롯 보정 뒤 차이는 15건에서 그림 1건·하이퍼링크 위치 1건으로 줄었다. 하이퍼링크
방출 뒤에는 HWPX 재파서의 `Field` 표기 2건이 새로 관찰됐으며, 둘은 HWP3의 비슬롯
`Hyperlink`와 같은 정보를 표현한 것이다. HWP3 전용 정규화는 이 `[field]` 한 개와 빈
`imgRect`의 canonical 물질화만 제외한다. 다른 컨트롤·`curSz`·`imgDim`·복합 기하 차이는
계속 실패한다.

## focused 검증 (Windows PowerShell)

| 검증 | 결과 |
|---|---|
| `cargo build --profile release-test --target-dir target\pr-review --bin rhwp` | 통과 |
| `HWP3-password-123456.hwp --password 123456 export-hwpx --verify --verify-pages` | exit 0, IR 무차이, 24쪽 |
| `cargo test --profile release-test --target-dir target\pr-review --lib issue_3739 -- --nocapture` | 4 passed |
| HWP3 URL 누락 fallback field 단위 테스트 | 1 passed |
| `cargo test --profile release-test --target-dir target\pr-review --test issue_3739_hwpx_same_char_shape_boundary -- --nocapture` | 4 passed |
| 변경 Rust 파일 `rustfmt --check`, `git diff --check` | 통과 |

통합 테스트는 HWP3의 BOM 포함 `--password-stdin`에서도 `--verify --verify-pages`를
통과하도록 고정한다. HWP5·HWPX 암호 표본은 기존처럼 BOM stdin과 페이지 재열기를
계속 확인한다.

## 범위 밖

- 전체 baseline·clippy·PR CI 성격 검증, 원격 push·PR 생성·merge
- 한컴 GUI 수동 판정

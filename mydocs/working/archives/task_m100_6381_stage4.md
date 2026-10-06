# Task M100 #6381 Stage 4 완료보고 — PR #6391 review 보정

- **이슈**: [#6381](https://github.com/edwardkim/rhwp/issues/6381)
- **PR**: [#6391](https://github.com/edwardkim/rhwp/pull/6391)
- **review**: [issuecomment-5464292086](https://github.com/edwardkim/rhwp/pull/6391#issuecomment-5464292086)
- **최신 기준**: `upstream/devel@2deb3dd6163d83d2932ab58ac5a0bf61bfce6d31`
- **merge commit**: `0240e043ebaa661773056980ffe341adc431ccab`
- **보정 code candidate**: `d8ab820b065618966dfb67969cf2c1b1ba26992a`
- **상태**: 로컬 보정·전체 검증 완료, trailing 기록과 원격 CI 대기

## 1. review 판단과 보정 순서

review가 지적한 핵심 두 항목을 먼저 재현 가능한 계약으로 좁혔다.

1. setter는 `Control::Picture`뿐 아니라 `Control::Shape(ShapeObject::Picture)`와 본문 뒤 Endnote 가상
   문단을 해석하지만, 기존 verifier는 본문 `Control::Picture`만 읽었다.
2. 기존 성공 test는 exit 0과 SVG만 확인해 verification 블록을 삭제해도 통과할 수 있었다.

그 뒤 상수·중복 lookup·임시 파일 정리를 보정하고, 마지막으로 help·capabilities·CLI 정본에서 명령의
고정 fixture 범위를 명시했다.

## 2. 제품·회귀 보정

- `resolve_picture`가 setter와 같은 좌표·picture 표현을 읽고, `verify_caption`이 방향·세로 정렬·폭·간격을
  한 경계에서 비교한다.
- 네 expectation과 폭·간격을 상수화하고 mutation 성공 vector 길이를 expectation 수에서 계산한다.
- 성공 subprocess test는 `caption=Some(...)` 네 줄을 필수 증적으로 요구한다.
- `Shape(Picture)` setter/getter test는 Right·Center·8504·850 필드를 재조회해 고정한다.
- CLI catalog test는 `Shape(Picture)`와 Endnote 해석 topology가 verifier에서 사라지는 회귀를 막는다.
- 임의 실문서 all-fail test도 RAII fixture 정리를 사용한다.
- HWP5 export가 `Shape(Picture)`를 `Control::Picture`로 정규화하므로 실제 CLI용 Shape fixture를 가장하지
  않고 model setter 계약과 verifier topology를 분리해 검증했다.

## 3. 자기서술과 범위

help·capabilities·`mydocs/manual/cli_commands.md`는 `test-caption`을 “고정 fixture 캡션 라운드트립 검증”으로
설명한다. 고정 좌표를 일반 문서의 picture 탐색으로 바꾸지 않았으며 renderer·layout·SVG 의미도 변경하지
않았다. 따라서 visual sweep은 적용 대상이 아니다.

## 4. 로컬 검증

| 게이트 | 결과 |
| --- | --- |
| integration prepare/check | 1,032 sources / 4,535 attrs / 48/48 targets, 통과 |
| unit tier check | 4,221 tests / 299 modules, 통과 |
| focused `test-caption` | 5/5 pass, run `bd1bcaa0-dd48-415d-ab11-5a325cdd718d` |
| focused CLI catalog | 20/20 pass, run `c7ba0e7a-ec8f-4b5b-a9ca-4643d1a6078e` |
| native Clippy | `-D warnings` 통과 |
| WASM32 library Clippy | `-D warnings` 통과 |
| workspace build | 통과 |
| workspace all-target Clippy | `-D warnings` 통과 |
| 전체 release-test nextest | 8,686/8,686 pass, 43 skipped, 1 slow, 135.413초 |
| 전체 nextest run | `554b5740-99fd-450e-982d-62c9c8810420` |
| format·diff·Markdown link | 통과 |

저장소 전체 문서 메타데이터 검사는 이번 diff 밖의 기존 4개 문서에서 16건을 보고했다. 변경한 문서의
신규 오류는 없으며 generated suite·manifest와 `target/`, `output/`은 제출 diff에 포함하지 않는다.

## 5. 원격 게이트

보정 code candidate와 이 trailing 기록을 push한 뒤 PR comment로 지적별 반영 내용을 게시한다. 새 head의
required GitHub Actions와 mergeability 확인 전에는 merge하지 않는다.

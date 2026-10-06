---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7155_7168_review_impl.md
last_verified: 2026-09-15
---

# PR #7155·#7157·#7165·#7166·#7167·#7168 통합 검토 실행 계획

## 현재 결과

최신 upstream/devel `da7ec5c0906b38b7a1cfa15cf70ca5534b7d7ebb` 위
`codex/pr7155-7167-review-20260915`에서 6개 원 PR을 출처 보존 체리픽했다.
리뷰어 jangster77을 원 PR마다 먼저 요청했고 API로 재확인했다. draft #7141/#7118은 제외했다.
#7168 추가 요청도 반영했다. 이 문서는 원격 통합 PR이나 merge 완료를 의미하지 않는다.

## 커밋과 충돌

| commit | 제목 |
| --- | --- |
| `abd1721d62ffb5d1e7081d3f35f9e2eb68774983` | 수정: HWP3 고정폭 빈칸·하이픈을 제어 표기로 보존한다 (#4680) |
| `536f06f1f5df1d7f43332f3df3382b5ba21e363b` | 수정: HWP3 제목 차례 표시를 버리지 않는다 (#4680) |
| `8004fdde66279e3a5055885af753b6b3429db6ca` | fix(renderer): preserve inline table ownership before a text line boundary |
| `e899a3b1a2c7dea5d00787e212db9e02610e49ab` | test: include synthetic stored-line fixture |
| `cfb5aedf009c1af038c778754318c85e2184c421` | fix(renderer): reuse painted picture ownership in split cells |
| `2faa8b38dd1af3bfa7cb7e8e65945ae9defc3494` | fix(renderer): keep spaces before inline objects in line distribution |
| `585d6b69dc6df55c1435dd7f9688887c99902e5b` | feat(i18n): 대화상자 영어 표시 (3/4) |
| `416da982d21b80414eaa321a0c301215f6bf91c3` | refactor(ui): 대화상자 탭을 이름 대신 ID 로 구분 (#5852) |
| `a86e96bb84478d204ff3d40b65d4010ad6049258` | test(i18n): 통합 탭 검사가 번역 카탈로그를 인식하도록 보정 |
| `36e41e3da52d0bfaeb1c4bd558e2d70478abd271` | fix(renderer): preserve stored rows at the unquantized column width |
| `e373698acdcd87d6cadb5eaf965d4a76a87482ae` | test(renderer): verify stored header horizontal placement |
| `4b19c3d28a4e7347fa95945d623d8489c25b6f32` | test(renderer): distinguish signed row widths from page units |
| `b5d7956f6eb24a5025476300024bec86059a7c17` | style: format stored width regression test |
| `925cd7434564657f1c78cf5568bcc05b0f19e9d3` | test(renderer): 합성 입력의 독립 한컴 PDF와 불일치 증거 보존 |

#7157의 선행 #7155 commit은 중복 체리픽하지 않았다. #7167에서 equation-props의 import/ID 선언과
page-border의 번역 label/ID 충돌 두 곳을 수동 해소했다. 원 기여자 author와 `-x` provenance를 보존했다.
번역된 label을 거부하던 테스트는 a86e96bb8에서 기존 i18n assertion helper로 보정했다.
925cd7434는 실제 한컴 변환 PDF와 출처 기록만 추가하며 합성 HWPX의 바이트를 변경하지 않는다.

## 검토 판정

| 원 PR | 판정 | 근거 |
| --- | --- | --- |
| [#7155](pr_7155_review.md) | 승인 | HWP3 고정폭 빈칸·제어 표기 보존 |
| [#7157](pr_7157_review.md) | 승인 | 제목 차례 표식·오프셋 보존 |
| [#7165](pr_7165_review.md) | 승인 | 독립 한컴 4경우 확보·첫 줄 개행의 중복 빈 줄 보정 |
| [#7166](pr_7166_review.md) | 승인 | 정적 대화상자 번역 3/4 단계, 일부 한글 잔존 명시 |
| [#7167](pr_7167_review.md) | 승인 | 탭 ID 및 번역 통합 보정 검증 |
| [#7168](pr_7168_review.md) | 승인 | 잘못된 폭 캐시 수용 제거·연속 공백 재조판 보정 |

## 최초 검토 경로와 결과

macOS에서 `target/pr7155-7167-review-20260915`를 독립 사용했다. 기존 target은 보존했다.
`node scripts/rust-test-suite-manifest.mjs --prepare` 후 이름별 `scripts/run-rust-test.mjs`를 사용했다.
처음 여러 이름을 한 번에 넘긴 호출은 인자 오류로 실행되지 않았고 이름별로 다시 실행했다.
기존 4개 집중 사례는 #7168 반영 후 다시 확인했다. Rust focused 합계는 9 passed다:
HWP3 control 2, title 3, stored inline 1, #6122 1, #6706 1, column quantization 1(내부 12조건).

- Native: `cargo build --locked --profile release-test --target-dir target/pr7155-7167-review-20260915` 성공.
- WASM: `CARGO_TARGET_DIR=target/pr7155-7167-review-20260915 scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev` 성공.
- Studio unit: 1740 passed / 2 skipped, 두 TypeScript 검사 성공. 최초 literal guard 실패 1건은 기록·보정했다.
- 실제 Chrome: 4종 대화상자×ko/en, 표시 문구 변경 후 ID 기반 탭 전환 성공.
- 기존 OLE 객체 선택 E2E, undo-contracts 5개 묶음 성공. undo 로그에는 Through 시 커서 위치 경고가 있었으나 assertion 실패는 없었다.
- fresh WASM/native visual sweep: two_digits p1, header p1, #6122 p6. #6122 native p5/p7도 baseline PNG 동일 확인.
- #6122 fidelity: 12쪽 전체 text-only/export-all-svg/layout-ledger 실행, 문자 누락·추가 후보 0.
- source 6개 PR은 마지막 확인한 각 head에서 pending/failed 없음. 최초 검토 당시 통합 head 원격 CI는 미실행이었다. 이후 결과는 아래 최종 CI 기록을 참조한다.
- 사용자 지시에 따라 광범위 전체 Rust/Native Skia 회귀를 중복 실행하지 않았다. 통합 CI 생략을 뜻하지 않는다.

수정 전 binary는 이전 검토 3e6c1b36c 산출물이며 SHA-256은
`124b4123b2d1d6c2c76141bf5df73006601858c07098ef1395a339d8b692b43d`다.
해당 commit과 현재 base의 src/crates/Cargo.toml/Cargo.lock 차이가 없음을 확인했다.
새 render binary는 b5d7956f6 소스에서 빌드했고 이후 925cd7434에는 실행 코드 변경이 없다.
각 바이너리/입력/PNG hash, source CI와 sweep 수치는
[증적 manifest](../assets/pr7155_7168_review_evidence.json)에 기록했다.

## 단계별 후속 처리

1. 완료: 분석 → source 체리픽·충돌 보정 → 집중/브라우저/독립 PDF 검증 → 결과 보고 → 코드·fixture 커밋.
2. 완료: 개별 review 6개와 이 계획, 대표 PNG를 작성·검사하고 최종 code CI 성공 뒤 오늘할일과 함께 trailing commit에 포함한다.
3. 완료: #7165/#7168의 독립 한컴 저장 입력을 확보하고 코드 보정·집중/시각 검증·결과 보고·커밋을 완료했다.
4. 완료: 사용자의 진행 지시에 따라 upstream 임시 codex/ branch로 push하고 devel 대상 PR #7171을 만들었다. owner를 자동 reviewer로 지정하지 않는다.
5. 통합 최종 code head CI 성공 뒤 이 개별 review·오늘할일·PNG를 같은 PR에 trailing commit한다. 통합 PR 번호의 별도 review 문서는 만들지 않는다.
6. merge 단계의 승인 범위와 최신 head/required checks/mergeability를 재확인한다. source PR을 먼저 닫지 않는다.
7. merge 후 해당 원 PR에 SHA·검토/시각 증적을 코멘트하고 필요한 source close를 수행한다. #4680/#5852는 부분 해결이므로 전체 종료하지 않는다.
8. post_merge.md에 따라 duration refresh·devel 동기화·본 작업 소유 branch/target cleanup을 한다. 다른 task 산출물이나 기여자 fork branch는 삭제하지 않는다.

부분 제외가 필요하면 새 최신-base branch에 승인된 source commit만 다시 누적한다. 현재 검토 branch를 강제 초기화하거나
다른 작업의 파일을 지우지 않는다. #7157은 #7155를 선행 적용하고, #7166/#7167의 충돌·guard 보정은 함께 고려한다.

## 추가 기존 입력

| 파일 | SHA-256 | 입력 commit |
| --- | --- | --- |
| [samples/SO-SUEOP.hwpx](../../../samples/SO-SUEOP.hwpx) | `ebbd6f5c86d6eda195bb9c4ba1fc35b329432dcc5ef1dc0fffbfb4542fce74b8` | `7393203034a92fdbb241812458b539753eb01985` |
| [samples/한셀OLE.hwp](../../../samples/한셀OLE.hwp) | `fdc595a2f5f99f97653b91e57b4c74090391495ce5f84ef2e60308ea4b8e9da3` | `1dc5a5706bfaf31115f2455829e9b135607d050a` |
| [samples/hwp3-sample16-hwp5.hwpx](../../../samples/hwp3-sample16-hwp5.hwpx) | `49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c` | `dcf64b4da051ea7b0bf164cf20e1935faa0250ae` |

## 메인터너 보정 회차 완료

- `42ef2240d`: 단일 페이지 visual sweep이 파일명의 연도 2020을 쪽 번호로 오인하지 않도록 수정. Python 테스트 51 passed.
- `0c80b8f1c74ff2f9510db335efb018db427af45c`: #7168 잘못된 폭 캐시 수용 제거와 연속 공백 경계 보정.
- `b5f8bf9553976a9aedacda54030df5f7ce0d0045`: #7165 첫 저장 줄 뒤 중복 빈 줄 제거와 독립 한컴 HWP 4경우/PDF 추가.

보정 소스에서 focused 10개, 줄 나눔/frame 단위 21개, fmt·Native/WASM/workspace Clippy,
workspace build·suite manifest·unit-tier 검사를 통과했다. Native/fresh WASM 최종 빌드 완료,
머리글 p1·인라인 표 4경우 p1·#6122 p6의 6페이지 visual sweep에서 Native/WASM raster가
모두 바이트 동일하다. #6122 p6은 메인터너 보정 전 Native와도 동일하다.

[보정 증적 manifest](../assets/pr7165_7168_maintainer_evidence.json)에 실행·입력·출력 해시,
실패 후 보정 이력과 기존 표 테두리/높이 차이를 기록했다. 비공개 실제 양식은 확보하지 못했으며
독립 한컴 저장 축소 사례의 검증으로 원본 전체 개선을 주장하지 않는다. source CI와 통합 CI는
구분한다. **전체 6개 원 PR의 로컬 검토는 승인, 통합 PR #7171의 최종 code CI도 성공**다.

코드·검증 HWP/HWPX/PDF·fixture 결과보고는 커밋했고 review/PNG/오늘할일은 기존 순서대로
통합 code CI 뒤 같은 PR의 trailing commit으로 처리한다.

## PR 생성과 CI 모니터링

[통합 PR #7171](https://github.com/edwardkim/rhwp/pull/7171)을 생성했다. 제출 직전 #7166의
추가 영문 문구 commit `60a3626c1`을 `95d631f0a`로 체리픽했고 관련 테스트 23개가 통과했다.
Rust 소스는 최종 메인터너 검증 때와 해시가 같으며 추가 code 변화는 영어 카탈로그 한 행이다.
code candidate Full CI 성공 후 준비된 review·오늘할일·증적을 trailing commit한다.
사용자로부터 최종 CI 이후 merge·후속처리까지 승인받았다.

## 최종 code CI 확인 (2026-09-15)

최종 code head `5a04e172247d1b168d350faf514beeef46cf0964`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/34978539946),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34978540233),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34978539840),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/34978539982),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/34978540234)가 성공했다.
확인 시 check 31개 성공·4개 조건부 생략, policy status 성공이며 실패·진행 중 항목은 없다.
Rust archive 4개·전체 shard, Native Skia, 필수 lint, frontend package 검증을 포함한다.
초기 `95d631f0a`의 source-side test 정책 실패는 `5a04e1722`에서 동일 assertion을 기존
테스트 계약으로 묶어 해결했다. 허용치·baseline·생산 코드는 변경하지 않았다.
이 문서·오늘할일·대표 증적을 single-parent trailing commit으로 반영하고, 문서 추가 후
최종 head의 fast-pass와 required checks·MERGEABLE/CLEAN을 별도로 확인한 뒤 병합한다.
최종 head 및 실제 merge SHA·후속 처리 결과는 PR #7171과 원 PR의 GitHub comment에 기록한다.

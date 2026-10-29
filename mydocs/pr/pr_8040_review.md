# PR #8040 검토 — 반복 저장과 편집 후 머리말 상속

## 최종 판정

**머지 보류 — 저장 문제는 최신 devel에서 해결됐지만, 뒤 구역 편집 후 짝수 쪽 머리말 소실은 재현된다.**

후속 상태: 메인테이너가 2026-10-29에 직접 보정·검증·머지를 승인했다. 아래 최초 조사와 안내 초안은 당시 기록이다. 기여자에게 재작업을 요청하지 않고, 원 head를 merge parent로 보존하면서 저장 보정의 tree 변경을 제외한다. 머리말 변경은 현행 `collect_header_footer_controls_with_storage`를 공유하도록 적용하며 Native/fresh WASM·편집 저장본의 독립 Print와 필수 로컬 검증을 진행한다. 검증 전 승인 판정으로 바꾸지 않는다.

원 PR 전체를 해결 완료로 닫거나 그대로 병합할 수 없다. 첫 번째 저장 보정 commit은 제외하고 두 번째 머리말 수정만 최신 devel의 자원 정책에 맞춰 재작업하는 것이 권고안이다. 현재 원 head는 3개 소스 파일에서 충돌한다. 편집 후 독립 Print·Native/fresh WASM 검증도 제출되지 않았다.

이번 단계는 이미 해결됐는지에 대한 조사와 원 head 검토다. 제품 소스·기존 검사·기대값은 변경하지 않았으며, 원격 comment·push·close·merge는 수행하지 않았다.

## 접수와 기준

- 조사일: 2026-10-29 KST.
- [원 PR #8040](https://github.com/edwardkim/rhwp/pull/8040), 작성자 `h1-andro2d`, OPEN/non-draft, base `devel`.
- 원 head: `2cfe84e5c72c430057d63c295a6103c29164440e`.
- PR 분기점: `224a326063a5aeab5a1ca8dcaa92918f55843ffc`.
- 고정한 최신 upstream/devel: `784ef465d8867efa8a7b6361175e7254c3bafac4`.
- reviewer `edwardkim`을 지정하고 requested_reviewers API로 확인했다. 작성자의 기존 병합 PR은 조회되지 않았고 전체 PR 조회에서는 #8040만 확인됐다.
- 기본 경로 `maintainer_general`; 보조 경로 `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `first_time_contributor`, `rework_and_exceptions`. 첫 기여자 안내의 외부 PR 절차도 확인했다.
- 원 PR은 관련 이슈가 없고 두 결함을 처음 보고한다고 명시한다. 관련 선행 #5802·#7994와 실제 통합 이력을 별도로 대조했다.

## 1. 반복 저장 문제: 제시된 두 재현은 이미 해결

원 첫 commit `9df69cd0b88e37442b3f031f188d7106e5d98471`은 파서가 간격을 반감하고 누적 vpos를 차감한다는 전제에서 저장 시 두 배 복원·bias 가산을 추가한다.

하지만 [#8064](https://github.com/edwardkim/rhwp/pull/8064)에 포함된 `b9bb438d99df6137c2ec6e8fb09d8d98eb2cb681`이 두 파서 변환을 제거했다. 실제 devel 통합은 2026-10-19의 `9b01d5d1447220b3a7345703e79ad42cf3f502f4`다. PR #8040 작성 다음 날의 통합이므로 당시 보고가 잘못됐다는 뜻은 아니다.

현행 `src/parser/mod.rs:577–598`은 HWP3 계보 표시만 설정하고 HWP5 원본 간격·저장 좌표를 보존한다. 과거 `normalize_variant_paragraph_vpos` 함수와 호출은 없다.

같은 전제 차이는 #7994를 다룬 원 PR #8028에서도 확인됐다. 후속 메인터너 보정 `60500e68eedf3aadce82720359e3e2a2341a7b7f`와 [#8148](https://github.com/edwardkim/rhwp/pull/8148)의 실제 통합 `2e087a5b4bc1515ad877e8020b30114639a59392`를 추적했다. [독립 인쇄 재검토 기록](assets/issue7445/deferred-20261020/source-pr8028/README.md)은 중복 간격 복원 때문에 같은 한컴 소비자의 출력이 24→27쪽이 되고, 이를 제거한 뒤 24쪽으로 복원됐음을 기록한다. 이 과거 HWPX 증거를 이번 HWP5 저장의 새 시각 검증으로 사용하지 않는다.

현재 devel에서 원 PR의 테스트 파일을 변경 없이 임시 진단 하네스로 실행했다.

| 재현 검사 | 결과 |
| --- | --- |
| `rewritten_section_vpos_survives_repeated_saves` | PASS: raw section 재사용을 끈 1회·2회 저장 후 모든 본문 vpos 보존 |
| `edited_para_shape_keeps_spacing` | PASS: 문단 정렬 편집 후 저장·재열기에서 위/아래 간격 보존 |
| 추가 관측: 정렬 편집 후 반복 저장 | 위/아래 간격 `(1136, 2264)`가 0·1·2회 모두 동일 |

따라서 첫 commit의 모델 bias 필드·파서 기록·진단 제외·저장 복원을 최신 기준에 추가할 필요가 없다. 현행 파서를 유지한 채 원 저장기만 적용하면 `saturating_mul(2)`와 vpos bias가 정상 저장값을 바꿀 위험이 있다. 이는 코드 검토상의 부적합 판정이며, 충돌을 임의 해소한 후보에서 새 회귀를 실행했다고 주장하지 않는다.

## 2. 머리말 상속 문제: 최신 devel에서도 미해결

재현 순서는 원 PR과 동일하다. `samples/exam_social.hwp`를 열고 0-based 구역 1의 첫 긴 문단(현재 paragraph 0), 문자 offset 1에 `가`를 삽입한 뒤 같은 글자를 삭제한다.

| 관측 | 편집 전 | 삽입·삭제 후 |
| --- | --- | --- |
| 문서 쪽수 | 4 | 4 |
| 2쪽 `(사회·문화)` | 있음 | 없음 |
| 4쪽 `(사회·문화)` | 있음 | 없음 |
| 3쪽 홀수 머리말 `사회탐구 영역` | 있음 | 있음 |

`editing_later_section_keeps_inherited_even_header`는 FAIL이다. 쪽 텍스트뿐 아니라 `render_page_svg_native`의 실제 SVG도 비교했다. SVG의 개별 `<text>` 노드를 순서대로 합친 결과 2·4쪽의 `사회·문화`가 편집 전 존재하고 편집 후 사라진다. SVG는 글자별 노드로 나뉘므로 원 XML의 연속 문자열 검색 실패를 글자 소실로 판정하지 않았다.

원인은 현재 `src/document_core/queries/rendering.rs:5398–5430`의 not-dirty 경로가 실제 페이지의 `active_header`만으로 carry를 복원하기 때문이다. 앞 구역에 정의됐지만 그 구역의 홀수 1쪽에서는 사용되지 않은 Even 머리말이 빠진다. 반면 dirty 경로 `5733–5800`은 구역 컨트롤 정의를 순회한다. 뒤 구역의 같은 홀짝 슬롯 선택은 `5852–5871`과 `5983` 이후에서 이 carry를 소비한다.

선행 [#5802](https://github.com/edwardkim/rhwp/issues/5802)는 [#5836](https://github.com/edwardkim/rhwp/pull/5836) → [#5844](https://github.com/edwardkim/rhwp/pull/5844), 통합 `1b8c39b6c32baf00574564016bd9d9f7d1d88198`을 통해 슬롯별 상속을 보정했다. 현재 `tests/cases/issue_5802_hf_cross_section_inherit.rs`는 최초 열기의 상속을 검사하며 뒤 구역만 편집하는 경로는 실행하지 않는다. 해당 이슈의 종료는 이번 증상의 해결 근거가 아니다.

두 경로가 같은 구역 정의 수집 결과를 소비하게 하는 원 두 번째 commit의 방향은 타당하다. 단, 최신 devel의 `storage.push`·`storage::try_clone`·오류 전파를 유지해야 한다. 원 head의 무조건 `Vec::push`·`clone`으로 현행 경계를 되돌리면 안 된다. 중첩 표의 머리말, Both/Odd/Even 슬롯, 앞 구역의 정의가 없는 경우, 첫쪽 감추기의 기존 계약도 유지해야 한다.

## 실행 방법과 증거 범위

최신 devel에서 Rust 1.93.1, Linux/WSL2의 공용 target을 사용했다. 별도 작업 트리와 target을 만들지 않았다.

```bash
CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review \
CARGO_BUILD_JOBS=6 CARGO_PROFILE_RELEASE_TEST_OPT_LEVEL=1 \
CARGO_PROFILE_RELEASE_TEST_CODEGEN_UNITS=64 \
cargo build --locked --profile release-test -p rhwp --lib --features native-skia --message-format=json
```

Cargo가 현재 소스의 라이브러리를 fresh로 확인했다. 원 head의 `tests/cases/hwp3_variant_resave_file_units.rs`와 `tests/cases/header_carry_later_section_edit.rs`를 `git show`로 임시 `output/pr-review/pr8040/`에 추출하고, 두 모듈을 포함한 진단 파일을 `rustc --edition=2021 --test`로 링크했다. `--extern rhwp=target/pr-review/release-test/librhwp.rlib`, `-L dependency=target/pr-review/release-test/deps`, `CARGO_MANIFEST_DIR`는 저장소 루트다. `--nocapture --test-threads=1` 실행 결과 **2 PASS / 1 FAIL**, test exit 101이다.

이는 검토용 진단 실행이며 `tests/cases/`·suite·manifest에 새 회귀를 등록하지 않았다. 빌드·실행 로그와 SVG는 임시 산출물이며 정리 대상이다. 기존 자동 수집 경로는 `scripts/run-rust-test.mjs`와 suite manifest이고, 정식 회귀 제출 시 그 경로를 사용한다.

| 입력 | SHA-256 |
| --- | --- |
| `samples/HWP5-nopassword-123456.hwp` | `a34ecb8cde85b6db49c64a954cb7fa5d23b5f49367bc4753c90bfe89a075b50d` |
| `samples/exam_social.hwp` | `ef1f84e7dd213194e68c243a67b0b60022472d9fa81e4db8dcf9bd9be0825146` |

두 입력은 원 PR head와 현재 devel에 같은 바이트로 추적되어 있다. 파일 부족으로 재현을 생략하지 않았다.

이번 진단은 기본 글꼴 초기화의 Native API·SVG 내용 관측이다. 독립 한컴 Print와의 배치 일치, 실제 글꼴 윤곽, Native/fresh WASM 시각 통과를 입증한 것이 아니다. 원 PR도 편집 후 Print·fresh WASM을 미검증으로 명시한다. 편집 없는 SVG 동일성은 편집 후 머리말의 검증을 대체하지 않는다. 전체 Rust 회귀·새 코드 lint·WASM 빌드를 이번 조사에서 반복하지 않았다.

## CI·충돌과 다음 처리

원 head의 조회된 CI 결과는 SUCCESS 또는 정상 SKIPPED/NEUTRAL이지만, 현재 GitHub 상태는 `CONFLICTING/DIRTY`다. 최신 devel과 `git merge-tree --write-tree`에서도 `src/document_core/queries/rendering.rs`, `src/parser/mod.rs`, `src/serializer/cfb_writer.rs`의 충돌을 확인했다. 작업 트리에서 충돌을 적용하거나 CI를 재실행하지 않았다.

1. 첫 저장 보정 commit을 제외하고 머리말 상속 수정으로 범위를 줄인다.
2. 최신 devel에서 공통 carry 수집과 현행 자원 정책을 보존한다.
3. 최초 열기·뒤 구역 편집·홀짝/중첩/감추기 대조군을 실행한다. 같은 편집 저장본의 독립 Print와 Native/fresh WASM 전쪽 TSV·확대 판독을 완료한다. 기존 회귀의 기대값은 통과 목적으로 바꾸지 않는다.
4. 수정 head의 필수 로컬 검증과 최신 CI 뒤 재검토한다. 현재 원 PR의 통과 CI를 수정 head의 검증으로 재사용하지 않는다.

## 기여자 안내 초안 — 미게시

rhwp 첫 기여와 구체적인 재현 검사를 보내주셔서 감사합니다. 최신 devel에서 두 문제를 나누어 확인했습니다.

- 저장 관련 두 재현 검사는 현재 모두 통과합니다. PR 작성 이후 병합된 #8064가 파서의 간격 반감과 vpos 차감을 제거했으므로 첫 번째 저장 복원 commit은 제외하는 편이 맞습니다. 같은 전제 변경은 #8028의 후속 검토에도 기록되어 있습니다.
- 머리말 문제는 여전히 재현됩니다. `exam_social.hwp`의 뒤 구역에 글자 하나를 삽입·삭제하면 2·4쪽의 `(사회·문화)`가 사라지며 Native SVG에서도 확인했습니다. 이 수정은 여전히 필요합니다.
- 머리말 수정만 최신 devel에 맞춰 재작업해 주시면 좋겠습니다. 기존 자원 한도 처리와 홀짝 슬롯·중첩 정의·첫쪽 감추기를 보존하고, 편집 후 독립 Print·Native/fresh WASM 증거를 보완해야 합니다. 원 PR은 현재 세 파일에서 충돌하므로 그대로 병합할 수 없습니다.

저장 문제가 후속 변경으로 해소된 것과 별개로, 편집 후 머리말 누락을 재현하고 공통 처리 방향을 제안해 주신 기여는 유효합니다.

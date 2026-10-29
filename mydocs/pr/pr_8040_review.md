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

## 메인테이너 보정 중 발견한 기존 편집 배치 차이

2026-10-29, source `1a7c5ad250`에서 현행 bounded collector를 공유하도록 통합했다. 편집 전 원본의 Native 전쪽 raw는 98.41/99.68/99.54/99.39%, base WASM은 98.92/99.86/99.66/99.58%다. 후보 Native의 편집 전 네 쪽 PNG는 base와 픽셀 단위로 같다.

동일한 삽입·삭제 후 저장본 `samples/pr8040/exam-social-insert-delete.hwp`를 한컴 2020으로 새 Print했다. MCP job `c1661260-a8de-4484-96d0-d93cc5549197`, 한컴 11.0.0.9136, `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`, 4쪽이다. 기준은 `pdf/pr8040/exam-social-insert-delete-2020.pdf`다. Native와 브라우저에 Windows 및 한컴 2020 실제 글꼴을 동일하게 공급했다.

| 편집 직후 live 출력 | 1쪽 | 2쪽 | 3쪽 | 4쪽 |
| --- | --- | --- | --- | --- |
| base Native | 98.42% | 56.24% | 54.84% | 50.84% |
| 보정 Native | 98.42% | 58.51% | 54.84% | 53.27% |
| base WASM | 98.93% | 56.27% | 54.72% | 50.86% |

보정 Native는 2·4쪽 머리말만 복원한다. 수정 전후 PNG의 차이 경계는 두 쪽 모두 `(70,83)–(959,133)`이며 1·3쪽 전체와 모든 쪽의 본문 픽셀이 동일하다. 삽입·삭제 후 저장 HWP도 base와 후보가 바이트 단위로 같다. 따라서 본문 배치 차이는 고정한 devel에 이미 존재하는 편집 재조판 결함이다. 저장본을 다시 열면 Native 전쪽 98.42/99.69/99.56/99.37%로 돌아오므로 최초 열기·저장본 재열기와 편집 직후 경로를 혼동하지 않는다.

후보 fresh WASM은 빌드·검증 중이며 아직 통과로 기록하지 않는다. 현재 전체 시각 gate는 `re_review_required`다. 기존 편집 본문 결함을 이번 PR 머지 차단 대상에서 제외하고 머리말 관계 회귀를 추가하려면, 전쪽 90% 및 신규 회귀 선행 조건에 대한 이 범위의 명시적 예외 결정이 필요하다. 일반적인 90% 기준이나 다른 문서에는 적용하지 않는다.

추가 직접 판독: 2·4쪽의 복원된 머리말은 편집 전 머리말과 픽셀 단위로 같다. 쪽번호의 가는 글꼴 등 독립 Print와의 기존 모양 차이를 새 보정이 만들지 않았다. 합성 IR 대조군은 Both/Odd/Even × 최상위/중첩 표 × 첫 구역 머리말·꼬리말 감추기 유무의 12조합에서, 정의 없는 중간 구역을 거쳐 뒤 구역을 편집해도 본문 표지 및 머리말·꼬리말 슬롯을 유지했다. 이는 실제 API 호출의 별도 진단이며 한컴 시각 일치 증거나 정식 신규 회귀 등록으로 승격하지 않는다. 원 기여자의 머리말 진단은 동일 후보 라이브러리에서 1 PASS다.

메인테이너 결정(2026-10-29): **예외 없이 본문 배치까지 함께 수정**한다. 90% 미만의 기존 본문 결함을 면제하는 안은 채택하지 않았다. 이후 범위는 동일 문서의 뒤 구역 텍스트 편집 직후 재조판 원인 보정, 같은 편집 저장본의 독립 Print 대조, 전쪽 Native/fresh WASM 90% 이상과 직접 판독, 관계 회귀 및 필수 로컬·CI 검증 완료 후 머지다. 머리말 단독 후보를 완료하거나 먼저 제출하지 않는다.

### 본문 원인 추적과 첫 보정

원 #6883의 `17e34fdca3`은 #6882의 셀 Enter 성장 문제 때문에 `session_edited`를 도입했고, #6944의 `74d0a68b74919761cc30343f7511dbc5d0fe32d3`으로 통합됐다. 후속 #7491은 성장 표의 앞줄·표 이월을 보정했으며 기존 Enter 8/20회 3/4쪽 계약을 유지했다. 두 검토 기록과 독립 Print 근거를 확인했다. 이 기존 성장 경로는 유지해야 한다.

현재 재현은 삽입·삭제 후 2구역의 본문 텍스트와 저장 LineSeg까지 원본으로 돌아왔지만, raw 스트림 무효화로 문서 전체 `session_edited`만 true다. 같은 IR의 캐시를 다시 구성해도 차이가 남고, 진단 사본에서 provenance 봉인만 제거해 이 플래그를 false로 만들면 전쪽 98% 이상으로 돌아온다. 이 조작은 원인 분리 진단이며 제품 보정으로 사용하지 않는다.

첫 원인 지점은 `typeset/controls/stored_tac.rs::prepare`다. 저장 TAC 줄의 원본 소유·현재 측정 높이·앞 본문 끝·동일 단 좌표축·가용 공간을 이미 검증하면서도 문서의 다른 곳이 편집됐다는 이유로 전체 계획을 거절한다. `controls.rs`가 이 계획의 동일 pen/end를 fit과 paint의 inline placement로 전달한다. 문서 전체 플래그 대신 기존 실제 유효성 검사를 사용한다. 실제로 자란 표는 현재 물리 높이와 선언 높이의 불일치로 거절되어 일반 분할 경로를 유지해야 한다. 먼저 원본과 편집 직후, #6882의 성장·복구 경계를 실행해 방향을 확인한다. 아직 이 추가 보정의 통과를 판정하지 않았다.

### 로컬 변환 글꼴과 편집 배치 보정

메인테이너가 제공한 `/home/edward/mygithub/ttfs/hft`의 387개 OTF를 로컬 검증에 사용한다. 동일 family 이름의 한글·영문·한자 프로그램이 분리되어 있으므로 디렉터리 나열만으로 공급 완료라고 판단하지 않았다. 원본 문서의 language slot/font ID와 실제 cmap을 대조하고 기존 `FontEnvironment.documentSubstitutions`에 실제 PostScript 이름·프로그램 SHA-256을 지정했다. Native와 Canvas는 동일 프로그램을 사용하며 글꼴 바이트·이름 테이블을 수정하지 않았다. 폰트는 로컬 사용 한정이므로 커밋·업로드하지 않는다.

보정한 값의 생산·소비 연결은 다음과 같다. 전역 편집 플래그를 끄거나 입력별 예외를 추가하지 않는다.

| 계약 | 생산과 측정 | 실제 배치와 반례 |
| --- | --- | --- |
| 저장 TAC 줄의 공통 pen/end | `stored_tac::prepare`의 원본 줄 소유, 현재 측정 높이, 앞 내용 끝, 가용 공간 검사 | `controls`의 inline placement를 fit과 paint가 공유. 실제 성장 표는 `measured_fits`로 거절 |
| 원본 본문 줄의 점유 끝 | `paragraph/content::stored_content_bottom`의 local dirty·현재 구성 메트릭·동일 단 축 | 같은 확정 끝을 후속 TAC 흐름이 소비. 편집·재구성된 줄은 거절 |
| 다음 저장 줄로의 전체 간격 | `composer::tac_next_line_full_spacing`의 현재/후속 줄 소유·닫힌 사다리 | fit/paint의 기존 공통 소비를 유지. 텍스트 또는 셀 서식 dirty이면 거절 |
| 저장 텍스트 셀의 안 여백 | `float_placement::stored_text_line_cell_minimum` → `HeightMeasurer::stored_cell_text_minimum` | `stored_plain_cell_content_minimum` → 행 높이·rowspan·셀 배치. 양쪽 모두 실제 셀 재조판과 공통 생산자의 dirty/합성 줄 검사를 사용 |

마지막 항목의 선행 원인은 #7445의 `1d28a62b073406076fcf6d4d947f26a6ce50e17a`, 리베이스 복원 Stage42, 실제 통합 #8191 `6097253468`의 소스와 독립 Print 근거를 대조했다. 문서의 다른 곳에서 편집하면 공통 생산자 앞의 전역 가드가 저장 텍스트 줄과 활성 패딩을 무효화했다. 이번 문서 4쪽 16번 내부 3행 표의 첫 행에서 3.76px가 빠지고 다음 두 행의 테두리·글자가 올라갔다. 측정과 배치가 동일한 셀별 유효성을 판단하도록 보정한다. 합성/실제 편집 셀에는 저장 계약을 강제하지 않는다.

진단 단계에서 1~3쪽 본문과 4쪽 중첩 표 밖의 PNG는 삽입·삭제 전후 픽셀 단위로 같아졌다. Native 전쪽은 이미 90% 이상이나 알려진 중첩 행 차이를 면제하지 않고 보정 중이다. 실제 셀 Enter 성장 8/20회와 되돌리기의 기존 검사 5건은 앞 세 보정에서 모두 통과했으며 마지막 보정 뒤 다시 실행한다. 최종 Native/fresh WASM 및 전체 회귀 통과는 아직 판정하지 않는다.

삽입만 한 저장본도 별도로 독립 인쇄했다. `samples/pr8040/exam-social-insert.hwp`와 `pdf/pr8040/exam-social-insert-2020.pdf`, MCP job `b04e86d5-061f-4141-bb42-771f2a978b5f`, Hancom 2020 Print, 4쪽이다. 중립 삽입·삭제뿐 아니라 문자가 실제로 남은 편집 결과도 대조한다.

마지막 셀 보정 후 Native 96DPI 전쪽 raw는 삽입/삽입·삭제 양쪽 모두 **98.37 / 98.49 / 98.58 / 98.49%**다. 삽입·삭제 후 네 쪽 PNG는 같은 글꼴 환경의 편집 전 PNG와 모두 픽셀이 같고, 실제 저장 바이트는 독립 Print를 요청한 두 fixture와 각각 SHA-256이 같다. 기존 셀 Enter 5건, 원본 HWP/HWPX 및 1쪽 분리본의 호스트 경계 4건이 통과했다. 신규 관계 검사 초안은 머리말만 보정한 라이브러리에서 2쪽 첫 표의 원점 이동으로 FAIL, 본문 보정 라이브러리에서 PASS다. 정식 등록은 fresh WASM·직접 판독 뒤 진행한다.

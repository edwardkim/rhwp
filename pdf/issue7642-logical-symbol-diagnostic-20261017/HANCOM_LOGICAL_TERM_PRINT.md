# 논리 기호·분자 소유의 한컴 Print 증거

Issue: #7642

## 생성·검증 범위

로컬 XML로 작성한 48식 진단 입력을 한컴 2024에서 정상 저장한 HWP와 그 저장본의
독립 Print PDF다. 두 입력 각각 HancomEQN/HYhwpEQ, 6/9/12/18pt, 여섯 식으로
6쪽을 구성한다. 원본 XML도 같은 디렉터리에 보존한다. 일반 원본과 중괄호 대조군을
서로 대신하지 않는다. Hancom 13.0.0.3901, PrintMethod=0,
`frame_print_to_pdf_ex_one_up`을 확인했다.

- 원본 Print job: `159528f2-cca1-47be-9aee-0163e1c48515`
- 중괄호 Print job: `ca3865ba-0fbb-4b46-9e9c-16d4183b931e`
- 검증 code source: `cc4255a58d8112c07013e89eda1b1a065c824fea`
- Native: stage2256 CLI, stage2257 전쪽 TSV, 각 6쪽 최저 raw 100%.
- fresh WASM: stage2258 고정 패키지, stage2262 전쪽 TSV, 각 6쪽 최저 raw 100%.
- 조건: Cairo 96dpi, 인쇄 프로필, compat 2024, 같은 입력/PDF·실루엣 방법.
- 직접 판독: stage2265의 Native 분수 확대, stage2279의 fresh WASM 같은 영역
  review·standalone overlay. raw 100%는 엄격 RGB 동일함을 뜻하지 않는다.
- TSV·실행 로그·JSON은 ignored output에 보존하며 커밋하지 않는다. PR 전 최종
  source에서 필수 gate·대표 안정 PNG를 다시 확인한다.

## 파일 SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `logical-symbol-matrix-saved-2024.hwp` | `1b9d4d07c7a293bb658ec3ead78cdbc4a0c33c48468e1a4bd420cbe8f97ab532` |
| `logical-symbol-matrix-saved-2024.pdf` | `c573d07845ef1018a0896cfcfa1accdab80ff043b378338f5dd4cd317aa37e0c` |
| `logical-symbol-isolated-fractions-saved-2024.hwp` | `da3f28ae0563bda26c48d176593c09d565a17358d8959c2c34e21f88888977c9` |
| `logical-symbol-isolated-fractions-saved-2024.pdf` | `f6637718ac72d7085d0c2d509786c8d35d394529b86581948eccdc2ce47244e1` |

## 독립 기대값과 회귀

`THEREFORE~q over p=39 over 108=13 over 36`의 붙은 normal 공간은 논리 기호를
q와 같은 분자에 소유시킨다. `THEREFORE~{q} over {p}={39} over {108}={13} over {36}`는
명시한 중괄호를 우선해 논리 기호가 분수 바깥에 남는다. 두 식 모두 세 분수와
모든 원본 제어가 한 번씩 보존된다. ∴/∵는 원래 수학 face의 1em이며 이항 연산자의
앞뒤 여백을 만들지 않는다. 관계 검사는 소유·순서·기준선·누락·중복을 보호한다.

새 회귀 파일은 `tests/cases/verified_hancom_logical_term_ownership.rs`다. 모든 쪽이
양쪽 100%이므로, 두 문서의 중요한 중첩 분수가 있는 6쪽을 좌표 보호 대상으로
선정했다. 문서 전체 98%를 가정해 좌표를 추가한 것이 아니라 각각 실제 전쪽 100%를
확인했다. 독립 PDF의 좌상단 CSS 96dpi 글리프 원점은 다음과 같다.

| 입력 | glyph | x | y | 허용 |
| --- | --- | ---: | ---: | --- |
| 원본 p6, HYhwpEQ 18pt 중첩 식 | ∴ | 332.77964 | 262.59302 | ±2px |
| 원본 같은 식 | q(U+E0F5) | 368.76013 | 262.59302 | ±2px |
| 중괄호 p6, HYhwpEQ 18pt 식 | ∴ | 299.67759 | 278.09609 | ±2px |
| 중괄호 같은 식 | q(U+E0F5) | 342.21452 | 262.59302 | ±2px |

관계 검사도 유지한다. glyph는 원본 script·face·크기·쪽 소유로 대응시키며 px 영역으로
대상을 선택하지 않는다. 수용 이후 두 경로의 해당 쪽 raw가 한쪽이라도 98% 아래로
떨어지면 회귀다. 원본 입력 보정 전의 구조 실패는 stage2216/2231에 보존했다.
새 정식 검사 자체의 수정 전 FAIL/수정 후 PASS 실행은 공유 Cargo 작업이 끝난 뒤
별도로 수행해야 하며, 현재 이 문서만으로 완료를 주장하지 않는다.

## 정식 Rust 실행 확인

수정 전 source `769535d7dcb1100701ca68f33091279296aad089`에서 3개 모두 assertion으로
실패했다(exit 100). 수정 후 `d019cd23d3a335b1dc1cc38d348aaefdd1af2ad2`에서
같은 테스트 source SHA의 3개 모두 통과했다(exit 0). stage2282 receipt의
`test_source_unchanged`와 before/after nextest summary를 확인했다.
전체 Rust 검증의 통과라는 뜻은 아니다. 로그는 같은 ignored output의
`stage2282-before-logical-regression.log`와 `stage2282-after-logical-regression.log`다.

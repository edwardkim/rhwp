# #7902 10단계 — 최종 producer 전쪽 무회귀와 실제 음성 대조

Issue: #7902. [9단계](task_m100_7902_stage9.md)는 `3a6c7d0e0`에 커밋했다.
검증 base는 `f5251e7de4e31ee0fb939f352c6ef144bb340bdb`이며 재 fetch에서도 변하지 않았다.

## 실행 중인 최종 검증

- 후보 source: `3a6c7d0e0895fb5258f6cfd28ca4e9cef244e1b8`.
- 최종 Native CLI SHA-256: `30f68b405f71a9b95581cd9c16c0d06dc3685d229551f5082af17bee03a1d089`.
- 최종 fresh WASM SHA-256: `d58034e1a3d0ec2fac30ac0fe8718230775c563e929304f2661e3b7feaa6c254`.
- 원래19문서·835쪽씩 Native/fresh WASM을 모두 재생성한다. Native2문서×2page workers,
  후보/기준 WASM 각각1문서×2page workers로 메모리를 제한한다. Cargo는 동시에 실행하지 않는다.
- Native devel은 같은 exporter·측정 script·alpha helper를 사용한 기존835쪽을 재사용한다.
  실제 baseline source `e4e6d0c8d`와 고정 devel의 `src`·`crates`·Cargo·build.rs 동일성을
  `git diff --quiet`로 확인했다. 비교 script의 옛 worktree 경로와 현재 root 경로는 SHA로 대조한다.
- WASM은 변경한 optional font registration bridge로 **devel835쪽도 다시 생성**한다.
  같은 exporter의 실제 산출물을 비교하며 원본·독립 Print·글꼴·DPI·마스크·반경을 유지한다.

현재 partial 결과는 완료·승인 판정이 아니다. 최종 전쪽90%, 쪽수/누락, 기준 대비0.01%p
하락0쪽과 직접 판독을 확인한 뒤 중요한 완료 페이지의 회귀 보호와 전체 Rust/Native Skia/lint를
마무리한다. PR 직전에는 devel HEAD를 다시 확인하고 실제 변경일 때만 리베이스한다.

## font-only 실제 수정 전 FAIL / 수정 후 PASS

정식 `hft_outline_font_contract`의 원점 계약을 옛 CLI와 최종 CLI에서도 실행했다. 기존 독립
한글 표본의 한 문단 구조를 사용하되 텍스트를 단일 `가`, face를 명시 HFT의 독창적인 사각형
CFF로 교체한 진단 입력2개다. 두 입력의 유일한 위치 변화는 용지 위 여백19HWPUNIT이며
독립적인 기대 이동은 `19 / 75 = 0.253333...px`다. 각 입력 바이트는 before/after 양쪽에 같다.

| 실제 CLI source | 관측 잉크 이동 | 기대 이동 ±0.08px |
| --- | ---: | --- |
| devel와 product 동일한 `e4e6d0c8d` | 1.0px | **FAIL** |
| 최종 후보 `3a6c7d0e0` | 0.2436279137px | **PASS** |

오래된 Font API의 정수 격자 이동과 새 윤곽의 소수 원점 보존을 구분한다. 빌드/환경 실패가
아니다. PNG를 흰 배경에 명시 합성한 뒤 전체 쪽 잉크를 검사했고 임의 영역 마스킹은 없다.
초기 RGB 직접 변환 결과는 보존하되 명시 alpha 합성 결과로 대체했다(최종 관측값은 같다).
이는 font-only 합성 계약이며 독립 한컴 문서 일치·실제 페이지의 98% px 고정 증거로 승격하지 않는다.
진단 HWPX는 ignored 경로에만 보존하고 활성 corpus나 golden으로 추가하지 않았다.

## 실행 증거

ignored 루트 `output/pr-review/issue7902-render-repair-20261014/`:
`stage9-final-producer-build.json`, `stage9-final-full-{native,wasm}-result.json`,
`stage9-base-current-exporter-full-wasm-result.json`, `stage9-font-before-after/result.json`에
명령·실제 바이너리/입력/font/PNG SHA와 결과를 연결한다. 해당 원시 자료는 Git에 추가하지 않는다.
사용자의 dirty `rhwp-studio/public/rhwp.js`는 보존했고 커밋에 포함하지 않았다.

## Native 전쪽 완료

최종 CLI의19문서·835쪽이 전부 완료됐다. 최저91.39728%,90% 미만·누락·쪽수 불일치0건,
devel 대비 최대 하락0.00661%p이며 **0.01%p 이상 하락0쪽**이다.
`stage9-native-decimal-audit.json`에835쪽 전부의 원값과 Decimal 차이를 보존했다.

WASM 후보/기준 전쪽 비교와 최종 release-test 전체 nextest는 실행 중이다. Rust는 jobs8,
공용 `target/pr-review`, 실행12threads, `--no-fail-fast`로 수행한다. PNG 검증은 고정 복제
바이너리/패키지를 사용하므로 실행 중인 Cargo가 producer를 바꾸지 않는다. 전체 결과는 아직
미검증이며 Native 수치만으로 최종 승인·PR 준비 완료로 변경하지 않는다.

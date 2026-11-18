---
kind: reference
status: active
canonical: mydocs/manual/pr_review/collaborator_self_merge.md
last_verified: 2026-11-18
---

# PR #9173 리뷰 — v0.9.1 배포 준비

## 최종 판정

**승인.** 보정 head `788bcc7282b27aec4debef481e5283fe1ac723a6`의 필수 로컬·GitHub 검증이 통과했다. 이 검토 기록이 포함된 최신 head의 필수 검사와 병합 가능 상태를 확인한 뒤 devel에 통합한다.

## 변경과 검증 범위

- [PR #9173](https://github.com/edwardkim/rhwp/pull/9173), 작성자 `edwardkim`, collaborator self-review. 기능 마감 base는 `197ce19e99438c2cba452c654ae1af504dab85ec`이다.
- 0.9.1 버전·패키지·배포 문서, 기여자 19명과 Dependabot, deprecated Atomic 보정, Chrome 설정 채널 종료와 Studio 초기 열기 경합 보정을 검토했다. 조판 규칙과 renderer 출력을 바꾸는 신규 구현은 없다. 기능 마감 지점에 포함된 #9157의 HWP3·Native 글꼴 보존은 보호 문서 전쪽 결과와 대표 구간 직접 판독으로 확인했다.
- 최초 `bb43fb78d1`의 [lint](https://github.com/edwardkim/rhwp/actions/runs/44259474754/job/134649184729)는 라이선스 고지와 VS Code 변경 이력의 버전 표기 계약을 실패했다. 로컬 검사 목록에서 이 계약 모듈을 빠뜨렸다. 표기와 내부 코덱 버전을 실제 manifest에 맞춘 뒤 동일 workflow 계약 643건과 문서 링크·고지 파일 생성을 통과했다. npm 코어와 VSIX 재포장에서 변경된 파일은 고지와 변경 이력뿐이다.
- Rust 전체 회귀 13,724건·Native 공식 4,073건은 `c0f9ea46f4`에서 실제 실행했다. Studio 2,811건 및 브라우저·최적화 WASM·AMO 재빌드는 `5167f63a26`에서 실행했다. 이후 보정은 문서로 한정했다. Cargo 14개 포장·별도 소비 및 실제 소스와 체크섬은 [배포 준비 보고서](../../report/release_0_9_1_preparation_report.md)와 [원장](../../tech/investigations/release-v0.9.1/README.md)에 있다.
- 보호 입력과 독립 PDF 18쌍을 저장소 및 manifest의 SHA와 대조했다. Native/fresh WASM 각 1,771쪽의 보호값 하락·누락은 0건이다. 최저 raw는 Native 98.86043%, WASM 98.86472%다. 편람 HWPX Native 383쪽 gate와 중첩 표·편람의 확대 판독도 완료했다. [전쪽 TSV·입력 조건](../../tech/investigations/release-v0.9.1/protected-validation.json)

## GitHub 검증

| 검사 | `788bcc7282` 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/44261053906) | Build & Test, lint, 프런트엔드 패키징 성공. Archive A–E 13,530개 통과·기존 skip 45개, 선택 Native 247개 통과 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44261053937) | Actions·JavaScript/TypeScript·Python·Rust 성공 |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44261053772) | Canvas/PDF 비교 성공 |
| [Adapter](https://github.com/edwardkim/rhwp/actions/runs/44261053804) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/44261053885) | 성공 |
| [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/44265264085) | Full CI 증거 확인 및 `CI Impact Policy: success` |

devel 병합 뒤 승격 검증 12개를 같은 devel SHA에서 실행한다. #9154 종료 판단은 main 반영과 정기 실행 확인 뒤에 한다. 첫 게시 전에는 최종 승인된 소스에서 패키지를 다시 만들고 공개 소비를 확인한다.

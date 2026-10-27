---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase83-sweep-fonts-resume/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase83-sweep-fonts-resume/goal-checkpoint.md` · 원문 SHA-256: `3947e3c54776bba8a2cbef5a42a85417b8c9dacb03d1c4b84383e12d9e575d34`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 83차 체크포인트

전체 목표는 ACTIVE/미완료다. 전수 조사 및 독립 판정을 완료한 뒤 이슈를 최종 갱신하고, 현행 지침에 어긋난 기존 검사를 실제 출력 보정/필수 시각 선행 조건과 함께 변경한다. 실제 보정한 파일·함수만 issue_8232 이름으로 바꾸며 helper/module/suite/manifest/생성/스크립트/E2E 참조도 갱신한다. 정상 검출 목적은 보존한다. 아직 제품·검사 수정 0, 개명 0이다. PR은 나중이며 원격 소스 push/PR 승인은 없다. 다른 task/subagent/예약/메모리 쓰기는 하지 않는다.

현재 R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD=01ced3d42724285c8f9765a453393ab6af326617다. 기존 dirty rhwp-studio/public/rhwp.js는 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6로 보존했다. 감사에서 만든 변경이 아니며 되돌리거나 커밋하지 않는다. 사용자 지적에 따라 정적 감사를 우선하고 실제 일치하는 기존 바이너리를 재사용한다. 이번 81~83차 Cargo/prepare/추가 빌드는 모두 0회다. 요청 없이 fetch/rebase하지 않는다. 원본/PDF/공유 target/다른 작업장/기존 Studio 서버를 보존한다.

정본은 active-audit.json이 가리키는 phase83-sweep-fonts-resume/audit-ledger.json과 scope-summary.json이다. 현재 21350그룹/2501파일, 직접 9572함수/Rust 6382검사/835파일이다. 미분류 11509 + 의미 메모 95 + 파일 문맥 165 = 11769그룹은 추가 판정이 필요하다. 직접 판정 안의 근거 pending 511개도 해결 전이다. 전체 그룹 수나 AST root 수를 실제 실행 testcase 수로 부르지 않는다.

80차에 초기 수치 검색 밖의 기존 AST 후보 Rust 2887·스크립트 5630 = 8517개를 미분류로 연결한 범위 정정은 유지한다. workspace의 명시적 vendor exclude 6496개 후보는 별도 보존했다. 사용자 제외 삭제 Rust 3검사(원장 4그룹), 신규 Rust 95검사/48파일, 새 JS callback 1(schema 2건)과 관련 helper도 감사/PASS에 넣지 않았다. is_test=false callback·매크로/동적 호출/생성 경로 및 feature별 실제 등록 대조까지 완료하기 전 전수 완료로 판정하지 않는다. 상세 범위 근거는 phase80-root-inventory-backstop/goal-checkpoint.md를 따른다.

81차는 schema/ontology/registry/WASM API의 선택 32함수 584줄을 읽어 29함수/24 Rust roots를 신규 판정했다. 기존 직접 판정 3함수는 그대로 보존했다. private helper 5개를 원장에 연결했다. 현재 등록 metadata에서 대상 실행 파일을 찾지 못해 Rust 실행 0이며, 이를 위해 빌드를 새로 하지 않았다. 실제 CLI export-ir-schema JSON의 41정의를 독립 루트 순회해 전부 도달함을 확인했다. 최초 export shell은 마지막 Python exit만 기록해 CLI 자체 exit는 unknown으로 명시했으며 정식 Rust PASS로 세지 않았다. 기존 도달성 검사는 전체 $defs의 참조를 모으므로 연결되지 않은 A↔B 순환을 놓치는 검출 한계를 Python shadow 진단으로 재현했다. 현재 제품 스키마 결함/정식 Rust FAIL/px 지침 위반으로 단정하지 않았다. 단위 온톨로지 표본과 실제 CLI 전체 계약, CanvasKit 계획 문자열과 실제 backend 실행, fallback 경로 문자열과 실제 글꼴 선택을 구분했다. 원 PR 3775/4239/761/925 및 통합 3801/4253/4496·폰트 4800·profile 2297의 목적/비목적을 확인하고 source blame/조상 관계를 남겼다. phase81-schema-api-contracts의 producer/independent-schema-reachability/origin-contract-map/validation이 근거다.

82차는 test_visual_sweep.py 앞 468줄의 39함수/32 Python roots/7 helpers를 판정했다. 기존 32검사 PASS, 실패 0, skip 0, 4.549초이며 빌드 0이다. 직접 작성 실루엣 이미지·검토 라벨/canvas·경로/쪽 선택·PDF fill 계약을 실물 문서의 잠정 페이지 배치 핀과 구분했다. 좌표 집합의 독립 순회로 raw 산술 3조건을 대조했으며 색상 보조값 100/raw 0인 실제 gate가 보류함을 확인했다. 작성 PDF의 사용하지 않은 stroke width가 nonzero/evenodd fill을 바꾸지 않는 기존 root는 실제 현재 PDFium runner를 호출해 통과했다. 다만 class의 pdftoppm 존재 조건은 현재 backend와 다른 낡은 실행 전제이며, 이번 환경에서는 skip하지 않았다. 라벨 검사는 크기/잉크 경계의 범위만 입증하고 문서 영역 byte 불변이나 모든 내용 중복까지 증명하지 않는 한계도 기록했다.

83차는 같은 파일 471..823줄의 24함수/20 Python roots/4 helpers를 판정했다. 기존 20검사 PASS, 실패 0, skip 0, 2.141초이며 빌드 0이다. 실제 TTF fixture는 읽기만 하고 진단 복제본에서 cmap을 제거했다. 현재 font cmap validator·CSS·export 소유·자료 지문/manifest 생산 경로를 대조했다. 작성 x12/y34는 CSS 삽입이 원 SVG를 보존하는 계약이며 실물 페이지 핀이 아니다. WASM/Native exporter는 mock이고 실제 WASM 빌드/Node/문서 렌더링은 하지 않았다. cmap 존재를 모든 glyph coverage/보이는 잉크로 확대하지 않았다. run provenance root는 실제 threshold 32→33 변이만 검사하고, checkpoint root는 dummy artifact 존재를 검증할 뿐 내용/해시/시각 유효성 전체를 인증하지 않음을 기록했다.

이번 목표 turn의 신규 판정은 총 92함수(신규 Rust roots 24·Python roots 52·helpers 16), 기존 3함수 재독이다. 실제 실행은 Python 52 PASS/0FAIL/0skip, 합계 6.690초다. Rust 신규 실행/빌드/캡처/제품·검사 수정/개명은 모두 0이다. 이전 turn의 상태 재확인만으로는 목표 진행을 주장하지 않으며, 이번에는 81~83차 정본 대장과 직접 진단/실행 증거를 실제 갱신했다.

#8232의 마지막 원격 본문은 80차 갱신(updated_at 2026-10-26T14:23:00Z, UTF16 65534)이다. 81~83차는 로컬 정본에 기록했고 아직 원격에 게시하지 않았다. 전수 완료 뒤 최종 이슈 갱신 지시를 유지한다. 다음 갱신도 fresh GET·UTF16 한도·다른 작업/실제 실패 근거 보존·exact readback을 따른다. 정상 중복 요약만 줄이고 실제 발견사항을 삭제하지 않는다.

이전 실제 잘못된 핀/검출 한계와 source·독립 PDF·Native/fresh WASM 자격 pending은 80차와 그 이전 기록대로 보존했다. 현재 원장의 change_required_for_issue8232_px_policy=true 87개는 보정 판정이지 구현 완료 수가 아니다. 같은 입력의 필수 시각 자격과 현재 base를 확인한 뒤 실제 보정한다. 정적 API 단위 PASS로 시각 미달/px pin을 수용하지 않는다.

다음은 test_visual_sweep.py의 826줄 이후 실제 합성 layout detector/영역 선택·clip·tail 검사를 읽고 각 독립 입력/허용 관계와 현재 지침을 대조한다. 이후 test_fidelity_compare.py의 detector·source ownership 계약과 source cfg(test) 미분류를 이어간다. 이미 실행한 52개나 같은 빌드를 반복하지 않는다. 511 pending은 독립 원본/Print와 현재 양 backend 자격 및 정상 반례를 확보해 해결해야 한다.

pub(super) fn print() {
    println!("  export-ir-schema [--bare] [-o <파일>] [--json]");
    println!("      공개 IR 의 JSON Schema — 외부 바인딩 코드 생성의 단일 출처");
    println!("      --bare 는 봉투 없이 스키마 본문만 (JSON Schema 도구 입력용)");
    println!("  run <계획.json> [--json]              선언적 편집 계획 실행 (#3703)");
    println!("  replay <계획.json> [--expect-output-sha256 <hex>] [--sign-key <키.json>] [--json]  작업 영수증 발급·재현 검증 (#4391)");
    println!("  audit <캡슐 폴더> [--json]            작업 캡슐 전수 재검증 — 재현율 회계 (#4393)");
    println!("  lineage <캡슐.json> [--deep] [--keyring <키링.json>] [--anchor-log <로그>] [--json]  작업 계보(해시 체인) 연대기 검증 (#4401)");
    println!("  keygen --key-id <id> --out <키.json>   Ed25519 서명키 발급 (#4509)");
    println!("  verify-signature <캡슐> --keyring <키링.json> [--sig <서명.json>] [--json]  캡슐 서명 검증 (#4509)");
    println!("  harness init <폴더> [--key-id <id>]     검증 작업장 생성 (#4537)");
    println!("  harness wrap --plan <JSON|@파일> --dir <작업장> [--sign-key <키>]  실행+영수증+캡슐+체인+서명 한 방 (#4537)");
    println!("  harness-status <작업장> [--keyring <키링>] [--deep] [--json]  체인·서명·재현 통합 판정 (읽기 전용) (#4537)");
    println!("  anchor add <캡슐> --log <anchor.ndjson>   투명성 로그 등재 (#4543)");
    println!("  anchor checkpoint --log <로그> [-o <파일>]  머클 체크포인트 산출 (#4543)");
    println!("  anchor verify <캡슐> --log <로그> [--checkpoint <파일>] [--json]  등재·무결·머클 경로 판정 (#4543)");
    println!("  gate <캡슐> --policy <policy.json> [--keyring][--anchor-log][--deep]  반입 정책 기계 판정 (#4545)");
    println!("  bundle export <머리캡슐> -o <x.lineage-bundle> [--anchor-log --checkpoint][--domain]  연합 번들 내보내기 (#4549)");
    println!(
        "  bundle verify <번들> --trust-domain <domain.json> [--json]  5단 오프라인 검증 (#4549)"
    );
    println!(
        "  disclose redact <캡슐> -o <가림> --opening-out <개봉>  salt 커밋 가림 발급 (#4551)"
    );
    println!(
        "  disclose verify <가림> --opening <부분개봉> [--json]   필드 단위 커밋 대조 (#4551)"
    );
    println!("  disclose restore <가림> --opening <전체개봉> -o <복원>  바이트 완전 복원 (#4551)");
    println!("  settle propose --workorder <wo> --capsule <c> --gate-envelope <g> -o <청구>  3해시 고정 청구 발급 (#4553)");
    println!("  settle verify <청구> --workorder <wo> --capsule <c> --gate-envelope <g> [--keyring] [--ledger]  청구 검증 (#4553)");
    println!("  settle record <청구> --ledger <원장>  이중 청구 검사 후 원장 기입 (#4553)");
    println!("  audit-report <캡슐 폴더> -o <보고서> [--deep] [--keyring] [--anchor-log] [--policy] [--sign-key]  감사 보고 표준 (#4558)");
    println!("  recall-scope --contaminated <캡슐|sha256> --among <폴더> [--ledger]  오염 후손 폐쇄집합 (#4558)");
    println!("  conformance <캡슐 폴더> --level <L1..L5> [--deep] [--keyring] [--anchor-log] [--policy] [--ledger]  적합성 자가진단 (#4558)");
    println!("      전 step 을 정적 선검증(불가 시 실행 0·exit 2)하고 인메모리로 원자");
    println!("      실행해 단언(verify) 통과 시에만 단 한 번 저장한다 — 실패 시 디스크 무변경.");
    println!("      steps: fill_fields{{data}} · replace_text{{find,replace[,occurrence]}}");
    println!("             · set_cell{{table,row,col,text}} · set_checkbox{{occurrence}}");
    println!("      --plan-json '<JSON>'      파일 대신 인라인 계획 (MCP hwp_run_plan 경로)");
    println!("      --dry-run                 선검증만 — preview 저널, 디스크 무변경 (계획서 dryRun:true 와 동일)");
    println!("      step 마다 if 조건 가능: {{fieldExists}}·{{fieldEquals:{{name,value}}}}·{{textFound}}");
    println!("      조건이 거짓이면 그 step 만 건너뛰고 저널에 skipped:true·reason 으로 남긴다");
    println!("      (거짓인 step 은 선검증도 면제 — 없는 필드를 채우는 step 도 위반이 아니다)");
    println!("      단언 실패는 exit 3 — 저널(steps[]·verify)로 판정을 데이터로 보고");
    println!();
    println!("  export-plan-schema [--bare] [-o <파일>] [--json]");
    println!("      run 계획서 문법의 JSON Schema 출력 — 계획을 쓰기 전에 읽는 정답지");
    println!();
    println!("      --bare                  봉투 없이 계획 스키마 본문만 출력");
    println!("      -o, --out <파일>        스키마를 파일로 저장 (생략 시 stdout)");
    println!("      --json                  -o 와 함께 쓰면 저장 결과를 JSON 봉투로 보고");
    println!();
    println!("내부 개발·회귀 도구 (일반 사용자 대상 아님):");
    println!("  test-caption <파일.hwp> [-o <폴더>] 캡션 라운드트립 검증");
    println!("  test-field <파일.hwp>               필드 라운드트립 검증");
    println!("  test-shape <입력.hwp> <출력.hwp>    도형 라운드트립 검증");
    println!("  gen-table                           표 테스트 HWP 생성");
    println!("  gen-pua                             PUA 문자 테스트 HWP 생성");
    println!();
    println!("옵션:");
    println!("  -h, --help      도움말 표시");
    println!("  -V, --version   버전 표시");
}

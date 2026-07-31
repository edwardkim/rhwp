//! [#3629] 에이전트 역할 프로필 — 직무별 도구 세트·워크플로 레시피의 **단일 출처**.
//!
//! 들어오는 에이전트는 직무가 다르다: 경영 보고 에이전트는 요약·조회만, 행정 서식
//! 에이전트는 채움·검증만 쓴다. 전 도구를 균일 노출하면 경량 에이전트의 도구 선택
//! 오류와 컨텍스트 낭비가 커진다. 본 표 하나가 `capabilities --mcp --profile` 와
//! `mcp-serve --profile` 양쪽을 구동한다 — 목록을 다른 곳에 복제하지 않는다
//! (플레이북 규칙 1).

/// 역할 프로필 하나. `tools` 는 무상태 MCP 도구 이름, `session` 은 세션 도구 포함 여부.
pub struct AgentProfile {
    pub name: &'static str,
    pub summary: &'static str,
    /// 이 직무가 쓰는 무상태 도구 이름들 (mcp_tool_definitions 의 name).
    pub tools: &'static [&'static str],
    /// 세션 도구(hwp_open/hwp_doc_*/hwp_close) 노출 여부 — 반복 조회·편집 직무만 true.
    pub session: bool,
    /// 권장 호출 순서 레시피 — 경량 에이전트가 순서 실수를 하지 않도록 계약으로 제공.
    pub recipe: &'static [&'static str],
}

pub const PROFILES: &[AgentProfile] = &[
    AgentProfile {
        name: "경영보고",
        summary: "임원·보고용 — 문서 파악과 요약 근거 수집, 제출용 산출물 확인",
        tools: &[
            "hwp_info",
            "hwp_export_text",
            "hwp_export_structure",
            "hwp_search",
            "hwp_thumbnail",
            "hwp_export_pdf",
        ],
        session: false,
        recipe: &[
            "hwp_info 로 규모·형식 파악",
            "hwp_export_structure 로 목차 확보 후 필요한 절만 hwp_export_text",
            "근거 위치는 hwp_search 로 쪽 번호까지",
            "제출용은 hwp_export_pdf, 훑어보기는 hwp_thumbnail",
        ],
    },
    AgentProfile {
        name: "행정서식",
        summary: "서식 자동 작성 — 누름틀·표·체크박스 채움과 제출 전 검증",
        tools: &[
            "hwp_fields",
            "hwp_fill_fields",
            "hwp_export_tables",
            "hwp_set_cell",
            "hwp_set_checkbox",
            "hwp_search",
            "hwp_export_svg",
            "hwp_ir_diff",
        ],
        session: true,
        recipe: &[
            "hwp_fields 로 무엇을 요구하는 서식인지 조사 (반복 이름은 '이름[N]')",
            "hwp_fill_fields → notFound/ambiguous 가 비어야 완료",
            "누름틀 없는 칸은 hwp_export_tables 좌표로 hwp_set_cell (overflow 확인)",
            "체크박스는 hwp_search 로 '□' 순번 확인 후 hwp_set_checkbox",
            "hwp_export_svg 로 바뀐 쪽 눈검증, hwp_ir_diff 로 의도한 변경만인지 확인",
        ],
    },
    AgentProfile {
        name: "데이터분석",
        summary: "표 데이터 수확 — HWP 표를 구조화 데이터로, 아카이브 일괄 추출",
        tools: &[
            "hwp_info",
            "hwp_export_tables",
            "hwp_search",
            "hwp_batch",
            "hwp_batch_search",
        ],
        session: false,
        recipe: &[
            "단건은 hwp_export_tables (병합은 rowSpan/colSpan 보존)",
            "대량은 paths 배열로 hwp_batch subcommand=export-tables",
            "값 위치 추적은 hwp_search 의 셀 주소",
        ],
    },
    AgentProfile {
        name: "콘텐츠제작",
        summary: "문서 생성·발행 — 명세로 새 문서를 만들고 배포 형식으로 내보냄",
        tools: &[
            "hwp_build_from_ingest",
            "hwp_export_svg",
            "hwp_export_pdf",
            "hwp_export_markdown",
            "hwp_thumbnail",
            "hwp_convert_hwpx",
        ],
        session: false,
        recipe: &[
            "hwp_build_from_ingest 로 ingest JSON → HWPX 생성",
            "hwp_export_svg 로 조판 확인 후 hwp_export_pdf 로 발행",
            "웹·LLM 소비용은 hwp_export_markdown",
        ],
    },
    AgentProfile {
        name: "아카이브검색",
        summary: "대량 문서 RAG·감사 — 수백 건 스윕과 근거 쪽 번호 인용",
        tools: &[
            "hwp_batch",
            "hwp_batch_search",
            "hwp_search",
            "hwp_export_text",
            "hwp_export_structure",
            "hwp_thumbnail",
            "hwp_split_document",
        ],
        session: true,
        recipe: &[
            "hwp_batch subcommand=info 로 아카이브 대장화",
            "hwp_batch_search 로 전 문서 검색 (어느 문서 몇 쪽)",
            "대형 문서 반복 조회는 hwp_open → hwp_doc_search/hwp_doc_text",
            "발췌 제출은 hwp_split_document",
        ],
    },
    AgentProfile {
        name: "품질검증",
        summary: "변환·편집 무손실 게이트 — 판정은 데이터(identical/diffCount)",
        tools: &[
            "hwp_ir_diff",
            "hwp_convert_hwpx",
            "hwp_convert_hwp5",
            "hwp_export_hml",
            "hwp_export_svg",
            "hwp_info",
        ],
        session: false,
        recipe: &[
            "변환은 hwp_convert_* 의 verify 봉투로 1차 판정",
            "차이가 있으면 hwp_ir_diff 로 categories 분류",
            "시각 대조가 필요하면 양쪽을 hwp_export_svg 로 렌더",
        ],
    },
    AgentProfile {
        name: "개발통합",
        summary: "전체 표면 — 필터 없음 (rhwp 를 통합하는 개발 에이전트)",
        tools: &[],
        session: true,
        recipe: &[
            "capabilities 로 전 명령 계약을 파악하고 시작",
            "mydocs/manual/agent_knowledge_map.md 가 진입점",
        ],
    },
];

/// 이름으로 프로필을 찾는다. `개발통합`(tools 빈 배열)은 "필터 없음"을 뜻한다.
pub fn find(name: &str) -> Option<&'static AgentProfile> {
    PROFILES.iter().find(|p| p.name == name)
}

/// 프로필 이름 목록 (오류 안내·자기서술용).
pub fn names() -> Vec<&'static str> {
    PROFILES.iter().map(|p| p.name).collect()
}

/// 무상태 도구가 이 프로필에 포함되는가. tools 가 비어 있으면 전체 허용.
pub fn allows_tool(profile: &AgentProfile, tool_name: &str) -> bool {
    profile.tools.is_empty() || profile.tools.contains(&tool_name)
}

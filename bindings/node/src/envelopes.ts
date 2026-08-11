/**
 * 명령별 봉투 타입 — **자동 생성 파일. 손으로 고치지 마세요.**
 *
 * 재생성: `npm run gen:types` (tools/gen-types.ts)
 * 출처:   `rhwp capabilities` — version 0.8.2, `--json` 봉투 46개
 *
 * `capabilities` 는 명령마다 **어떤 필드가 있는지**(`recordFields`)만 선언하고 타입은
 * 말하지 않습니다. 그래서 대부분의 필드가 `unknown` 입니다 — 짐작한 타입을 적으면 그
 * 짐작이 컴파일러의 보증으로 둔갑하고, 사용자는 검사받았다고 믿은 채 틀린 코드를 씁니다.
 * 이름만으로 확실한 소수(`schemaVersion`·`pageCount`·`verify` …)에만 타입을 줍니다.
 *
 * 모든 필드가 선택(`?`)인 이유: 옵션에 따라 나오지 않는 필드가 있는데 `capabilities`
 * 는 그 조건을 서술하지 않습니다. 없을 수 있다는 사실을 타입에 남깁니다.
 *
 * 인덱스 시그니처는 봉투의 **추가-전용** 계약이자, 각 인터페이스가
 * `Envelope<T extends RawEnvelope>` 의 제약을 만족하게 하는 장치입니다.
 *
 * @packageDocumentation
 */

import type { RawVerifyReport } from './envelope.js';

/** 이 파일을 만들어 낸 capabilities 스냅샷 버전(= rhwp 버전). */
export const CAPABILITIES_SNAPSHOT_VERSION = '0.8.2';

/**
 * `rhwp anchor --json` 봉투.
 *
 * 투명성 로그(T7 방어) — add(append-only 등재, 깨진 로그 거부)·checkpoint(머클
 * 루트)·verify(등재·자기 무결·머클 경로 판정, 아님 exit 3). 공표는 운영 절차 (#4543)
 */
export interface AnchorEnvelope {
  readonly capsuleSha256?: unknown;
  readonly entries?: unknown;
  readonly inCheckpoint?: unknown;
  readonly log?: unknown;
  readonly logChainOk?: unknown;
  readonly logged?: unknown;
  readonly merklePath?: unknown;
  readonly merkleRoot?: unknown;
  readonly schemaVersion?: string;
  readonly seq?: unknown;
  readonly upToSeq?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp audit --json` 봉투.
 *
 * 작업 캡슐(*.capsule.json) 폴더 전수 재실행·대조 — 에이전트 노동의 재현율 회계. 불일치
 * 1건이라도 있으면 exit 3 (#4393)
 */
export interface AuditEnvelope {
  readonly failed?: unknown;
  readonly reproduced?: unknown;
  readonly reproducedRate?: unknown;
  readonly root?: unknown;
  readonly schemaVersion?: string;
  readonly total?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp batch --json` 봉투.
 *
 * stdin 파일 목록을 한 프로세스에서 파일 간 병렬 처리, NDJSON 스트림 출력 (fill 축만 stdin 대신
 * --form 서식 + --data 행 파일로 메일머지)
 */
export interface BatchEnvelope {
  readonly error?: string;
  readonly exitClass?: unknown;
  readonly filledCount?: unknown;
  readonly notFound?: unknown;
  readonly output?: string;
  readonly row?: unknown;
  readonly schemaVersion?: string;
  readonly source?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp build-from-ingest --json` 봉투.
 *
 * ingest JSON에서 HWPX 생성 (--json 봉투)
 */
export interface BuildFromIngestEnvelope {
  readonly bytes?: number;
  readonly format?: string;
  readonly output?: string;
  readonly paragraphCount?: number;
  readonly questionCount?: number;
  readonly schemaVersion?: string;
  readonly source?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp bundle --json` 봉투.
 *
 * 연합 교환 — export(계보 폐쇄집합+서명+머클 증명을 zip
 * 하나로)·verify(컨테이너·폐쇄집합·계보·서명[도메인 키링만, 동봉 불신]·앵커 5단 오프라인 판정,
 * 깨짐 exit 3) (#4549)
 */
export interface BundleEnvelope {
  readonly anchored?: unknown;
  readonly brokenAt?: unknown;
  readonly bundle?: unknown;
  readonly capsules?: unknown;
  readonly closureOk?: unknown;
  readonly containerOk?: unknown;
  readonly head?: unknown;
  readonly lineageValid?: unknown;
  readonly proofs?: unknown;
  readonly schemaVersion?: string;
  readonly signatures?: unknown;
  readonly signed?: unknown;
  readonly trustDomain?: unknown;
  readonly verdict?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp capabilities` 봉투.
 *
 * 본 자기서술 JSON 출력
 */
export interface CapabilitiesEnvelope {
  readonly batch?: unknown;
  readonly commands?: unknown;
  readonly exitCodes?: unknown;
  readonly schemaRegistry?: unknown;
  readonly schemaVersion?: string;
  readonly tool?: string;
  readonly version?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp convert --json` 봉투.
 *
 * HWPX/배포용→편집 가능 HWP5 변환 (--verify 게이트 exit 3/4, --json 봉투)
 */
export interface ConvertEnvelope {
  readonly bytes?: number;
  readonly format?: string;
  readonly output?: string;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly verify?: RawVerifyReport | null;
  readonly verifyPages?: unknown;
  readonly wasDistribution?: boolean;

  readonly [key: string]: unknown;
}

/**
 * `rhwp csv-to-table --json` 봉투.
 *
 * CSV 로 기존 표 N 의 셀 덮어쓰기 — 표 크기 불변, 행·열 불일치는 invalid+exit 2
 */
export interface CsvToTableEnvelope {
  readonly changed?: unknown;
  readonly changedCount?: unknown;
  readonly changedPages?: readonly number[] | null;
  readonly colCount?: unknown;
  readonly csv?: unknown;
  readonly dryRun?: unknown;
  readonly invalid?: unknown;
  readonly output?: string;
  readonly outputFormat?: string;
  readonly rowCount?: unknown;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly table?: unknown;
  readonly verify?: RawVerifyReport | null;

  readonly [key: string]: unknown;
}

/**
 * `rhwp digest --json` 봉투.
 *
 * 문서 요약 봉투(메타·개요·발췌·nextStep)를 한 번 호출로 출력
 */
export interface DigestEnvelope {
  readonly excerpt?: string;
  readonly format?: string;
  readonly nextStep?: string;
  readonly outline?: unknown;
  readonly pageCount?: number;
  readonly paraCount?: number;
  readonly schemaVersion?: string;
  readonly sections?: unknown;
  readonly source?: string;
  readonly truncated?: boolean;

  readonly [key: string]: unknown;
}

/**
 * `rhwp dump-pages --json` 봉투.
 *
 * 페이지네이션 항목 덤프 (--json: 조판 진단 기계 계약)
 */
export interface DumpPagesEnvelope {
  readonly pageCount?: number;
  readonly pageFilter?: unknown;
  readonly pages?: unknown;
  readonly respectVposReset?: unknown;
  readonly schemaVersion?: string;
  readonly source?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp edit --json` 봉투.
 *
 * 문서 편집 — fill-fields: 누름틀 채우기 / replace-text: 일괄 치환(--occurrence k번째만) /
 * set-cell: 표 셀 기록 / insert-image: 도장·서명 그림 삽입 / redact: 개인정보 마스킹 /
 * sanitize: 메타데이터 제거
 */
export interface EditEnvelope {
  readonly binDataId?: unknown;
  readonly changedPages?: readonly number[] | null;
  readonly col?: unknown;
  readonly dryRun?: boolean;
  readonly filled?: unknown;
  readonly filledCount?: number;
  readonly findingCount?: unknown;
  readonly findings?: unknown;
  readonly height?: unknown;
  readonly image?: unknown;
  readonly inPlace?: unknown;
  readonly keepPreview?: unknown;
  readonly keepStyle?: unknown;
  readonly kinds?: unknown;
  readonly mask?: unknown;
  readonly newText?: unknown;
  readonly notFound?: unknown;
  readonly oldText?: unknown;
  readonly output?: string;
  readonly outputFormat?: string;
  readonly overflow?: unknown;
  readonly page?: unknown;
  readonly redactedCount?: unknown;
  readonly removed?: unknown;
  readonly removedCount?: unknown;
  readonly replacedCount?: number;
  readonly row?: unknown;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly table?: unknown;
  readonly verify?: RawVerifyReport | null;
  readonly width?: unknown;
  readonly x?: unknown;
  readonly y?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp explain --json` 봉투.
 *
 * 문서를 결정론적 규칙 문장으로 요약(형식·쪽수·문단·표·누름틀·각주/미주·암호 여부)
 */
export interface ExplainEnvelope {
  readonly encrypted?: unknown;
  readonly endnoteCount?: unknown;
  readonly fields?: unknown;
  readonly footnoteCount?: unknown;
  readonly format?: string;
  readonly pageCount?: number;
  readonly paragraphCount?: unknown;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly summary?: unknown;
  readonly tables?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-agent-manifest --json` 봉투.
 *
 * capabilities+irSchema+provenanceMap+planSchema 를 한 번의 호출로 조립 — 누락 축이 생기면
 * missingAxes 로 명시 (#3828 B2)
 */
export interface ExportAgentManifestEnvelope {
  readonly capabilities?: unknown;
  readonly irSchema?: unknown;
  readonly missingAxes?: unknown;
  readonly planSchema?: unknown;
  readonly provenanceMap?: unknown;
  readonly schemaVersion?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-capabilities-schema --json` 봉투.
 *
 * capabilities 자기서술 자체의 JSON Schema 산출 — 명령 표면 코드 생성의 단일 출처 (#3776)
 */
export interface ExportCapabilitiesSchemaEnvelope {
  readonly capabilitiesSchemaVersion?: string;
  readonly definitionCount?: number;
  readonly dialect?: string;
  readonly mcpSchema?: unknown;
  readonly schema?: unknown;
  readonly schemaVersion?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-doclang --json` 봉투.
 *
 * 문서를 DocLang v0.6 XML로 내보내기 (--json 봉투)
 */
export interface ExportDoclangEnvelope {
  readonly assetCount?: number;
  readonly assetsDir?: string;
  readonly bytes?: number;
  readonly doclangVersion?: string;
  readonly format?: string;
  readonly lossCount?: number;
  readonly output?: string;
  readonly schemaVersion?: string;
  readonly source?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-hml --json` 봉투.
 *
 * HML 원본을 HWPML 2.91 XML로 저장 (--json 봉투)
 */
export interface ExportHmlEnvelope {
  readonly bytes?: number;
  readonly format?: string;
  readonly output?: string;
  readonly schemaVersion?: string;
  readonly source?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-hwpx --json` 봉투.
 *
 * HWP→HWPX 변환 저장 (--verify 게이트 exit 3/4, --json 봉투)
 */
export interface ExportHwpxEnvelope {
  readonly bytes?: number;
  readonly format?: string;
  readonly output?: string;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly verify?: RawVerifyReport | null;
  readonly verifyPages?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-ir-schema --json` 봉투.
 *
 * 공개 IR 의 JSON Schema 산출 — 외부 바인딩 코드 생성의 단일 출처 (#3762)
 */
export interface ExportIrSchemaEnvelope {
  readonly definitionCount?: unknown;
  readonly dialect?: unknown;
  readonly irSchemaVersion?: unknown;
  readonly schema?: unknown;
  readonly schemaVersion?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-markdown --json` 봉투.
 *
 * 페이지별 텍스트를 Markdown으로 추출 (--json 매니페스트)
 */
export interface ExportMarkdownEnvelope {
  readonly format?: string;
  readonly imageCount?: number;
  readonly outputDir?: string;
  readonly pageCount?: number;
  readonly pages?: unknown;
  readonly renderedCount?: number;
  readonly schemaVersion?: string;
  readonly source?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-ontology --json` 봉투.
 *
 * 자기서술에서 기계 유도한 JSON-LD 온톨로지 산출 — IR 클래스·속성, 명령/MCP 행위, 신뢰 술어
 * (#3907 O1)
 */
export interface ExportOntologyEnvelope {
  readonly actionCount?: unknown;
  readonly classCount?: unknown;
  readonly ontology?: unknown;
  readonly propertyCount?: unknown;
  readonly schemaVersion?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-pdf --json` 봉투.
 *
 * 문서를 PDF로 렌더 (svg|direct backend, --json 매니페스트)
 */
export interface ExportPdfEnvelope {
  readonly backend?: string;
  readonly bytes?: number;
  readonly format?: string;
  readonly output?: string;
  readonly pageCount?: number;
  readonly renderedCount?: number;
  readonly schemaVersion?: string;
  readonly source?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-plan-schema --json` 봉투.
 *
 * 계획서(run) 문법의 JSON Schema 산출 — 계획 생성의 단일 출처 (#3719 §6-4)
 */
export interface ExportPlanSchemaEnvelope {
  readonly definitionCount?: unknown;
  readonly dialect?: unknown;
  readonly planSchemaVersion?: unknown;
  readonly schema?: unknown;
  readonly schemaVersion?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-provenance-map --json` 봉투.
 *
 * 명령별 문서 파생(신뢰 불가) 봉투 필드 지도 — 봉투의 untrustedContent/untrustedFields 표지의
 * 원천
 */
export interface ExportProvenanceMapEnvelope {
  readonly commands?: unknown;
  readonly envelopeFlags?: unknown;
  readonly pathSyntax?: unknown;
  readonly policy?: unknown;
  readonly schemaVersion?: string;
  readonly tool?: unknown;
  readonly version?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-structure --json` 봉투.
 *
 * 문서 개요/조문 계층을 JSON 트리로 추출
 */
export interface ExportStructureEnvelope {
  readonly mode?: string;
  readonly nodeCount?: number;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly structure?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-svg --json` 봉투.
 *
 * 문서를 페이지별 SVG로 렌더하고 --json 매니페스트 출력
 */
export interface ExportSvgEnvelope {
  readonly format?: string;
  readonly outputDir?: string;
  readonly pageCount?: number;
  readonly pages?: unknown;
  readonly renderedCount?: number;
  readonly schemaVersion?: string;
  readonly source?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-tables --json` 봉투.
 *
 * 표를 병합·중첩 구조를 보존한 격자 JSON으로 추출
 */
export interface ExportTablesEnvelope {
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly tableCount?: number;
  readonly tables?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp export-text --json` 봉투.
 *
 * 페이지별 텍스트 추출 (TXT 파일 또는 --json stdout)
 */
export interface ExportTextEnvelope {
  readonly omittedCount?: unknown;
  readonly pageCount?: number;
  readonly pages?: unknown;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly truncated?: boolean;

  readonly [key: string]: unknown;
}

/**
 * `rhwp extract-data --json` 봉투.
 *
 * 날짜·금액·수량을 구역·문단·페이지·문자 오프셋 주소와 함께 추출
 */
export interface ExtractDataEnvelope {
  readonly counts?: unknown;
  readonly itemCount?: unknown;
  readonly items?: unknown;
  readonly kind?: unknown;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly totalItemCount?: unknown;
  readonly truncated?: boolean;

  readonly [key: string]: unknown;
}

/**
 * `rhwp extract-pages --json` 봉투.
 *
 * 쪽 범위만 남겨 저장 (--json 봉투; 발췌·부분 제출·결함 이분법)
 */
export interface ExtractPagesEnvelope {
  readonly from?: number;
  readonly output?: string;
  readonly pagesAfter?: number;
  readonly pagesBefore?: number;
  readonly paragraphsKept?: number;
  readonly paragraphsRemoved?: number;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly to?: number;

  readonly [key: string]: unknown;
}

/**
 * `rhwp fields --json` 봉투.
 *
 * 누름틀/필드를 이름·안내문·현재값·위치와 함께 조사
 */
export interface FieldsEnvelope {
  readonly fieldCount?: number;
  readonly fields?: unknown;
  readonly schemaVersion?: string;
  readonly source?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp gate --json` 봉투.
 *
 * 반입 정책 기계 판정 — admissionPolicy(연산자 eq·in·gte·lte 4종 고정, deny 기본, 미지 키 로드
 * 거부)를 캡슐에 적용. 재료는 자기 신고가 아니라 재계산(계보·서명·앵커·--deep 재실행), 거부는
 * exit 3 + violations[] (#4545)
 */
export interface GateEnvelope {
  readonly evaluated?: unknown;
  readonly policy?: unknown;
  readonly policyPath?: unknown;
  readonly policySigned?: unknown;
  readonly schemaVersion?: string;
  readonly target?: unknown;
  readonly targetSha256?: unknown;
  readonly verdict?: unknown;
  readonly violations?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp harness --json` 봉투.
 *
 * 검증 루프의 쓰는 쪽 — init(작업장 규약)·wrap(실산출+영수증+캡슐+자동 부모 연결+서명 한 방).
 * 판정은 harness-status (#4537)
 */
export interface HarnessEnvelope {
  readonly capsule?: unknown;
  readonly dir?: unknown;
  readonly output?: string;
  readonly parent?: unknown;
  readonly schemaVersion?: string;
  readonly signed?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp harness-status --json` 봉투.
 *
 * 작업장 통합 판정 — 캡슐 체인 무결·(--keyring) 서명 집계·(--deep) 전수 재현을 한 봉투로. 깨짐
 * exit 3, brokenAt 이 원인 캡슐 (#4537)
 */
export interface HarnessStatusEnvelope {
  readonly brokenAt?: unknown;
  readonly capsules?: unknown;
  readonly chainValid?: unknown;
  readonly dir?: unknown;
  readonly reproduced?: unknown;
  readonly schemaVersion?: string;
  readonly signed?: unknown;
  readonly verdict?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp info --json` 봉투.
 *
 * 문서 메타(포맷·버전·페이지/문단 수·폰트·제목) 표시
 */
export interface InfoEnvelope {
  readonly fonts?: readonly string[];
  readonly format?: string;
  readonly pageCount?: number;
  readonly paraCount?: number;
  readonly schemaVersion?: string;
  readonly sections?: number;
  readonly sizeBytes?: number;
  readonly source?: string;
  readonly title?: string;
  readonly version?: string;
  readonly warnings?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp inspect --json` 봉투.
 *
 * 은닉 텍스트·프롬프트 주입·유니코드 기만을 조사하는 읽기 전용 보안 검사 명령군
 */
export interface InspectEnvelope {
  readonly clean?: unknown;
  readonly findingCount?: unknown;
  readonly findings?: unknown;
  readonly hiddenCharCount?: unknown;
  readonly hiddenText?: unknown;
  readonly highestConfidence?: unknown;
  readonly includeFields?: unknown;
  readonly includeOffPage?: unknown;
  readonly injectionSignals?: unknown;
  readonly kindCounts?: unknown;
  readonly kindFilter?: unknown;
  readonly minConfidence?: unknown;
  readonly scanScopes?: unknown;
  readonly scannedChars?: unknown;
  readonly schemaVersion?: string;
  readonly severityCounts?: unknown;
  readonly signalCount?: unknown;
  readonly source?: string;
  readonly thresholdPt?: unknown;
  readonly untrustedContent?: unknown;
  readonly untrustedFields?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp ir-diff --json` 봉투.
 *
 * 두 문서의 IR 차이를 JSON으로 비교
 */
export interface IrDiffEnvelope {
  readonly a?: string;
  readonly b?: string;
  readonly categories?: unknown;
  readonly diffCount?: number;
  readonly identical?: boolean;
  readonly schemaVersion?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp keygen --json` 봉투.
 *
 * Ed25519 서명키 파일 발급 — 캡슐 귀속(4년 축)의 시작점. 비밀키가 담기므로 기존 파일 덮어쓰기
 * 금지, 보관 책임은 소유자 (#4509)
 */
export interface KeygenEnvelope {
  readonly keyFile?: unknown;
  readonly keyId?: unknown;
  readonly publicKey?: unknown;
  readonly schemaVersion?: string;

  readonly [key: string]: unknown;
}

/**
 * `rhwp lineage --json` 봉투.
 *
 * 작업 캡슐 해시 체인을 거슬러 연대기를 검증 — 부모 파일 무결·계보 불변식(부모 산출=자식
 * 입력)·(--deep) 링크별 재현·(--keyring) 링크별 서명 귀속. 깨진 체인은 exit 3, brokenAt 명세
 * (#4401·#4509)
 */
export interface LineageEnvelope {
  readonly brokenAt?: unknown;
  readonly depth?: unknown;
  readonly head?: unknown;
  readonly links?: unknown;
  readonly schemaVersion?: string;
  readonly valid?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp render-diff --json` 봉투.
 *
 * 왕복/두 파일 렌더 기하 차이 검증 — --json 회귀 검출은 exit 3 (--batch 는 NDJSON)
 */
export interface RenderDiffEnvelope {
  readonly hardStructPages?: unknown;
  readonly maxDisp?: unknown;
  readonly mode?: unknown;
  readonly overPages?: unknown;
  readonly pageCountA?: unknown;
  readonly pageCountB?: unknown;
  readonly pageCountMismatch?: unknown;
  readonly pageFilter?: unknown;
  readonly pages?: unknown;
  readonly regression?: unknown;
  readonly schemaVersion?: string;
  readonly sourceA?: unknown;
  readonly sourceB?: unknown;
  readonly status?: unknown;
  readonly structPages?: unknown;
  readonly threshold?: unknown;
  readonly via?: unknown;
  readonly worstPage?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp replay --json` 봉투.
 *
 * 계획을 임시 산출로 재실행해 작업 영수증(입력·계획·산출 SHA-256)을 발급하고,
 * --expect-output-sha256 로 타인의 작업 주장을 재현 검증한다 — 불일치는 exit 3 (#4391)
 */
export interface ReplayEnvelope {
  readonly expectedOutputSha256?: unknown;
  readonly input?: unknown;
  readonly inputSha256?: unknown;
  readonly mode?: unknown;
  readonly outputSha256?: unknown;
  readonly planSha256?: unknown;
  readonly reproduced?: unknown;
  readonly schemaVersion?: string;
  readonly steps?: unknown;
  readonly toolVersion?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp run --json` 봉투.
 *
 * 선언적 편집 계획 실행 — 정적 선검증·원자 실행·저널 (#3703)
 */
export interface RunEnvelope {
  readonly input?: unknown;
  readonly invalid?: unknown;
  readonly output?: string;
  readonly outputFormat?: string;
  readonly planVersion?: string;
  readonly schemaVersion?: string;
  readonly steps?: unknown;
  readonly verify?: RawVerifyReport | null;

  readonly [key: string]: unknown;
}

/**
 * `rhwp scan --json` 봉투.
 *
 * 디렉터리 재귀 발견·분류 — 확장자↔매직 대조(extMismatch), --probe 파싱 시도(암호·쪽수), batch
 * stdin 목록의 원천
 */
export interface ScanEnvelope {
  readonly files?: unknown;
  readonly roots?: unknown;
  readonly schemaVersion?: string;
  readonly summary?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp search --json` 봉투.
 *
 * 문서 검색 결과를 구역·문단·페이지·문자 오프셋 주소와 함께 출력
 */
export interface SearchEnvelope {
  readonly caseSensitive?: boolean;
  readonly matchCount?: number;
  readonly matches?: unknown;
  readonly omittedCount?: unknown;
  readonly query?: string;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly totalMatchCount?: number;
  readonly truncated?: boolean;

  readonly [key: string]: unknown;
}

/**
 * `rhwp table-to-csv --json` 봉투.
 *
 * 본문 최상위 표를 병합 격자를 채운 RFC 4180 CSV 로 내보내기
 */
export interface TableToCsvEnvelope {
  readonly bom?: unknown;
  readonly output?: string;
  readonly outputFormat?: string;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly tableCount?: unknown;
  readonly tables?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp thumbnail --json` 봉투.
 *
 * 내장 썸네일(PrvImage) 추출 (--json 봉투)
 */
export interface ThumbnailEnvelope {
  readonly bytes?: number;
  readonly format?: string;
  readonly height?: number;
  readonly mime?: string;
  readonly output?: string;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly width?: number;

  readonly [key: string]: unknown;
}

/**
 * `rhwp verify --json` 봉투.
 *
 * 기대
 * 조건(--expect-pages/min-pages/max-pages/min-chars/min-tables/table-count/contains/not-contains/field/format)
 * 대조 — 전부 만족 exit 0, 불일치는 봉투 후 exit 3
 */
export interface VerifyEnvelope {
  readonly expectations?: unknown;
  readonly failCount?: unknown;
  readonly passCount?: unknown;
  readonly schemaVersion?: string;
  readonly source?: string;
  readonly verdict?: unknown;

  readonly [key: string]: unknown;
}

/**
 * `rhwp verify-signature --json` 봉투.
 *
 * 캡슐 분리 서명(<캡슐>.sig.json)을 파일 바이트·키 등록부와 대조 —
 * verdict(valid|invalid|unknownKey|revoked|malformed)는 봉투 데이터, 유효 아님 = exit 3 (#4509)
 */
export interface VerifySignatureEnvelope {
  readonly capsule?: unknown;
  readonly capsuleSha256?: unknown;
  readonly capsuleShaMatches?: unknown;
  readonly keyId?: unknown;
  readonly keyKnown?: unknown;
  readonly revoked?: unknown;
  readonly schemaVersion?: string;
  readonly sigPath?: unknown;
  readonly signatureOk?: unknown;
  readonly verdict?: unknown;

  readonly [key: string]: unknown;
}

/**
 * 명령 이름 → 봉투 타입.
 *
 * `recordFields` 를 선언한 명령만 들어 있습니다 — 나머지는 `--json` 봉투를 내지 않습니다.
 */
export interface EnvelopeByCommand {
  anchor: AnchorEnvelope;
  audit: AuditEnvelope;
  batch: BatchEnvelope;
  "build-from-ingest": BuildFromIngestEnvelope;
  bundle: BundleEnvelope;
  capabilities: CapabilitiesEnvelope;
  convert: ConvertEnvelope;
  "csv-to-table": CsvToTableEnvelope;
  digest: DigestEnvelope;
  "dump-pages": DumpPagesEnvelope;
  edit: EditEnvelope;
  explain: ExplainEnvelope;
  "export-agent-manifest": ExportAgentManifestEnvelope;
  "export-capabilities-schema": ExportCapabilitiesSchemaEnvelope;
  "export-doclang": ExportDoclangEnvelope;
  "export-hml": ExportHmlEnvelope;
  "export-hwpx": ExportHwpxEnvelope;
  "export-ir-schema": ExportIrSchemaEnvelope;
  "export-markdown": ExportMarkdownEnvelope;
  "export-ontology": ExportOntologyEnvelope;
  "export-pdf": ExportPdfEnvelope;
  "export-plan-schema": ExportPlanSchemaEnvelope;
  "export-provenance-map": ExportProvenanceMapEnvelope;
  "export-structure": ExportStructureEnvelope;
  "export-svg": ExportSvgEnvelope;
  "export-tables": ExportTablesEnvelope;
  "export-text": ExportTextEnvelope;
  "extract-data": ExtractDataEnvelope;
  "extract-pages": ExtractPagesEnvelope;
  fields: FieldsEnvelope;
  gate: GateEnvelope;
  harness: HarnessEnvelope;
  "harness-status": HarnessStatusEnvelope;
  info: InfoEnvelope;
  inspect: InspectEnvelope;
  "ir-diff": IrDiffEnvelope;
  keygen: KeygenEnvelope;
  lineage: LineageEnvelope;
  "render-diff": RenderDiffEnvelope;
  replay: ReplayEnvelope;
  run: RunEnvelope;
  scan: ScanEnvelope;
  search: SearchEnvelope;
  "table-to-csv": TableToCsvEnvelope;
  thumbnail: ThumbnailEnvelope;
  verify: VerifyEnvelope;
  "verify-signature": VerifySignatureEnvelope;
}

/** `--json` 봉투를 내는 명령 이름. */
export type EnvelopeCommand = keyof EnvelopeByCommand;

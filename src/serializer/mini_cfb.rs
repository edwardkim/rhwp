//! 최소 CFB (Compound File Binary) v3 빌더
//!
//! `cfb` 크레이트의 `CompoundFile::create()`가 `SystemTime::now()`를 호출하여
//! `wasm32-unknown-unknown` 타겟에서 panic이 발생하므로,
//! SystemTime을 사용하지 않는 자체 CFB 빌더를 구현한다.
//!
//! CFB v3 사양:
//! - 섹터 크기: 512바이트
//! - 미니 섹터 크기: 64바이트
//! - 미니 스트림 컷오프: 4096바이트 (표준값)

const SECTOR_SIZE: usize = 512;
const MINI_SECTOR_SIZE: usize = 64;
const MINI_STREAM_CUTOFF: usize = 4096;
const DIR_ENTRY_SIZE: usize = 128;
const ENTRIES_PER_DIR_SECTOR: usize = SECTOR_SIZE / DIR_ENTRY_SIZE; // 4
const FAT_ENTRIES_PER_SECTOR: usize = SECTOR_SIZE / 4; // 128
const HEADER_DIFAT_COUNT: usize = 109;
// DIFAT 섹터는 128 엔트리 중 마지막 1개를 다음 DIFAT 섹터 체인 포인터로 쓰므로
// FAT 섹터 포인터는 섹터당 127개만 담는다.
const DIFAT_ENTRIES_PER_SECTOR: usize = FAT_ENTRIES_PER_SECTOR - 1; // 127

const ENDOFCHAIN: u32 = 0xFFFFFFFE;
const FREESECT: u32 = 0xFFFFFFFF;
const FATSECT: u32 = 0xFFFFFFFD;
const DIFSECT: u32 = 0xFFFFFFFC;
const NOSTREAM: u32 = 0xFFFFFFFF;

/// CFB 시그니처 (Magic Number)
const CFB_SIGNATURE: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

struct DirEntry {
    name: String,
    obj_type: u8,
    data: Vec<u8>,
    parent: usize,
    children: Vec<usize>,
    left: u32,
    right: u32,
    child: u32,
    start_sector: u32,
    is_mini: bool,
    /// 디렉터리 엔트리 +80, 16바이트. OLE 개체는 이 값으로 서버를 식별한다 (#4097).
    ///
    /// 현재는 `build_cfb_with_root_clsid` 가 Root(5)에만 채우지만, 값을 여기 필드로 둬서
    /// 나중에 스토리지별 CLSID 가 필요해지면 `build_entries` 에 조회 한 줄만 추가하면
    /// 되게 한다 — `write_dir_entry` 는 무변경이다.
    clsid: [u8; 16],
}

impl DirEntry {
    fn new(name: &str, obj_type: u8, parent: usize) -> Self {
        DirEntry {
            name: name.to_string(),
            obj_type,
            data: Vec::new(),
            parent,
            children: Vec::new(),
            left: NOSTREAM,
            right: NOSTREAM,
            child: NOSTREAM,
            // Storage(1)는 start_sector=0 (MS-CFB 스펙: "SHOULD be set to all zeroes")
            // Root(5), Stream(2)은 ENDOFCHAIN → 나중에 실제 값으로 교체
            start_sector: if obj_type == 1 { 0 } else { ENDOFCHAIN },
            is_mini: false,
            clsid: [0u8; 16],
        }
    }
}

/// 명명된 스트림 목록으로 CFB v3 바이너리를 생성한다 (루트 CLSID 없음).
///
/// # 인자
/// - `named_streams`: `(경로, 데이터)` 쌍. 경로는 `/FileHeader`, `/BodyText/Section0` 형식.
///   구분자는 `/` 와 `\` 를 모두 받는다 (`build_entries` 가 정규화한다).
///
/// # 반환
/// CFB v3 바이너리 바이트.
///
/// # 중첩 OLE CFB 를 재포장할 때는 이 함수를 쓰면 안 된다
///
/// 루트 CLSID 를 `[0u8;16]` 으로 고정하므로 **`build_cfb_with_root_clsid` 를 써야 한다**.
/// OLE 개체는 루트 스토리지 엔트리의 CLSID 로 서버를 식별한다 — 비면 한컴이 개체를 알아보지
/// 못해 틀과 선택 핸들만 그리고 내용을 비운다
/// (#4097, 2026-08-05 한컴 실측 / `mydocs/report/task_m100_4055_report.md` §3).
///
/// 이 함수가 맞는 경우는 원본 루트 CLSID 가 애초에 0 인 CFB 다 — HWP5 바깥 CFB
/// (`serializer::cfb_writer::write_hwp_cfb`)가 그렇다.
pub fn build_cfb(named_streams: &[(&str, &[u8])]) -> Result<Vec<u8>, String> {
    build_cfb_with_root_clsid(named_streams, [0u8; 16])
}

/// 루트 CLSID 를 지정해 CFB v3 바이너리를 생성한다. (#4097)
///
/// 중첩 OLE CFB 재포장은 반드시 이 쪽을 쓰고, `root_clsid` 는 원본에서 읽어 넘긴다
/// (`parser::ole_container::ole_root_clsid` / `parser::cfb_reader::root_clsid`).
///
/// CLSID 를 실을 수 있는 엔트리는 MS-CFB 상 Root(5)와 Storage(1) 둘뿐이고 Stream(2)은 0 이어야
/// 한다. 현재 소비자(중첩 차트 CFB·HWP3 승격 재포장·B1 차트 편집)는 전부 **평탄 구조**라
/// 루트 외에 CLSID 를 실을 자리가 없으므로 루트만 받는다.
pub fn build_cfb_with_root_clsid(
    named_streams: &[(&str, &[u8])],
    root_clsid: [u8; 16],
) -> Result<Vec<u8>, String> {
    // 1. 엔트리 목록 구축
    let mut entries = build_entries(named_streams)?;
    entries[0].clsid = root_clsid;

    // 2. 디렉토리 트리 구축
    build_tree(&mut entries, 0);

    // 3. 미니 스트림 구축 (< 4096 바이트 스트림)
    let mut mini_stream = Vec::new();
    let mut mini_fat: Vec<u32> = Vec::new();

    for entry in entries.iter_mut() {
        if entry.obj_type == 2 && !entry.data.is_empty() && entry.data.len() < MINI_STREAM_CUTOFF {
            entry.is_mini = true;
            let start_mini = mini_fat.len();
            entry.start_sector = start_mini as u32;

            let num_mini = (entry.data.len() + MINI_SECTOR_SIZE - 1) / MINI_SECTOR_SIZE;
            for i in 0..num_mini {
                mini_fat.push(if i + 1 < num_mini {
                    (start_mini + i + 1) as u32
                } else {
                    ENDOFCHAIN
                });
            }

            mini_stream.extend_from_slice(&entry.data);
            let pad = (MINI_SECTOR_SIZE - (entry.data.len() % MINI_SECTOR_SIZE)) % MINI_SECTOR_SIZE;
            mini_stream.resize(mini_stream.len() + pad, 0);
        }
    }

    // Root Entry에 미니 스트림 컨테이너 저장
    let mini_stream_size = mini_stream.len();
    if !mini_stream.is_empty() {
        entries[0].data = mini_stream;
    }

    // 4. 정규 섹터 할당
    let dir_sectors = (entries.len() + ENTRIES_PER_DIR_SECTOR - 1) / ENTRIES_PER_DIR_SECTOR;
    let mut next_sector = dir_sectors as u32;

    // 큰 스트림 (>= 4096 바이트) → 정규 섹터
    for entry in entries.iter_mut() {
        if entry.obj_type == 2 && !entry.data.is_empty() && !entry.is_mini {
            entry.start_sector = next_sector;
            let num = (entry.data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE;
            next_sector += num as u32;
        }
    }

    // Root Entry 미니 스트림 컨테이너 → 정규 섹터
    if mini_stream_size > 0 {
        entries[0].start_sector = next_sector;
        let num = (entries[0].data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE;
        next_sector += num as u32;
    }

    // 미니 FAT 섹터
    let mini_fat_start;
    let mini_fat_sector_count;
    if !mini_fat.is_empty() {
        mini_fat_start = next_sector;
        mini_fat_sector_count =
            ((mini_fat.len() + FAT_ENTRIES_PER_SECTOR - 1) / FAT_ENTRIES_PER_SECTOR) as u32;
        next_sector += mini_fat_sector_count;
    } else {
        mini_fat_start = ENDOFCHAIN;
        mini_fat_sector_count = 0;
    }

    // FAT/DIFAT 섹터 수 계산 (고정점 반복)
    //
    // CFB v3 헤더는 FAT 섹터 포인터를 최대 109개만 담는다. FAT 섹터가 109개를
    // 초과하면(출력 > 약 7.14MB = 109 × 128 × 512 byte) 나머지 포인터는
    // DIFAT(이중 간접 FAT) 섹터에 기록해야 한다. FAT 섹터와 DIFAT 섹터 자체도
    // 섹터를 차지하여 total_sectors를 늘리고, 이는 다시 fat_count(→difat_count)를
    // 늘릴 수 있으므로 두 값을 함께 고정점 반복으로 수렴시킨다.
    let non_meta_sectors = next_sector; // FAT/DIFAT 제외 섹터 수
    let mut fat_count = 1u32;
    let mut difat_count = 0u32;
    loop {
        let total = non_meta_sectors + fat_count + difat_count;
        let needed_fat =
            (((total as usize) + FAT_ENTRIES_PER_SECTOR - 1) / FAT_ENTRIES_PER_SECTOR) as u32;
        let needed_difat = if needed_fat as usize > HEADER_DIFAT_COUNT {
            (((needed_fat as usize - HEADER_DIFAT_COUNT) + DIFAT_ENTRIES_PER_SECTOR - 1)
                / DIFAT_ENTRIES_PER_SECTOR) as u32
        } else {
            0
        };
        if needed_fat <= fat_count && needed_difat <= difat_count {
            break;
        }
        // 섹터 수는 단조 증가만 하므로 max로 수렴을 보장한다.
        fat_count = needed_fat.max(fat_count);
        difat_count = needed_difat.max(difat_count);
    }

    let fat_start = non_meta_sectors;
    let difat_start = fat_start + fat_count;
    let total_sectors = non_meta_sectors + fat_count + difat_count;

    // 5. FAT 구축
    let mut fat = vec![FREESECT; total_sectors as usize];

    // 디렉토리 체인
    for i in 0..dir_sectors {
        fat[i] = if i + 1 < dir_sectors {
            (i + 1) as u32
        } else {
            ENDOFCHAIN
        };
    }

    // 큰 스트림 체인
    for entry in entries.iter() {
        if entry.obj_type == 2 && !entry.data.is_empty() && !entry.is_mini {
            let start = entry.start_sector as usize;
            let num = (entry.data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE;
            for i in 0..num {
                fat[start + i] = if i + 1 < num {
                    (start + i + 1) as u32
                } else {
                    ENDOFCHAIN
                };
            }
        }
    }

    // Root Entry (미니 스트림 컨테이너) 체인
    if entries[0].start_sector != ENDOFCHAIN && !entries[0].data.is_empty() {
        let start = entries[0].start_sector as usize;
        let num = (entries[0].data.len() + SECTOR_SIZE - 1) / SECTOR_SIZE;
        for i in 0..num {
            fat[start + i] = if i + 1 < num {
                (start + i + 1) as u32
            } else {
                ENDOFCHAIN
            };
        }
    }

    // 미니 FAT 체인
    if mini_fat_start != ENDOFCHAIN {
        let start = mini_fat_start as usize;
        for i in 0..mini_fat_sector_count as usize {
            fat[start + i] = if i + 1 < mini_fat_sector_count as usize {
                (start + i + 1) as u32
            } else {
                ENDOFCHAIN
            };
        }
    }

    // FAT 섹터 마커
    for i in 0..fat_count as usize {
        fat[fat_start as usize + i] = FATSECT;
    }

    // DIFAT 섹터 마커
    for i in 0..difat_count as usize {
        fat[difat_start as usize + i] = DIFSECT;
    }

    // 6. 바이너리 조립
    let file_size = 512 + total_sectors as usize * SECTOR_SIZE;
    let mut output = vec![0u8; file_size];

    // 헤더 작성
    write_header(
        &mut output,
        fat_count,
        fat_start,
        mini_fat_start,
        mini_fat_sector_count,
        difat_start,
        difat_count,
    );

    // 디렉토리 엔트리 작성
    for (i, entry) in entries.iter().enumerate() {
        let sector_idx = i / ENTRIES_PER_DIR_SECTOR;
        let entry_in_sector = i % ENTRIES_PER_DIR_SECTOR;
        let offset = 512 + sector_idx * SECTOR_SIZE + entry_in_sector * DIR_ENTRY_SIZE;
        write_dir_entry(&mut output, offset, entry);
    }

    // 큰 스트림 데이터 작성
    for entry in &entries {
        if entry.obj_type == 2 && !entry.data.is_empty() && !entry.is_mini {
            let start_offset = 512 + entry.start_sector as usize * SECTOR_SIZE;
            output[start_offset..start_offset + entry.data.len()].copy_from_slice(&entry.data);
        }
    }

    // Root Entry 데이터 (미니 스트림 컨테이너) 작성
    if entries[0].start_sector != ENDOFCHAIN && !entries[0].data.is_empty() {
        let start_offset = 512 + entries[0].start_sector as usize * SECTOR_SIZE;
        output[start_offset..start_offset + entries[0].data.len()]
            .copy_from_slice(&entries[0].data);
    }

    // 미니 FAT 작성
    if mini_fat_start != ENDOFCHAIN {
        for (i, &mf) in mini_fat.iter().enumerate() {
            let sector_idx = i / FAT_ENTRIES_PER_SECTOR;
            let entry_in_sector = i % FAT_ENTRIES_PER_SECTOR;
            let offset =
                512 + (mini_fat_start as usize + sector_idx) * SECTOR_SIZE + entry_in_sector * 4;
            output[offset..offset + 4].copy_from_slice(&mf.to_le_bytes());
        }
    }

    // FAT 작성
    for (i, &fat_entry) in fat.iter().enumerate() {
        let fat_sector_idx = i / FAT_ENTRIES_PER_SECTOR;
        let entry_in_sector = i % FAT_ENTRIES_PER_SECTOR;
        let offset =
            512 + (fat_start as usize + fat_sector_idx) * SECTOR_SIZE + entry_in_sector * 4;
        output[offset..offset + 4].copy_from_slice(&fat_entry.to_le_bytes());
    }

    // DIFAT 섹터 작성
    // 헤더가 담는 109개를 제외한 나머지 FAT 섹터 포인터를 DIFAT 섹터에 기록한다.
    // 각 DIFAT 섹터: 엔트리 0..127 = FAT 섹터 SID, 엔트리 127 = 다음 DIFAT 섹터 체인.
    for d in 0..difat_count as usize {
        let sector_base = 512 + (difat_start as usize + d) * SECTOR_SIZE;
        for j in 0..DIFAT_ENTRIES_PER_SECTOR {
            let fat_idx = HEADER_DIFAT_COUNT + d * DIFAT_ENTRIES_PER_SECTOR + j;
            let value = if (fat_idx as u32) < fat_count {
                fat_start + fat_idx as u32
            } else {
                FREESECT
            };
            let off = sector_base + j * 4;
            output[off..off + 4].copy_from_slice(&value.to_le_bytes());
        }
        // 마지막 엔트리(127번): 다음 DIFAT 섹터 체인 (마지막 섹터면 ENDOFCHAIN)
        let next = if d + 1 < difat_count as usize {
            difat_start + (d as u32) + 1
        } else {
            ENDOFCHAIN
        };
        let off = sector_base + DIFAT_ENTRIES_PER_SECTOR * 4;
        output[off..off + 4].copy_from_slice(&next.to_le_bytes());
    }

    Ok(output)
}

/// 경로 목록에서 엔트리 목록을 구축한다.
///
/// 경로 구분자는 `/` 로 정규화한다. `cfb` 크레이트의 `Entry::path()` 는 `PathBuf` 라
/// Windows 에서 `/BinData\BIN0001.OLE` 처럼 구분자를 섞어 돌려준다(`Path::join` 이
/// `MAIN_SEPARATOR` 를 넣는다). 정규화하지 않으면 스토리지가 사라지고 이름에 역슬래시가 든
/// 루트 스트림 하나로 뭉개진다 (#4097).
/// MS-CFB §2.6.1 이 엔트리 이름에서 `/ \ : !` 를 금지하므로 이 치환은 무손실이다.
fn build_entries(named_streams: &[(&str, &[u8])]) -> Result<Vec<DirEntry>, String> {
    let mut entries = Vec::new();

    // Root Entry
    entries.push(DirEntry::new("Root Entry", 5, 0));

    for &(path, data) in named_streams {
        let normalized = path.replace('\\', "/");
        // 빈 세그먼트를 버려 선행 `/`, 중복 `//`, 후행 `/` 를 한꺼번에 처리한다.
        // 종전 `trim_start_matches('/')` 는 선행만 처리해 `/A/` 가 **이름 없는 스트림**이 됐다.
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return Err(format!("CFB 경로에 이름 세그먼트가 없다: {path:?}"));
        }
        let mut parent_idx = 0;

        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            let want_type = if is_last { 2u8 } else { 1u8 };

            // Root Entry(인덱스 0)는 `parent == 0` 이라 최상위 후보와 겹친다. 건너뛰지 않으면
            // 최상위에 "Root Entry" 이름 스트림이 올 때 루트 데이터가 덮어써진다 (#4097).
            let existing = entries
                .iter()
                .enumerate()
                .skip(1)
                .find(|(_, e)| e.parent == parent_idx && e.name == *part)
                .map(|(idx, _)| idx);

            if let Some(idx) = existing {
                // 같은 이름이 한 번은 스토리지, 한 번은 스트림으로 오면 CFB 로 표현할 수 없다.
                // 종전에는 스토리지에 `.data` 만 채우고 `write_dir_entry` 가 type 1 에 크기 0 을
                // 써서 그 데이터가 **조용히 소실**됐다 (#4097).
                if entries[idx].obj_type != want_type {
                    return Err(format!(
                        "CFB 경로 충돌: {path:?} 의 '{part}' 가 이미 {}(으)로 존재한다",
                        if entries[idx].obj_type == 1 {
                            "스토리지"
                        } else {
                            "스트림"
                        }
                    ));
                }
                if is_last {
                    entries[idx].data = data.to_vec();
                }
                parent_idx = idx;
            } else {
                let new_idx = entries.len();
                let mut entry = DirEntry::new(part, want_type, parent_idx);
                if is_last {
                    entry.data = data.to_vec();
                }
                entries[parent_idx].children.push(new_idx);
                entries.push(entry);
                parent_idx = new_idx;
            }
        }
    }

    Ok(entries)
}

/// 각 스토리지의 자식을 정렬된 균형 이진 트리로 구축한다.
fn build_tree(entries: &mut Vec<DirEntry>, idx: usize) {
    let children = entries[idx].children.clone();
    if children.is_empty() {
        entries[idx].child = NOSTREAM;
        return;
    }

    // CFB 사양에 따라 이름 비교: 길이 우선, 같은 길이면 대소문자 무시
    let mut sorted = children.clone();
    sorted.sort_by(|&a, &b| {
        let na = entries[a].name.to_uppercase();
        let nb = entries[b].name.to_uppercase();
        na.len().cmp(&nb.len()).then(na.cmp(&nb))
    });

    let root = build_balanced_tree(entries, &sorted);
    entries[idx].child = root;

    // 하위 스토리지에 대해 재귀
    for &child_idx in &children {
        if entries[child_idx].obj_type == 1 {
            build_tree(entries, child_idx);
        }
    }
}

/// 정렬된 인덱스 배열로 균형 이진 트리를 구축한다.
fn build_balanced_tree(entries: &mut Vec<DirEntry>, sorted: &[usize]) -> u32 {
    if sorted.is_empty() {
        return NOSTREAM;
    }
    let mid = sorted.len() / 2;
    let root = sorted[mid] as u32;

    let left = build_balanced_tree(entries, &sorted[..mid]);
    let right = if mid + 1 < sorted.len() {
        build_balanced_tree(entries, &sorted[mid + 1..])
    } else {
        NOSTREAM
    };

    entries[root as usize].left = left;
    entries[root as usize].right = right;
    root
}

/// CFB v3 헤더 (512바이트) 작성
fn write_header(
    output: &mut [u8],
    fat_count: u32,
    fat_start: u32,
    mini_fat_start: u32,
    mini_fat_sector_count: u32,
    difat_start: u32,
    difat_count: u32,
) {
    // 시그니처
    output[0..8].copy_from_slice(&CFB_SIGNATURE);

    // CLSID (16바이트 zero) — 이미 0

    // Minor version: 0x003E
    output[24..26].copy_from_slice(&0x003Eu16.to_le_bytes());
    // Major version: 0x0003 (v3)
    output[26..28].copy_from_slice(&0x0003u16.to_le_bytes());
    // Byte order: 0xFFFE (little-endian)
    output[28..30].copy_from_slice(&0xFFFEu16.to_le_bytes());
    // Sector shift: 9 (512 bytes)
    output[30..32].copy_from_slice(&9u16.to_le_bytes());
    // Mini sector shift: 6 (64 bytes)
    output[32..34].copy_from_slice(&6u16.to_le_bytes());

    // Reserved (6 bytes) — 이미 0

    // Total directory sectors: 0 (v3에서는 미사용)
    // output[40..44] — 이미 0

    // Total FAT sectors
    output[44..48].copy_from_slice(&fat_count.to_le_bytes());

    // First directory sector SID: 0 (항상 섹터 0부터)
    output[48..52].copy_from_slice(&0u32.to_le_bytes());

    // Transaction signature: 0
    // output[52..56] — 이미 0

    // Mini stream cutoff: 4096 (표준값)
    output[56..60].copy_from_slice(&(MINI_STREAM_CUTOFF as u32).to_le_bytes());

    // First mini FAT sector
    output[60..64].copy_from_slice(&mini_fat_start.to_le_bytes());
    // Total mini FAT sectors
    output[64..68].copy_from_slice(&mini_fat_sector_count.to_le_bytes());

    // First DIFAT sector: DIFAT 섹터가 있으면 그 시작 SID, 없으면 ENDOFCHAIN
    let first_difat = if difat_count > 0 {
        difat_start
    } else {
        ENDOFCHAIN
    };
    output[68..72].copy_from_slice(&first_difat.to_le_bytes());
    // Total DIFAT sectors
    output[72..76].copy_from_slice(&difat_count.to_le_bytes());

    // 헤더 내 DIFAT 배열 (선두 109개 FAT 섹터 포인터, 각 4바이트, 바이트 오프셋 76부터)
    // FAT 섹터가 109개를 초과하는 나머지는 DIFAT 섹터에 기록된다.
    let header_difat_offset = 76;
    for i in 0..HEADER_DIFAT_COUNT {
        let offset = header_difat_offset + i * 4;
        if (i as u32) < fat_count {
            let sid = fat_start + i as u32;
            output[offset..offset + 4].copy_from_slice(&sid.to_le_bytes());
        } else {
            output[offset..offset + 4].copy_from_slice(&FREESECT.to_le_bytes());
        }
    }
}

/// 디렉토리 엔트리 (128바이트) 작성
fn write_dir_entry(output: &mut [u8], offset: usize, entry: &DirEntry) {
    let buf = &mut output[offset..offset + DIR_ENTRY_SIZE];

    // 이름 (UTF-16LE, null 종료, 최대 32 UTF-16 코드 유닛)
    let name_utf16: Vec<u16> = entry.name.encode_utf16().collect();
    let name_len = name_utf16.len().min(31); // 최대 31자 + null
    for i in 0..name_len {
        let pos = i * 2;
        buf[pos..pos + 2].copy_from_slice(&name_utf16[i].to_le_bytes());
    }
    // null 종료
    let null_pos = name_len * 2;
    buf[null_pos..null_pos + 2].copy_from_slice(&0u16.to_le_bytes());

    // 이름 길이 (바이트, null 포함)
    let name_byte_len = ((name_len + 1) * 2) as u16;
    buf[64..66].copy_from_slice(&name_byte_len.to_le_bytes());

    // Object type
    buf[66] = entry.obj_type;

    // Color flag: 1 = black (유효한 red-black 트리)
    buf[67] = 1;

    // Left sibling
    buf[68..72].copy_from_slice(&entry.left.to_le_bytes());
    // Right sibling
    buf[72..76].copy_from_slice(&entry.right.to_le_bytes());
    // Child
    buf[76..80].copy_from_slice(&entry.child.to_le_bytes());

    // CLSID (16 bytes) — OLE 서버 식별자 (#4097).
    // MS-CFB 상 Stream(2)의 CLSID 는 0 이어야 하고, 현재 설계상 비-0 이 들어오는 엔트리는
    // Root(5) 뿐이다(`build_cfb_with_root_clsid`). 값을 특별 취급하지 않고 엔트리 필드를
    // 그대로 쓰므로, 스토리지별 CLSID 를 지원하게 돼도 이 함수는 무변경이다.
    buf[80..96].copy_from_slice(&entry.clsid);

    // State bits — 이미 0

    // Creation/Modified time (FILETIME, 8 bytes each)
    // Root Entry(5)와 Storage(1)에 고정 타임스탬프 설정
    // WASM에서 SystemTime::now()를 사용할 수 없으므로 고정값 사용
    // 2024-01-01 00:00:00 UTC ≈ 0x01DA5E8B_80000000
    if entry.obj_type == 5 || entry.obj_type == 1 {
        let filetime: u64 = 0x01DA_5E8B_8000_0000;
        let ft_bytes = filetime.to_le_bytes();
        buf[100..108].copy_from_slice(&ft_bytes); // Creation time
        buf[108..116].copy_from_slice(&ft_bytes); // Modified time
    }

    // Start sector
    buf[116..120].copy_from_slice(&entry.start_sector.to_le_bytes());

    // Stream size (lower 32 bits)
    // type 2 (스트림): 원본 데이터 크기
    // type 5 (루트): 미니 스트림 컨테이너 크기
    let size = if entry.obj_type == 2 || entry.obj_type == 5 {
        entry.data.len() as u32
    } else {
        0
    };
    buf[120..124].copy_from_slice(&size.to_le_bytes());

    // Stream size (upper 32 bits, v3: must be 0)
    // buf[124..128] — 이미 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cfb_signature() {
        let streams = vec![("/TestStream", b"Hello" as &[u8])];
        let bytes = build_cfb(&streams).unwrap();

        assert!(bytes.len() >= 512);
        assert_eq!(&bytes[0..8], &CFB_SIGNATURE);
    }

    #[test]
    fn test_build_cfb_empty() {
        let streams: Vec<(&str, &[u8])> = Vec::new();
        let bytes = build_cfb(&streams).unwrap();

        assert_eq!(&bytes[0..8], &CFB_SIGNATURE);
    }

    #[test]
    fn test_build_cfb_readable_by_cfb_crate() {
        let fh = vec![0xAAu8; 256];
        let di = vec![0xBBu8; 100];
        let streams = vec![("/FileHeader", fh.as_slice()), ("/DocInfo", di.as_slice())];
        let bytes = build_cfb(&streams).unwrap();

        // cfb 크레이트로 읽기
        let cursor = std::io::Cursor::new(&bytes);
        let mut cfb = cfb::CompoundFile::open(cursor).unwrap();

        let mut fh_read = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/FileHeader").unwrap(), &mut fh_read)
            .unwrap();
        assert_eq!(fh_read.len(), 256);
        assert!(fh_read.iter().all(|&b| b == 0xAA));

        let mut di_read = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/DocInfo").unwrap(), &mut di_read)
            .unwrap();
        assert_eq!(di_read.len(), 100);
        assert!(di_read.iter().all(|&b| b == 0xBB));
    }

    #[test]
    fn test_build_cfb_with_storages() {
        let d1 = vec![0x01u8; 256];
        let d2 = vec![0x02u8; 500];
        let d3 = vec![0x03u8; 2000];
        let d4 = vec![0x04u8; 1500];
        let streams = vec![
            ("/FileHeader", d1.as_slice()),
            ("/DocInfo", d2.as_slice()),
            ("/BodyText/Section0", d3.as_slice()),
            ("/BodyText/Section1", d4.as_slice()),
        ];
        let bytes = build_cfb(&streams).unwrap();

        let cursor = std::io::Cursor::new(&bytes);
        let mut cfb = cfb::CompoundFile::open(cursor).unwrap();

        let mut s0 = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/BodyText/Section0").unwrap(), &mut s0)
            .unwrap();
        assert_eq!(s0.len(), 2000);
        assert!(s0.iter().all(|&b| b == 0x03));

        let mut s1 = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/BodyText/Section1").unwrap(), &mut s1)
            .unwrap();
        assert_eq!(s1.len(), 1500);
        assert!(s1.iter().all(|&b| b == 0x04));
    }

    #[test]
    fn test_build_cfb_large_stream() {
        // 10KB 스트림 (다중 섹터, >= 4096이므로 정규 섹터)
        let data = vec![0x55u8; 10240];
        let streams = vec![("/BigStream", data.as_slice())];
        let bytes = build_cfb(&streams).unwrap();

        let cursor = std::io::Cursor::new(&bytes);
        let mut cfb = cfb::CompoundFile::open(cursor).unwrap();

        let mut read_data = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/BigStream").unwrap(), &mut read_data)
            .unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_build_cfb_mixed_sizes() {
        // 미니 스트림(< 4096)과 정규 스트림(>= 4096) 혼합
        let small = vec![0x11u8; 100];
        let large = vec![0x22u8; 5000];
        let streams = vec![("/Small", small.as_slice()), ("/Large", large.as_slice())];
        let bytes = build_cfb(&streams).unwrap();

        let cursor = std::io::Cursor::new(&bytes);
        let mut cfb = cfb::CompoundFile::open(cursor).unwrap();

        let mut s_read = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/Small").unwrap(), &mut s_read).unwrap();
        assert_eq!(s_read, small);

        let mut l_read = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream("/Large").unwrap(), &mut l_read).unwrap();
        assert_eq!(l_read, large);
    }

    #[test]
    fn test_build_cfb_difat_over_threshold() {
        // 회귀(#1227): FAT 섹터가 109개를 초과하면(헤더 DIFAT 슬롯 109개 한계 →
        // 출력 ≈ 109×128×512 = 7,143,424 byte ≈ 7.14MB 초과) DIFAT 섹터가 필요하다.
        // 과거 mini_cfb는 DIFAT 미작성으로 109개 초과분 FAT 섹터 위치가 유실되어
        // FAT 체인이 단절, cfb 크레이트가 "next_id invalid"로 열기에 실패했다.
        //
        // 임계값 바로 위(약 7.2MB)로 최소화해 CI 메모리/시간 부담을 줄인다. 이보다
        // 작으면 FAT 섹터가 109개 이하라 DIFAT 경로를 타지 않으므로 더 줄일 수 없다.
        // 결정적 패턴을 써서 별도 대용량 기대 버퍼 없이 검증하고, 입력은 즉시 해제한다.
        let n = 7_200_000usize;
        let big: Vec<u8> = (0..n).map(|i| (i % 251) as u8).collect();
        let bytes = {
            let streams = vec![("/BinData/BIN0001", big.as_slice())];
            build_cfb(&streams).unwrap()
        };
        drop(big); // 입력 버퍼 즉시 해제 — 동시 보유 메모리 절감

        // 헤더에 DIFAT 섹터가 기록되었는지 확인
        let first_difat = u32::from_le_bytes(bytes[68..72].try_into().unwrap());
        let num_difat = u32::from_le_bytes(bytes[72..76].try_into().unwrap());
        assert!(num_difat > 0, "출력이 7.14MB를 넘는데 DIFAT 섹터가 0개");
        assert_ne!(
            first_difat, ENDOFCHAIN,
            "DIFAT가 필요한데 first_difat가 ENDOFCHAIN"
        );

        // cfb 크레이트로 라운드트립 검증 (FAT 체인이 온전해야 열림)
        let cursor = std::io::Cursor::new(&bytes);
        let mut cfb = cfb::CompoundFile::open(cursor).unwrap();
        let mut read_data = Vec::new();
        std::io::Read::read_to_end(
            &mut cfb.open_stream("/BinData/BIN0001").unwrap(),
            &mut read_data,
        )
        .unwrap();
        // 길이 + 결정적 패턴 일치로 검증 (별도 대용량 기대 버퍼 보유 없음)
        assert_eq!(read_data.len(), n);
        assert!(
            read_data
                .iter()
                .enumerate()
                .all(|(i, &b)| b == (i % 251) as u8),
            "라운드트립 데이터가 원본 패턴과 불일치"
        );
    }

    // ── [#4097] 루트 CLSID 보존과 경로 정규화 ────────────────────────────────

    /// 스트림 하나를 `cfb` 크레이트로 되읽는다 — 판정 오라클은 우리 코드가 아니라 크레이트다.
    fn read_stream(bytes: &[u8], path: &str) -> Vec<u8> {
        let mut cfb = cfb::CompoundFile::open(std::io::Cursor::new(bytes)).expect("CFB 열기");
        let mut out = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream(path).expect("스트림 열기"), &mut out)
            .expect("스트림 읽기");
        out
    }

    #[test]
    fn test_build_cfb_normalizes_backslash_path_separator() {
        // Windows 에서 `cfb::Entry::path()` 는 `/BinData\BIN0001.OLE` 를 돌려준다.
        // 정규화 없이 넘기면 스토리지가 사라지고 역슬래시가 든 루트 스트림 하나로 뭉개진다.
        let data = vec![0x77u8; 300];
        let back = build_cfb(&[("/BinData\\BIN0001.OLE", data.as_slice())]).unwrap();
        let fwd = build_cfb(&[("/BinData/BIN0001.OLE", data.as_slice())]).unwrap();

        // 구분자 표기만 다른 경로는 플랫폼과 무관하게 같은 CFB 를 만들어야 한다.
        assert_eq!(back, fwd, "구분자 표기가 출력 바이트를 바꾸면 안 된다");
        // 스토리지 구조가 실제로 살아 있는지 독립 리더로 확인한다.
        assert_eq!(read_stream(&back, "/BinData/BIN0001.OLE"), data);
    }

    #[test]
    fn test_build_cfb_collapses_empty_path_segments() {
        let data = vec![0x11u8; 10];
        let messy = build_cfb(&[("//A//B/", data.as_slice())]).unwrap();
        let clean = build_cfb(&[("/A/B", data.as_slice())]).unwrap();
        assert_eq!(
            messy, clean,
            "빈 세그먼트는 이름 없는 엔트리를 만들면 안 된다"
        );
        assert_eq!(read_stream(&messy, "/A/B"), data);
    }

    #[test]
    fn test_build_cfb_rejects_degenerate_paths() {
        let data = b"x" as &[u8];
        // 이름 세그먼트가 하나도 없는 경로는 만들 수 있는 엔트리가 없다.
        assert!(build_cfb(&[("", data)]).is_err());
        assert!(build_cfb(&[("/", data)]).is_err());
        assert!(build_cfb(&[("\\", data)]).is_err());
        assert!(build_cfb(&[("///", data)]).is_err());
    }

    #[test]
    fn test_build_entries_does_not_clobber_root_entry() {
        // 최상위에 "Root Entry" 이름 스트림이 와도 루트 엔트리가 덮어써지면 안 된다.
        // 종전에는 dedup 후보에 인덱스 0(Root, parent==0)이 들어가 루트 데이터가 날아갔다.
        let data = vec![0xEEu8; 64];
        let bytes = build_cfb(&[("/Root Entry", data.as_slice())]).unwrap();
        assert_eq!(read_stream(&bytes, "/Root Entry"), data);
    }

    #[test]
    fn test_build_cfb_rejects_storage_stream_conflict() {
        // 같은 이름을 스토리지로도 스트림으로도 쓰면 CFB 로 표현할 수 없다.
        // 종전에는 조용히 데이터가 소실됐다.
        let data = vec![0x22u8; 10];
        assert!(build_cfb(&[("/A/B", data.as_slice()), ("/A", data.as_slice())]).is_err());
        assert!(build_cfb(&[("/A", data.as_slice()), ("/A/B", data.as_slice())]).is_err());
    }

    #[test]
    fn test_build_cfb_with_root_clsid_writes_dir_entry_offset_80() {
        // 코퍼스 차트가 달고 있는 실제 CLSID {4C3DA137-DC90-47B9-9BED-59DAE352A280}.
        const CLSID: [u8; 16] = [
            0x37, 0xa1, 0x3d, 0x4c, 0x90, 0xdc, 0xb9, 0x47, 0x9b, 0xed, 0x59, 0xda, 0xe3, 0x52,
            0xa2, 0x80,
        ];
        let data = vec![0x99u8; 32];
        let bytes = build_cfb_with_root_clsid(&[("/Contents", data.as_slice())], CLSID).unwrap();

        // 헤더 CLSID(8..24)는 MS-CFB 상 0 이어야 한다 — 값이 가는 곳은 여기가 아니다.
        assert_eq!(&bytes[8..24], &[0u8; 16], "헤더 CLSID 는 0 이어야 한다");
        // mini_cfb 는 sector_shift=9, first dir sector=0 고정이라 루트 엔트리는 항상 512 다.
        assert_eq!(
            &bytes[512 + 80..512 + 96],
            &CLSID,
            "루트 디렉터리 엔트리 +80"
        );
        // Stream(2) 엔트리의 CLSID 는 0 이어야 한다 (MS-CFB 요구, cfb Strict 요건).
        assert_eq!(&bytes[512 + 128 + 80..512 + 128 + 96], &[0u8; 16]);

        // 독립 리더가 같은 값을 읽는다.
        let via_crate = cfb::CompoundFile::open(std::io::Cursor::new(&bytes))
            .unwrap()
            .root_entry()
            .clsid()
            .to_bytes_le();
        assert_eq!(via_crate, CLSID);

        // 인자 없는 API 는 "CLSID 없음"으로 위임한다 — 이것이 계약이다.
        let zeroed = build_cfb(&[("/Contents", data.as_slice())]).unwrap();
        assert_eq!(&zeroed[512 + 80..512 + 96], &[0u8; 16]);
    }
}

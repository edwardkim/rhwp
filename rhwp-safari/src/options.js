'use strict';

// ─── 탭 전환 ───

document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById('tab-' + btn.dataset.tab).classList.add('active');
  });
});

// ─── 설정 ───

const DEFAULT_DOMAINS = ['.go.kr', '.or.kr', '.ac.kr', '.mil.kr', '.korea.kr', '.sc.kr'];
const TOGGLE_KEYS = [
  'allSitesEnabled',
  'autoOpen',
  'showBadges',
  'hoverPreview',
  'disableExternalWebFonts',
  'allowHttp',
  'httpWarning',
  'devMode',
  'securityLog',
];
const DEFAULTS = {
  allowedDomains: DEFAULT_DOMAINS,
  allSitesEnabled: false,
  autoOpen: true,
  showBadges: true,
  hoverPreview: true,
  disableExternalWebFonts: false,
  allowHttp: true,
  httpWarning: true,
  maxFileSize: 20,
  devMode: false,
  securityLog: false,
};

let currentDomains = [];

// ─── 설정 로드 ───

browser.storage.local.get(DEFAULTS).then(settings => {
  currentDomains = settings.allowedDomains || DEFAULT_DOMAINS;
  renderDomains();
  for (const key of TOGGLE_KEYS) {
    document.getElementById(key).checked = !!settings[key];
  }
  document.getElementById('maxFileSize').value = settings.maxFileSize || 20;
});

// ─── 도메인 목록 ───

function renderDomains() {
  const list = document.getElementById('domain-list');
  list.innerHTML = '';
  for (const domain of currentDomains) {
    const li = document.createElement('li');
    li.textContent = domain;
    const btn = document.createElement('button');
    btn.className = 'remove-btn';
    btn.textContent = '\u2715';
    btn.addEventListener('click', () => {
      currentDomains = currentDomains.filter(d => d !== domain);
      renderDomains();
      save();
    });
    li.appendChild(btn);
    list.appendChild(li);
  }
}

document.getElementById('add-domain-btn').addEventListener('click', () => {
  const input = document.getElementById('new-domain');
  let domain = input.value.trim();
  if (!domain) return;
  if (!domain.startsWith('.')) domain = '.' + domain;
  if (!currentDomains.includes(domain)) {
    currentDomains.push(domain);
    renderDomains();
    save();
  }
  input.value = '';
});

// ─── 자동 저장 ───

for (const key of TOGGLE_KEYS) {
  document.getElementById(key).addEventListener('change', save);
}
document.getElementById('maxFileSize').addEventListener('change', save);

function save() {
  const settings = { allowedDomains: currentDomains };
  for (const key of TOGGLE_KEYS) {
    settings[key] = document.getElementById(key).checked;
  }
  settings.maxFileSize = parseInt(document.getElementById('maxFileSize').value) || 20;
  browser.storage.local.set(settings).then(() => {
    const msg = document.getElementById('saved-msg');
    msg.classList.add('show');
    setTimeout(() => msg.classList.remove('show'), 1500);
  }).catch(err => {
    console.error('[rhwp-options] 저장 실패:', err);
  });
}

// ─── 보안 로그 ───

const LOG_TYPE_LABELS = {
  'fetch-blocked': '파일 차단',
  'url-blocked': 'URL 차단',
  'sender-blocked': '발신자 차단',
  'signature-blocked': '서명 불일치',
};

const LOG_TYPE_CLASSES = {
  'fetch-blocked': 'log-type-fetch',
  'url-blocked': 'log-type-url',
  'sender-blocked': 'log-type-sender',
  'signature-blocked': 'log-type-signature',
};

function formatLogTime(isoString) {
  try {
    const d = new Date(isoString);
    const pad = n => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
  } catch {
    return isoString || '-';
  }
}

function renderLog(events) {
  const container = document.getElementById('log-container');
  const tbody = document.getElementById('log-tbody');
  const empty = document.getElementById('log-empty');
  const count = document.getElementById('log-count');

  container.style.display = 'block';
  tbody.innerHTML = '';

  if (!events || events.length === 0) {
    empty.style.display = 'block';
    count.textContent = '0건';
    return;
  }

  empty.style.display = 'none';
  count.textContent = `${events.length}건`;

  // 최신순 표시
  const sorted = [...events].reverse();
  for (const evt of sorted) {
    const tr = document.createElement('tr');

    const tdTime = document.createElement('td');
    tdTime.textContent = formatLogTime(evt.time);
    tdTime.style.whiteSpace = 'nowrap';
    tr.appendChild(tdTime);

    const tdType = document.createElement('td');
    const typeSpan = document.createElement('span');
    typeSpan.className = `log-type ${LOG_TYPE_CLASSES[evt.type] || ''}`;
    typeSpan.textContent = LOG_TYPE_LABELS[evt.type] || evt.type;
    tdType.appendChild(typeSpan);
    tr.appendChild(tdType);

    const tdUrl = document.createElement('td');
    tdUrl.textContent = evt.url || '-';
    tdUrl.title = evt.url || '';
    tr.appendChild(tdUrl);

    const tdReason = document.createElement('td');
    tdReason.textContent = evt.reason || '-';
    tr.appendChild(tdReason);

    tbody.appendChild(tr);
  }
}

document.getElementById('load-log-btn').addEventListener('click', () => {
  browser.storage.local.get({ securityEvents: [] }).then(data => {
    renderLog(data.securityEvents);
  });
});

document.getElementById('clear-log-btn').addEventListener('click', () => {
  if (confirm('보안 로그를 모두 삭제하시겠습니까?')) {
    browser.storage.local.set({ securityEvents: [] }).then(() => {
      renderLog([]);
    });
  }
});

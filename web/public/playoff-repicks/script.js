const e_prefix = window.e_prefix ?? '';

// ---- Theme toggle (same pattern as the playoff page) ----
function initTheme() {
  const saved = localStorage.getItem('theme') || 'dark';
  document.documentElement.setAttribute('data-bs-theme', saved);
  updateThemeIcon(saved);
}

function toggleTheme() {
  const cur = document.documentElement.getAttribute('data-bs-theme');
  const next = cur === 'light' ? 'dark' : 'light';
  document.documentElement.setAttribute('data-bs-theme', next);
  localStorage.setItem('theme', next);
  updateThemeIcon(next);
}

function updateThemeIcon(theme) {
  const icon = document.getElementById('theme-icon');
  if (icon) icon.className = theme === 'light' ? 'bi bi-moon-fill' : 'bi bi-sun-fill';
}

document.getElementById('theme-toggle')?.addEventListener('click', toggleTheme);
initTheme();

// ---- Team lookup: sname -> { sname, name, division, conf } ----
const TEAMS = {};
afcTeams.forEach(t => TEAMS[t.sname] = { ...t, conf: 'AFC' });
nfcTeams.forEach(t => TEAMS[t.sname] = { ...t, conf: 'NFC' });

// Display order is cosmetic; winners are keyed by division name, not index.
const DISPLAY_DIVS = ['East', 'North', 'South', 'West'];

// ---- Build the 14 editable slots from the original picks ----
// One slot = one capsule pick. The slot carries its structural role so the
// swap modal (next step) can compute valid replacements.
function buildSlots(conf, winners, wildcards) {
  const slots = [];
  DISPLAY_DIVS.forEach(div => {
    slots.push({ id: `${conf}-win-${div}`, conf, kind: 'winner', division: div, label: `${conf} ${div}`, team: winners[div] });
  });
  wildcards.forEach((team, i) => {
    slots.push({ id: `${conf}-wc-${i}`, conf, kind: 'wildcard', slot: i, label: `${conf} Wild Card`, team });
  });
  return slots;
}

const slots = [
  ...buildSlots('AFC', originalPicks.afcWinners, originalPicks.afcWildcards),
  ...buildSlots('NFC', originalPicks.nfcWinners, originalPicks.nfcWildcards),
];

// Original snapshot + working copy, both keyed by slot id (values are snames).
const original = {};
slots.forEach(s => original[s.id] = s.team);
const current = { ...original };

// ---- Budget ----
// "changed" = slots whose current team differs from the original (net diff, so
// reverting a slot frees budget). remaining = budget - changed.
function changedCount() {
  return slots.filter(s => current[s.id] !== original[s.id]).length;
}

function remaining() {
  return repicksBudget - changedCount();
}

// ---- Rendering ----
function cardHtml(s) {
  const sname = current[s.id];
  const team = TEAMS[sname];
  const origTeam = TEAMS[original[s.id]];
  const changed = current[s.id] !== original[s.id];
  // An unchanged slot can't be opened once the budget is spent; a changed slot
  // stays editable so the pooler can re-pick or revert it.
  const locked = !changed && remaining() === 0;

  return `
    <div class="card repick-card mb-3 ${changed ? 'changed' : ''} ${locked ? 'locked' : ''}" data-slot="${s.id}">
      <div class="card-body p-3">
        <div class="d-flex justify-content-between align-items-center mb-2">
          <span class="badge slot-label text-uppercase">${s.label}</span>
          <button class="btn btn-sm btn-outline-secondary swap-btn" ${locked ? 'disabled' : ''} onclick="openSwap('${s.id}')">
            <i class="bi bi-arrow-left-right"></i> Échanger
          </button>
        </div>
        <div class="d-flex align-items-center">
          <img src="/teams/${e_prefix}${sname}.png" alt="${team.name}" class="team-logo me-2">
          <div>
            <div class="fw-bold team-name">${team.name}</div>
            <div class="text-muted small">${team.conf}</div>
          </div>
        </div>
        <div class="swap-out small">
          ${changed ? `
            <span class="badge out-badge me-1">Retiré</span>
            <img src="/teams/${e_prefix}${original[s.id]}.png" alt="${origTeam.name}" class="team-logo-sm me-1">
            <span class="text-decoration-line-through text-muted">${origTeam.name}</span>
          ` : ''}
        </div>
      </div>
    </div>
  `;
}

function render() {
  document.getElementById('afc-picks').innerHTML =
    slots.filter(s => s.conf === 'AFC').map(cardHtml).join('');
  document.getElementById('nfc-picks').innerHTML =
    slots.filter(s => s.conf === 'NFC').map(cardHtml).join('');

  document.getElementById('changes-remaining').textContent = remaining();

  // Review is reachable once at least one slot changed (and never over budget).
  const n = changedCount();
  document.getElementById('review-btn').disabled = !(n >= 1 && n <= repicksBudget);
}

// ---- Swap modal ----
const swapModal = new bootstrap.Modal(document.getElementById('swap-modal'));
let swapSlotId = null;

// Teams used in every slot EXCEPT the one being edited (set of snames).
function usedTeams(exceptSlotId) {
  const used = new Set();
  slots.forEach(s => { if (s.id !== exceptSlotId) used.add(current[s.id]); });
  return used;
}

// Role-valid teams for a slot: a winner slot is limited to its division,
// a wildcard slot can be any team in the conference.
function roleTeams(slot) {
  const pool = slot.conf === 'AFC' ? afcTeams : nfcTeams;
  return slot.kind === 'winner' ? pool.filter(t => t.division === slot.division) : pool;
}

// Find the wildcard slot (same conference) currently holding a team, if any.
function wildcardSlotHolding(conf, sname) {
  return slots.find(s => s.kind === 'wildcard' && s.conf === conf && current[s.id] === sname);
}

// Would promoting the wildcard at wcId into the winner slot (and demoting the
// current winner into that wildcard slot) push the changed count over budget?
function atomicWouldExceed(winnerId, wcId) {
  const before = (current[winnerId] !== original[winnerId] ? 1 : 0)
               + (current[wcId]     !== original[wcId]     ? 1 : 0);
  const after  = (current[wcId]     !== original[winnerId] ? 1 : 0)
               + (current[winnerId] !== original[wcId]     ? 1 : 0);
  return changedCount() - before + after > repicksBudget;
}

// Decide how a candidate team can be selected for the given slot.
// On a winner slot a used same-division team is necessarily held as a wildcard,
// so we offer an atomic promote (winner <-> wildcard) unless it busts the budget.
// Wildcard slots keep used teams locked.
function candidateAction(slot, t, used) {
  if (!used.has(t.sname)) return { mode: 'simple' };
  if (slot.kind === 'winner') {
    const wc = wildcardSlotHolding(slot.conf, t.sname);
    if (wc && !atomicWouldExceed(slot.id, wc.id)) return { mode: 'atomic', wcSlotId: wc.id };
  }
  return { mode: 'disabled' };
}

// action: { mode: 'simple' | 'atomic' | 'disabled', wcSlotId? }
function swapRowHtml(slot, t, action) {
  const disabled = action.mode === 'disabled';
  let onclick = '';
  if (action.mode === 'simple') onclick = `onclick="chooseSwap('${t.sname}')"`;
  else if (action.mode === 'atomic') onclick = `onclick="atomicSwap('${slot.id}', '${action.wcSlotId}')"`;

  let note = '';
  if (disabled) note = '<span class="text-muted small"><i class="bi bi-slash-circle"></i> Déjà choisi</span>';
  else if (action.mode === 'atomic') note = '<span class="text-info small"><i class="bi bi-arrow-left-right"></i> Promouvoir</span>';

  return `
    <button type="button"
            class="list-group-item list-group-item-action d-flex align-items-center ${disabled ? 'disabled' : ''}"
            ${disabled ? 'aria-disabled="true"' : onclick}>
      <img src="/teams/${e_prefix}${t.sname}.png" alt="${t.name}" class="team-logo me-2">
      <div class="flex-grow-1 text-start">
        <div class="fw-bold team-name">${t.name}</div>
        <div class="text-muted small">${slot.conf}</div>
      </div>
      ${note}
    </button>
  `;
}

function openSwap(slotId) {
  const slot = slots.find(s => s.id === slotId);
  swapSlotId = slotId;

  document.getElementById('swap-slot-label').textContent = slot.label;
  document.getElementById('swap-current-team').textContent = TEAMS[current[slotId]].name;

  const used = usedTeams(slotId);
  document.getElementById('swap-list').innerHTML = roleTeams(slot)
    .filter(t => t.sname !== current[slotId])   // omit the team already in this slot
    .map(t => swapRowHtml(slot, t, candidateAction(slot, t, used)))
    .join('');

  swapModal.show();
}

function chooseSwap(sname) {
  current[swapSlotId] = sname;
  swapSlotId = null;
  swapModal.hide();
  render();
}

// Atomic promote: the wildcard team moves into the winner slot, and the current
// winner takes the wildcard's slot. Counts as two changes (via the diff).
function atomicSwap(winnerSlotId, wildcardSlotId) {
  const winnerTeam = current[winnerSlotId];
  current[winnerSlotId] = current[wildcardSlotId];
  current[wildcardSlotId] = winnerTeam;
  swapSlotId = null;
  swapModal.hide();
  render();
}

// ---- Review phase ----
function reviewRowHtml(s) {
  const outTeam = TEAMS[original[s.id]];
  const inTeam = TEAMS[current[s.id]];
  return `
    <div class="card repick-card mb-2">
      <div class="card-body p-3 d-flex align-items-center flex-wrap">
        <span class="badge slot-label text-uppercase me-3">${s.label}</span>
        <span class="d-flex align-items-center text-muted me-3">
          <img src="/teams/${e_prefix}${original[s.id]}.png" alt="${outTeam.name}" class="team-logo-sm me-1">
          <span class="text-decoration-line-through">${outTeam.name}</span>
        </span>
        <i class="bi bi-arrow-right me-3"></i>
        <span class="d-flex align-items-center fw-bold">
          <img src="/teams/${e_prefix}${current[s.id]}.png" alt="${inTeam.name}" class="team-logo me-2">
          <span>${inTeam.name}</span>
        </span>
      </div>
    </div>
  `;
}

function renderReview() {
  document.getElementById('review-summary').innerHTML = slots
    .filter(s => current[s.id] !== original[s.id])
    .map(reviewRowHtml)
    .join('');
}

function goToReview() {
  document.getElementById('edit-view').style.display = 'none';
  document.getElementById('review-view').style.display = 'block';
  renderReview();
}

function backToEdit() {
  document.getElementById('review-view').style.display = 'none';
  document.getElementById('edit-view').style.display = 'block';
  render();
}

// ---- Submit ----
// Rebuild the four-bucket payload (snames) from the working copy so the server
// gets the full final 14 and diffs it against the stored picks.
function buildPayload() {
  const payload = { afcWinners: {}, nfcWinners: {}, afcWildcards: [], nfcWildcards: [] };
  slots.forEach(s => {
    if (s.kind === 'winner') {
      payload[s.conf === 'AFC' ? 'afcWinners' : 'nfcWinners'][s.division] = current[s.id];
    } else {
      payload[s.conf === 'AFC' ? 'afcWildcards' : 'nfcWildcards'][s.slot] = current[s.id];
    }
  });
  return payload;
}

function submitRepicks() {
  const payload = buildPayload();
  document.getElementById('afc-winners-input').value = JSON.stringify(payload.afcWinners);
  document.getElementById('nfc-winners-input').value = JSON.stringify(payload.nfcWinners);
  document.getElementById('afc-wildcards-input').value = JSON.stringify(payload.afcWildcards);
  document.getElementById('nfc-wildcards-input').value = JSON.stringify(payload.nfcWildcards);
  document.getElementById('repick-form').submit();
}

render();

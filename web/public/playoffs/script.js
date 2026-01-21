// Theme toggle functionality
function initTheme() {
  const savedTheme = localStorage.getItem('theme') || 'light';
  document.documentElement.setAttribute('data-bs-theme', savedTheme);
  updateThemeIcon(savedTheme);
}

function toggleTheme() {
  const currentTheme = document.documentElement.getAttribute('data-bs-theme');
  const newTheme = currentTheme === 'light' ? 'dark' : 'light';
  document.documentElement.setAttribute('data-bs-theme', newTheme);
  localStorage.setItem('theme', newTheme);
  updateThemeIcon(newTheme);
}

function updateThemeIcon(theme) {
  const icon = document.getElementById('theme-icon');
  if (icon) {
    icon.className = theme === 'light' ? 'bi bi-moon-fill' : 'bi bi-sun-fill';
  }
}

document.getElementById('theme-toggle')?.addEventListener('click', toggleTheme);
initTheme();

let phase = 'winners';
let afcWinners = {};
let nfcWinners = {};
let afcWildcards = [];
let nfcWildcards = [];

const divisions = [ 'North', 'South', 'East', 'West' ];

function toggleWinner(conference, division, teamName) {
  const winners = conference === 'AFC' ? afcWinners : nfcWinners;
  
  if (winners[division] === teamName) {
    delete winners[division];
  } else {
    winners[division] = teamName;
  }
  
  render();
}

function toggleWildcard(conference, teamName) {
  const wildcards = conference === 'AFC' ? afcWildcards : nfcWildcards;
  const idx = wildcards.indexOf(teamName);
  
  if (idx > -1) {
    wildcards.splice(idx, 1);
  } else if (wildcards.length < 3) {
    wildcards.push(teamName);
  }
  
  render();
}

function allWinnersSelected() {
  return divisions.every(d => afcWinners[d] && nfcWinners[d]);
}

function canSubmit() {
  return phase === 'wildcards' && afcWildcards.length === 3 && nfcWildcards.length === 3;
}

function proceedToWildcards() {
  phase = 'wildcards';
  render();
}

function backToWinners() {
  phase = 'winners';
  render();
}

function renderDivisionWinners(teams, division, conference) {
  const divisionTeams = teams.filter(t => t.division === division);
  const winners = conference === 'AFC' ? afcWinners : nfcWinners;
  const selectedWinner = winners[division];
  
  let html = `
    <div class="col-6 col-sm-6">
      <div class="card">
        <div class="card-body p-2 p-md-3">
          <div class="d-flex align-items-center mb-2">
            <i class="bi bi-trophy-fill text-warning me-2"></i>
            <h6 class="mb-0 fw-bold text-uppercase small">${division}</h6>
          </div>
          <div class="d-grid gap-2">
  `;
  
  divisionTeams.forEach(team => {
    const isSelected = selectedWinner === team.name;
    const btnClass = isSelected ? 'btn-winner-selected' : 'btn-outline-secondary';
    
    html += `
      <button class="btn ${btnClass} btn-sm text-start d-flex align-items-center"
              onclick="toggleWinner('${conference}', '${division}', '${team.name}')">
        <img src="/teams/${team.sname}.png" alt="${team.name}" class="team-logo me-2">
        <span class="flex-grow-1 team-name">${team.name}</span>
        ${isSelected ? '<i class="bi bi-check-lg"></i>' : ''}
      </button>
    `;
  });
  
  html += `
          </div>
        </div>
      </div>
    </div>
  `;
  
  return html;
}

function renderDivisionWildcards(teams, division, conference) {
  const divisionTeams = teams.filter(t => t.division === division);
  const winners = conference === 'AFC' ? afcWinners : nfcWinners;
  const wildcards = conference === 'AFC' ? afcWildcards : nfcWildcards;
  const divisionWinner = winners[division];
  
  let html = `
    <div class="col-6 col-sm-6">
      <div class="card">
        <div class="card-body p-2 p-md-3">
          <h6 class="mb-2 fw-bold text-uppercase small">${division}</h6>
          <div class="d-grid gap-2">
  `;
  
  divisionTeams.forEach(team => {
    const isWinner = team.name === divisionWinner;
    const isSelected = wildcards.includes(team.name);
    const isDisabled = isWinner || (!isSelected && wildcards.length >= 3);
    
    let btnClass = 'btn-outline-secondary';
    if (isWinner) btnClass = 'btn-winner-locked';
    else if (isSelected) btnClass = 'btn-wildcard-selected';
    
    const disabled = isDisabled ? 'disabled' : '';
    const icon = isWinner ? '<i class="bi bi-trophy-fill text-warning"></i>' : 
                 isSelected ? '<i class="bi bi-check-lg"></i>' : '';
    
    html += `
      <button class="btn ${btnClass} btn-sm text-start d-flex align-items-center" ${disabled}
              onclick="${!isWinner ? `toggleWildcard('${conference}', '${team.name}')` : ''}">
        <img src="/teams/${team.sname}.png" alt="${team.name}" class="team-logo me-2">
        <span class="flex-grow-1 team-name">${team.name}</span>
        ${icon}
      </button>
    `;
  });
  
  html += `
          </div>
        </div>
      </div>
    </div>
  `;
  
  return html;
}

function render() {
  const afcDiv = document.getElementById('afc-divisions');
  const nfcDiv = document.getElementById('nfc-divisions');
  const afcCounter = document.getElementById('afc-counter');
  const nfcCounter = document.getElementById('nfc-counter');
  const actionBtn = document.getElementById('action-btn');
  const backBtn = document.getElementById('back-btn');
  const phaseDesc = document.getElementById('phase-description');
  
  if (phase === 'winners') {
    phaseDesc.textContent = 'Step 1: Select division winners (1 per division)';
    
    afcDiv.innerHTML = divisions.map(d => renderDivisionWinners(afcTeams, d, 'AFC')).join('');
    nfcDiv.innerHTML = divisions.map(d => renderDivisionWinners(nfcTeams, d, 'NFC')).join('');
    
    const afcCount = Object.keys(afcWinners).length;
    const nfcCount = Object.keys(nfcWinners).length;
    
    afcCounter.textContent = `${afcCount} / 4`;
    nfcCounter.textContent = `${nfcCount} / 4`;
    afcCounter.className = afcCount === 4 ? 'badge bg-success' : 'badge bg-secondary';
    nfcCounter.className = nfcCount === 4 ? 'badge bg-success' : 'badge bg-secondary';
    
    actionBtn.textContent = 'Continue to Wild Cards';
    actionBtn.disabled = !allWinnersSelected();
    actionBtn.onclick = proceedToWildcards;
    backBtn.style.display = 'none';
  } else {
    phaseDesc.textContent = 'Step 2: Select wild card teams (3 per conference)';
    afcDiv.innerHTML = divisions.map(d => renderDivisionWildcards(afcTeams, d, 'AFC')).join('');
    nfcDiv.innerHTML = divisions.map(d => renderDivisionWildcards(nfcTeams, d, 'NFC')).join('');
    
    afcCounter.textContent = `${afcWildcards.length} / 3`;
    nfcCounter.textContent = `${nfcWildcards.length} / 3`;
    afcCounter.className = afcWildcards.length === 3 ? 'badge bg-success' : 'badge bg-secondary';
    nfcCounter.className = nfcWildcards.length === 3 ? 'badge bg-success' : 'badge bg-secondary';
    
    actionBtn.textContent = 'Submit Predictions';
    actionBtn.disabled = !canSubmit();
    actionBtn.onclick = submitForm;
    backBtn.style.display = 'inline-block';
    backBtn.onclick = backToWinners;
  }
}

function submitForm() {
  document.getElementById('afc-winners-input').value = JSON.stringify(afcWinners);
  document.getElementById('nfc-winners-input').value = JSON.stringify(nfcWinners);
  document.getElementById('afc-wildcards-input').value = JSON.stringify(afcWildcards);
  document.getElementById('nfc-wildcards-input').value = JSON.stringify(nfcWildcards);
  document.getElementById('playoff-form').submit();
}

render();

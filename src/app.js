const statusEl = document.getElementById('status');
const processTable = document.getElementById('processTable');
const historyTable = document.getElementById('historyTable');
const blacklistTable = document.getElementById('blacklistTable');
const searchInput = document.getElementById('searchInput');
const searchBtn = document.getElementById('searchBtn');
const clearSearchBtn = document.getElementById('clearSearchBtn');
const pythonBtn = document.getElementById('pythonBtn');
const refreshBtn = document.getElementById('refreshBtn');
const autoRefreshSelect = document.getElementById('autoRefresh');
const clearHistoryBtn = document.getElementById('clearHistoryBtn');
const exportCsvBtn = document.getElementById('exportCsvBtn');
const blacklistNameInput = document.getElementById('blacklistName');
const blacklistReasonInput = document.getElementById('blacklistReason');
const addBlacklistBtn = document.getElementById('addBlacklistBtn');
const autoKillToggle = document.getElementById('autoKillToggle');
const exportRulesBtn = document.getElementById('exportRulesBtn');
const importFile = document.getElementById('importFile');
const importRulesBtn = document.getElementById('importRulesBtn');
const fetchUrlInput = document.getElementById('fetchUrl');
const fetchRulesBtn = document.getElementById('fetchRulesBtn');
const detailPanel = document.getElementById('detailPanel');
const detailBody = document.getElementById('detailBody');
const closeDetailBtn = document.getElementById('closeDetailBtn');

let allProcesses = [];
let autoRefreshTimer = null;
let selectedPid = null;
let activeFilter = 'all'; // 'all', 'python', 'search'
let activeSearchQuery = '';

function setStatus(text, type = 'ready') {
    statusEl.textContent = text;
    statusEl.className = 'status ' + type;
}

async function invoke(command, args = {}) {
    setStatus('Working...', 'working');
    try {
        const result = await window.__TAURI__.invoke(command, args);
        setStatus('Ready', 'ready');
        return result;
    } catch (error) {
        setStatus('Error', 'error');
        console.error('Invoke error:', error);
        return null;
    }
}

// ========== TAB SWITCHING ==========

document.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
        tab.classList.add('active');
        document.getElementById('tab-' + tab.dataset.tab).classList.add('active');

        if (tab.dataset.tab === 'history') loadHistory();
        if (tab.dataset.tab === 'blacklist') loadBlacklist();
    });
});

// ========== PROCESS TAB ==========

function renderError(message) {
    processTable.innerHTML = `<tr><td colspan="6"><div class="empty-state"><div class="icon">ERROR</div><div>${escapeHtml(message)}</div></div></td></tr>`;
}

function renderEmpty() {
    processTable.innerHTML = `<tr><td colspan="6"><div class="empty-state"><div class="icon">NO DATA</div><div>No processes found. Try Refresh.</div></div></td></tr>`;
}

function renderProcesses(processes) {
    if (!processes || processes.length === 0) { renderEmpty(); return; }

    processTable.innerHTML = '';
    processes.forEach(p => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td class="pid">${p.pid}</td>
            <td>${escapeHtml(p.name)}</td>
            <td class="memory">${p.memory_mb.toFixed(1)} MB</td>
            <td class="cpu">-</td>
            <td class="command" title="${escapeHtml(p.cmdline)}">${escapeHtml(p.cmdline || '')}</td>
            <td><button class="kill-btn" data-pid="${p.pid}" data-name="${escapeHtml(p.name)}">Kill</button></td>
        `;
        row.style.cursor = 'pointer';
        row.addEventListener('click', (e) => {
            if (!e.target.classList.contains('kill-btn')) showDetail(p.pid);
        });
        processTable.appendChild(row);
    });

    processTable.querySelectorAll('.kill-btn').forEach(btn => {
        btn.addEventListener('click', async (e) => {
            e.stopPropagation();
            const pid = Number(btn.dataset.pid);
            const name = btn.dataset.name;
            if (confirm(`Kill process ${name} (PID ${pid})?`)) {
                const result = await invoke('kill_process', { pid });
                if (result) {
                    // Log to history
                    await invoke('db_add_kill_record', {
                        record: {
                            timestamp: new Date().toISOString(),
                            pid,
                            name,
                            memory_mb: 0,
                            method: 'manual',
                            command_line: '',
                            reason: ''
                        }
                    });
                    if (result.success) loadProcesses();
                }
            }
        });
    });

    document.getElementById('searchStats').textContent = `${processes.length} process${processes.length !== 1 ? 'es' : ''}`;
}

async function loadProcesses() {
    const processes = await invoke('list_processes', { limit: 200 });
    if (processes) {
        allProcesses = processes;
        // Re-apply active filter
        if (activeFilter === 'python') {
            const pythonProcesses = allProcesses.filter(p => p.name.to_lowercase().contains('python'));
            renderProcesses(pythonProcesses);
        } else if (activeFilter === 'search' && activeSearchQuery) {
            searchProcesses(activeSearchQuery);
        } else {
            renderProcesses(allProcesses);
        }
    }
}

function searchProcesses(query) {
    activeSearchQuery = query;
    activeFilter = query.trim() ? 'search' : 'all';
    if (!query.trim()) { renderProcesses(allProcesses); return; }
    const q = query.toLowerCase();
    const filtered = allProcesses.filter(p =>
        p.name.toLowerCase().includes(q) ||
        String(p.pid).includes(q) ||
        (p.cmdline && p.cmdline.toLowerCase().includes(q))
    );
    renderProcesses(filtered);
    document.getElementById('searchStats').textContent = `${filtered.length} of ${allProcesses.length}`;
}

async function showDetail(pid) {
    const detail = await invoke('process_detail', { pid });
    if (!detail) return;

    detailBody.innerHTML = `
        <div class="detail-field"><label>PID</label><div class="value">${detail.pid}</div></div>
        <div class="detail-field"><label>Name</label><div class="value">${escapeHtml(detail.name)}</div></div>
        <div class="detail-field"><label>Parent PID</label><div class="value">${detail.parent_pid || '-'}</div></div>
        <div class="detail-field"><label>CPU%</label><div class="value">${detail.cpu_percent.toFixed(1)}%</div></div>
        <div class="detail-field"><label>Memory</label><div class="value">${detail.memory_mb.toFixed(1)} MB</div></div>
        <div class="detail-field"><label>Command Line</label><div class="value copyable" title="Click to copy">${escapeHtml(detail.cmdline)}</div></div>
        <div class="detail-field"><label>Executable Path</label><div class="value copyable" title="Click to copy">${escapeHtml(detail.exe_path)}</div></div>
        <div class="detail-field"><label>Status</label><div class="value">${escapeHtml(detail.status)}</div></div>
    `;

    // Add copy listeners
    detailBody.querySelectorAll('.copyable').forEach(el => {
        el.addEventListener('click', () => {
            navigator.clipboard.writeText(el.textContent);
        });
    });

    // If Python process, show Python info
    if (detail.name.toLowerCase().includes('python')) {
        const pyInfo = await invoke('inspect_python', { pid });
        if (pyInfo) {
            detailBody.innerHTML += `
                <div class="detail-field"><label>Script Path</label><div class="value copyable">${escapeHtml(pyInfo.script_path)}</div></div>
                <div class="detail-field"><label>Working Dir</label><div class="value copyable">${escapeHtml(pyInfo.working_dir)}</div></div>
                <div class="detail-field"><label>Venv</label><div class="value">${pyInfo.is_venv ? escapeHtml(pyInfo.venv_path) : 'None detected'}</div></div>
                <div class="detail-field"><label>Python Version</label><div class="value">${escapeHtml(pyInfo.python_version || 'Unknown')}</div></div>
            `;
        }
    }

    // Kill and open buttons
    detailBody.innerHTML += `
        <div style="margin-top: 16px; display: flex; gap: 8px;">
            <button class="btn danger" id="detailKillBtn">Kill Process</button>
            <button class="btn" id="detailOpenBtn">Open Directory</button>
        </div>
    `;

    document.getElementById('detailKillBtn').addEventListener('click', async () => {
        if (confirm(`Kill ${detail.name} (PID ${detail.pid})?`)) {
            await invoke('kill_process', { pid });
            detailPanel.classList.remove('open');
            loadProcesses();
        }
    });

    document.getElementById('detailOpenBtn').addEventListener('click', async () => {
        const dir = detail.exe_path.substring(0, detail.exe_path.lastIndexOf('\\'));
        await invoke('open_terminal', { path: dir });
    });

    detailPanel.classList.add('open');
}

closeDetailBtn.addEventListener('click', () => detailPanel.classList.remove('open'));

searchBtn.addEventListener('click', () => searchProcesses(searchInput.value));
searchInput.addEventListener('keypress', (e) => { if (e.key === 'Enter') searchProcesses(searchInput.value); });
clearSearchBtn.addEventListener('click', () => { searchInput.value = ''; activeSearchQuery = ''; activeFilter = 'all'; renderProcesses(allProcesses); });
pythonBtn.addEventListener('click', async () => {
    activeFilter = 'python';
    const processes = await invoke('find_python_processes');
    if (processes) { allProcesses = processes; renderProcesses(allProcesses); }
});
refreshBtn.addEventListener('click', loadProcesses);

autoRefreshSelect.addEventListener('change', (e) => {
    if (autoRefreshTimer) clearInterval(autoRefreshTimer);
    const seconds = parseInt(e.target.value);
    if (seconds > 0) {
        autoRefreshTimer = setInterval(loadProcesses, seconds * 1000);
    }
});

// ========== HISTORY TAB ==========

function renderHistory(records) {
    if (!records || records.length === 0) {
        historyTable.innerHTML = `<tr><td colspan="6"><div class="empty-state"><div class="icon">NO HISTORY</div><div>No kill history yet.</div></div></td></tr>`;
        document.getElementById('historyStats').textContent = '';
        return;
    }

    historyTable.innerHTML = '';
    records.forEach(r => {
        const row = document.createElement('tr');
        const result = r.success !== undefined ? (r.success ? 'Success' : 'Failed') : (r.reason ? 'Success' : 'Unknown');
        row.innerHTML = `
            <td>${escapeHtml(r.timestamp)}</td>
            <td class="pid">${r.pid}</td>
            <td>${escapeHtml(r.name)}</td>
            <td class="memory">${r.memory_mb.toFixed(1)} MB</td>
            <td>${escapeHtml(r.method)}</td>
            <td>${result}</td>
        `;
        historyTable.appendChild(row);
    });

    document.getElementById('historyStats').textContent = `${records.length} record${records.length !== 1 ? 's' : ''}`;
}

async function loadHistory() {
    const records = await invoke('db_list_kill_records', { limit: 200 });
    if (records) renderHistory(records);
}

clearHistoryBtn.addEventListener('click', async () => {
    if (confirm('Clear all history?')) {
        await invoke('db_clear_kill_records');
        loadHistory();
    }
});

exportCsvBtn.addEventListener('click', async () => {
    const records = await invoke('db_export_kill_records');
    if (!records || records.length === 0) { alert('No records to export'); return; }

    const header = 'Timestamp,PID,Name,Memory_MB,Method,Command_Line,Reason\n';
    const rows = records.map(r =>
        `"${r.timestamp}",${r.pid},"${r.name}",${r.memory_mb.toFixed(1)},"${r.method}","${(r.command_line || '').replace(/"/g, '""')}","${r.reason}"`
    ).join('\n');

    const blob = new Blob([header + rows], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `python-hunter-history-${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
});

// ========== BLACKLIST TAB ==========

function renderBlacklist(entries) {
    if (!entries || entries.length === 0) {
        blacklistTable.innerHTML = `<tr><td colspan="4"><div class="empty-state"><div class="icon">NO ENTRIES</div><div>No blacklisted processes.</div></div></td></tr>`;
        return;
    }

    blacklistTable.innerHTML = '';
    entries.forEach(e => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td>${escapeHtml(e.name)}</td>
            <td>${escapeHtml(e.reason || '')}</td>
            <td>-</td>
            <td><button class="kill-btn remove-blacklist" data-name="${escapeHtml(e.name)}">Remove</button></td>
        `;
        blacklistTable.appendChild(row);
    });

    blacklistTable.querySelectorAll('.remove-blacklist').forEach(btn => {
        btn.addEventListener('click', async () => {
            await invoke('db_remove_blacklist_entry', { name: btn.dataset.name });
            loadBlacklist();
        });
    });
}

async function loadBlacklist() {
    const entries = await invoke('db_list_blacklist');
    if (entries) renderBlacklist(entries);
}

addBlacklistBtn.addEventListener('click', async () => {
    const name = blacklistNameInput.value.trim();
    const reason = blacklistReasonInput.value.trim();
    if (!name) { alert('Enter a process name'); return; }

    await invoke('db_add_blacklist_entry', { entry: { name, reason: reason || null } });
    blacklistNameInput.value = '';
    blacklistReasonInput.value = '';
    loadBlacklist();
});

// ========== RULES TAB ==========

exportRulesBtn.addEventListener('click', async () => {
    const json = await invoke('export_rules');
    if (!json) { alert('Export failed'); return; }

    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `python-hunter-rules-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
});

importRulesBtn.addEventListener('click', async () => {
    const file = importFile.files[0];
    if (!file) { alert('Select a JSON file'); return; }

    const text = await file.text();
    const result = await invoke('import_rules', { json: text });
    if (result !== null) {
        alert('Rules imported successfully');
        loadBlacklist();
    } else {
        alert('Import failed');
    }
});

fetchRulesBtn.addEventListener('click', async () => {
    const url = fetchUrlInput.value.trim();
    if (!url) { alert('Enter a URL'); return; }

    const json = await invoke('fetch_rules', { url });
    if (json) {
        await invoke('import_rules', { json });
        alert('Rules fetched and imported');
        loadBlacklist();
    } else {
        alert('Fetch failed');
    }
});

// ========== UTILITY ==========

function escapeHtml(text) {
    return String(text || '')
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

// ========== INIT ==========

loadProcesses();
// Start auto-refresh at 5s
autoRefreshTimer = setInterval(loadProcesses, 5000);

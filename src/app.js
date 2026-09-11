const statusEl = document.getElementById('status');
const processTable = document.getElementById('processTable');
const searchInput = document.getElementById('searchInput');
const searchBtn = document.getElementById('searchBtn');
const clearSearchBtn = document.getElementById('clearSearchBtn');
const searchStats = document.getElementById('searchStats');
const refreshBtn = document.getElementById('refreshBtn');
const pythonBtn = document.getElementById('pythonBtn');
const killSelectedBtn = document.getElementById('killSelectedBtn');

let allProcesses = [];
let selectedPid = null;

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
        renderError(error.message || String(error));
        return null;
    }
}

function renderError(message) {
    processTable.innerHTML = `
        <tr>
            <td colspan="5">
                <div class="empty-state">
                    <div class="icon">ERROR</div>
                    <div>${escapeHtml(message)}</div>
                </div>
            </td>
        </tr>
    `;
}

function renderEmpty() {
    processTable.innerHTML = `
        <tr>
            <td colspan="5">
                <div class="empty-state">
                    <div class="icon">NO DATA</div>
                    <div>No processes found. Try Refresh.</div>
                </div>
            </td>
        </tr>
    `;
}

function renderProcesses(processes) {
    if (!processes || processes.length === 0) {
        renderEmpty();
        searchStats.textContent = '';
        return;
    }

    processTable.innerHTML = '';
    processes.forEach((p) => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td class="pid">${p.pid}</td>
            <td>${escapeHtml(p.name)}</td>
            <td class="memory">${p.memory_mb.toFixed(1)} MB</td>
            <td class="command" title="${escapeHtml(p.cmdline)}">${escapeHtml(p.cmdline || '')}</td>
            <td>
                <button class="kill-btn" data-pid="${p.pid}" data-name="${escapeHtml(p.name)}">Kill</button>
            </td>
        `;
        processTable.appendChild(row);
    });

    processTable.querySelectorAll('.kill-btn').forEach((btn) => {
        btn.addEventListener('click', async () => {
            const pid = Number(btn.dataset.pid);
            const name = btn.dataset.name;
            if (confirm(`Kill process ${name} (PID ${pid})?`)) {
                const result = await invoke('kill_process', { pid });
                if (result && result.success) {
                    addHistory(result.message);
                    loadProcesses();
                } else if (result) {
                    addHistory(`Failed: ${result.message}`);
                }
            }
        });
    });

    searchStats.textContent = `${processes.length} process${processes.length !== 1 ? 'es' : ''}`;
}

function addHistory(message) {
    console.log('[History]', message);
}

async function loadProcesses() {
    const processes = await invoke('list_processes', { limit: 200 });
    if (processes) {
        allProcesses = processes;
        renderProcesses(allProcesses);
    }
}

function searchProcesses(query) {
    if (!query.trim()) {
        renderProcesses(allProcesses);
        return;
    }
    const q = query.toLowerCase();
    const filtered = allProcesses.filter(p =>
        p.name.toLowerCase().includes(q) ||
        String(p.pid).includes(q) ||
        (p.cmdline && p.cmdline.toLowerCase().includes(q))
    );
    renderProcesses(filtered);
    searchStats.textContent = `${filtered.length} of ${allProcesses.length} process${allProcesses.length !== 1 ? 'es' : ''}`;
}

refreshBtn.addEventListener('click', loadProcesses);

pythonBtn.addEventListener('click', async () => {
    const processes = await invoke('find_python_processes');
    if (processes) {
        allProcesses = processes;
        renderProcesses(allProcesses);
    }
});

searchBtn.addEventListener('click', () => {
    searchProcesses(searchInput.value);
});

searchInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
        searchProcesses(searchInput.value);
    }
});

clearSearchBtn.addEventListener('click', () => {
    searchInput.value = '';
    renderProcesses(allProcesses);
    searchStats.textContent = `${allProcesses.length} process${allProcesses.length !== 1 ? 'es' : ''}`;
});

killSelectedBtn.addEventListener('click', async () => {
    if (!selectedPid) {
        alert('Select a process first');
        return;
    }
    const proc = allProcesses.find(p => p.pid === selectedPid);
    if (proc && confirm(`Kill ${proc.name} (PID ${proc.pid})?`)) {
        const result = await invoke('kill_process', { pid: selectedPid });
        if (result && result.success) {
            addHistory(result.message);
            loadProcesses();
        }
    }
});

function escapeHtml(text) {
    return String(text || '')
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

loadProcesses();

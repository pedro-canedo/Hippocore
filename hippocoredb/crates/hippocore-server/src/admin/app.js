const root = document.getElementById('app');

const NAV = [
  ['operate', 'Dashboard', 'dashboard'],
  ['operate', 'Data Explorer', 'data-explorer'],
  ['operate', 'SQL Editor', 'sql-editor'],
  ['data', 'Collections', 'collections'],
  ['data', 'Records', 'records'],
  ['data', 'Memories', 'memories'],
  ['data', 'Documents', 'documents'],
  ['data', 'Files', 'files'],
  ['context', 'Ingestion & Recall', 'ingestion-recall'],
  ['context', 'Graph', 'graph'],
  ['admin', 'API Reference', 'api-reference'],
  ['admin', 'Tenants', 'tenants'],
  ['admin', 'Service Keys', 'service-keys'],
  ['admin', 'Observability', 'observability'],
  ['admin', 'Settings', 'settings'],
];

const GROUP_LABELS = {
  operate: { en: 'Workspace', pt: 'Workspace' },
  data: { en: 'Data', pt: 'Dados' },
  context: { en: 'Context', pt: 'Contexto' },
  admin: { en: 'Admin', pt: 'Admin' },
};

const I18N = {
  en: {
    active: 'active',
    adminPassword: 'admin password',
    adminUser: 'admin user',
    apiReferenceLead: 'HTTP surface grouped by domain. Implemented endpoints are callable today; planned entries describe the product direction.',
    appSubtitle: 'Local-first memory and context database',
    appTitle: 'Hippocore Control Plane',
    buildContext: 'Build context',
    collection: 'Collection',
    collectionDescription: 'Collection description',
    collectionHelp: 'Collections are logical namespaces inside a tenant. Records, memories, documents, and file-derived context live under them.',
    copied: 'Copied',
    createCollection: 'Create collection',
    createTenant: 'Create tenant',
    currentTenant: 'Current tenant',
    dataDir: 'Data directory',
    dataExplorerLead: 'Browse logical records and physical context objects without mixing storage internals into the main workflow.',
    dashboardLead: 'Overview of the active local database, tenant scope, object counts, and next actions.',
    documents: 'Documents',
    empty: 'Nothing to show yet.',
    error: 'Error',
    files: 'Files',
    graph: 'Graph',
    graphLead: 'Inspect graph edges between context items. Multi-hop traversal is available through the service API.',
    health: 'Health',
    ingestLead: 'Store context and immediately test recall/build_context in the selected tenant.',
    language: 'Language',
    lastOperation: 'Last operation',
    login: 'Log in',
    loginLead: 'Use the admin credentials configured for this local server.',
    logout: 'Log out',
    memories: 'Memories',
    memoryText: 'Memory text',
    name: 'Name',
    noTenant: 'No tenant selected',
    objectCounts: 'Object counts',
    observabilityLead: 'Operational view over health, stats, storage, WAL, snapshot, audit, and index status.',
    openDataExplorer: 'Open Data Explorer',
    openSql: 'Open SQL Editor',
    planned: 'Planned',
    providerLead: 'Manage local LLM provider registry and rotate the service API key used by automation.',
    refresh: 'Refresh',
    records: 'Records',
    recall: 'Run recall',
    recallQuery: 'Recall query',
    rotateKey: 'Rotate service key',
    run: 'Run',
    save: 'Save',
    serverStatus: 'Server status',
    session: 'Session',
    signedOut: 'signed out',
    sqlExamples: 'Examples',
    sqlLead: 'Run the supported read-only SQL-like record query layer with tenant isolation enforced by the backend.',
    storeDocument: 'Store document',
    storeMemory: 'Store memory',
    storeRecord: 'Store record',
    storeType: 'Type',
    table: 'Table',
    tenant: 'Tenant',
    tenantId: 'Tenant id',
    tenants: 'Tenants',
    title: 'Title',
  },
  pt: {
    active: 'ativa',
    adminPassword: 'senha admin',
    adminUser: 'usuario admin',
    apiReferenceLead: 'Superficie HTTP agrupada por dominio. Endpoints Implemented existem hoje; Planned indica direcao do produto.',
    appSubtitle: 'Banco local-first de memoria e contexto',
    appTitle: 'Hippocore Control Plane',
    buildContext: 'Montar contexto',
    collection: 'Collection',
    collectionDescription: 'Descricao da collection',
    collectionHelp: 'Collections sao namespaces logicos dentro de um tenant. Records, memorias, documentos e contexto de arquivos vivem nelas.',
    copied: 'Copiado',
    createCollection: 'Criar collection',
    createTenant: 'Criar tenant',
    currentTenant: 'Tenant atual',
    dataDir: 'Diretorio de dados',
    dataExplorerLead: 'Navegue dados logicos e objetos fisicos de contexto sem misturar detalhes internos no fluxo principal.',
    dashboardLead: 'Visao geral do banco local ativo, escopo do tenant, contagens de objetos e proximas acoes.',
    documents: 'Documentos',
    empty: 'Nada para mostrar ainda.',
    error: 'Erro',
    files: 'Arquivos',
    graph: 'Grafo',
    graphLead: 'Inspecione arestas entre itens de contexto. Travessia multi-hop esta disponivel pela API de servico.',
    health: 'Health',
    ingestLead: 'Armazene contexto e teste recall/build_context no tenant selecionado.',
    language: 'Idioma',
    lastOperation: 'Ultima operacao',
    login: 'Entrar',
    loginLead: 'Use as credenciais admin configuradas para este servidor local.',
    logout: 'Sair',
    memories: 'Memorias',
    memoryText: 'Texto da memoria',
    name: 'Nome',
    noTenant: 'Nenhum tenant selecionado',
    objectCounts: 'Contagens',
    observabilityLead: 'Visao operacional de health, stats, storage, WAL, snapshot, audit e status de indice.',
    openDataExplorer: 'Abrir Data Explorer',
    openSql: 'Abrir SQL Editor',
    planned: 'Planned',
    providerLead: 'Gerencie providers LLM locais e rotacione a API key usada por automacoes.',
    refresh: 'Atualizar',
    records: 'Records',
    recall: 'Executar recall',
    recallQuery: 'Consulta de recall',
    rotateKey: 'Rotacionar service key',
    run: 'Executar',
    save: 'Salvar',
    serverStatus: 'Status do servidor',
    session: 'Sessao',
    signedOut: 'desconectada',
    sqlExamples: 'Exemplos',
    sqlLead: 'Execute a camada read-only de query estilo SQL com isolamento de tenant aplicado pelo backend.',
    storeDocument: 'Salvar documento',
    storeMemory: 'Salvar memoria',
    storeRecord: 'Salvar record',
    storeType: 'Tipo',
    table: 'Tabela',
    tenant: 'Tenant',
    tenantId: 'Tenant id',
    tenants: 'Tenants',
    title: 'Titulo',
  },
};

const state = {
  session: localStorage.getItem('hippocore.adminSession') || '',
  username: localStorage.getItem('hippocore.adminUser') || 'admin',
  lang: localStorage.getItem('hippocore.lang') || 'en',
  page: localStorage.getItem('hippocore.page') || 'dashboard',
  tenant: localStorage.getItem('hippocore.tenant') || '',
  collection: localStorage.getItem('hippocore.collection') || '',
  tab: 'logical',
  sqlTab: 'table',
  bootstrap: null,
  collections: [],
  lists: {},
  providers: [],
  providerResult: null,
  sqlResult: null,
  sqlError: '',
  detail: null,
  toast: '',
  lastOperation: 'No operation in this session.',
};

function t(key) {
  return (I18N[state.lang] && I18N[state.lang][key]) || I18N.en[key] || key;
}

function esc(value) {
  return String(value ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

function json(value) {
  return esc(JSON.stringify(value ?? null, null, 2));
}

function persist() {
  localStorage.setItem('hippocore.adminSession', state.session);
  localStorage.setItem('hippocore.adminUser', state.username);
  localStorage.setItem('hippocore.lang', state.lang);
  localStorage.setItem('hippocore.page', state.page);
  localStorage.setItem('hippocore.tenant', state.tenant);
  localStorage.setItem('hippocore.collection', state.collection);
}

async function request(path, options = {}) {
  const headers = Object.assign(
    { 'content-type': 'application/json', 'x-admin-session': state.session },
    options.headers || {},
  );
  const res = await fetch(path, Object.assign({}, options, { headers }));
  const text = await res.text();
  let body = null;
  if (text) {
    try {
      body = JSON.parse(text);
    } catch (_) {
      body = text;
    }
  }
  if (!res.ok) {
    throw new Error((body && body.error) || text || `HTTP ${res.status}`);
  }
  return body;
}

async function publicRequest(path, options = {}) {
  const res = await fetch(path, options);
  const text = await res.text();
  const body = text ? JSON.parse(text) : null;
  if (!res.ok) throw new Error((body && body.error) || text || `HTTP ${res.status}`);
  return body;
}

function route(page, extra = {}) {
  Object.assign(state, extra, { page });
  persist();
  render();
  if (state.session) hydratePage().catch(showError);
}

function showToast(message) {
  state.toast = message;
  render();
  setTimeout(() => {
    state.toast = '';
    render();
  }, 2400);
}

function showError(err) {
  state.lastOperation = `${t('error')}: ${err.message || err}`;
  showToast(state.lastOperation);
}

// Components -----------------------------------------------------------------

function StatusBadge(label, tone = 'info') {
  return `<span class="status-badge ${tone}">${esc(label)}</span>`;
}

function ActionButton(label, action, tone = '') {
  return `<button class="${tone}" data-action="${esc(action)}" type="button">${esc(label)}</button>`;
}

function StatCard(label, value, note = '') {
  return `<div class="stat"><span>${esc(label)}</span><strong>${esc(value)}</strong>${note ? `<span>${esc(note)}</span>` : ''}</div>`;
}

function EmptyState(title, message, action = '') {
  return `<div class="empty-state"><h3>${esc(title)}</h3><p>${esc(message)}</p>${action}</div>`;
}

function JsonViewer(value) {
  return `<pre>${json(value)}</pre>`;
}

function CodeBlock(label, code, action = '') {
  return `<div class="code-block"><div class="copy-row"><strong>${label}</strong>${action}</div><pre>${esc(code)}</pre></div>`;
}

function FormField(label, id, value = '', type = 'text', attrs = '') {
  return `<label class="field"><span>${esc(label)}</span><input id="${esc(id)}" type="${esc(type)}" value="${esc(value)}" ${attrs} /></label>`;
}

function TextAreaField(label, id, value = '', attrs = '') {
  return `<label class="field wide"><span>${esc(label)}</span><textarea id="${esc(id)}" ${attrs}>${esc(value)}</textarea></label>`;
}

function Tabs(tabs, active, action) {
  return `<div class="tabs">${tabs.map((tab) => `<button type="button" data-action="${action}" data-tab="${esc(tab.id)}" aria-selected="${tab.id === active}">${esc(tab.label)}</button>`).join('')}</div>`;
}

function DataTable(rows, columns, emptyTitle = t('empty')) {
  if (!rows || rows.length === 0) {
    return EmptyState(emptyTitle, 'Create or select data to populate this view.');
  }
  return `<div class="table-wrap"><table><thead><tr>${columns.map((c) => `<th>${esc(c.label)}</th>`).join('')}<th></th></tr></thead><tbody>${rows.map((row, idx) => `<tr>${columns.map((c) => `<td>${esc(c.render ? c.render(row) : row[c.key])}</td>`).join('')}<td><button class="ghost" type="button" data-action="detail" data-list="${esc(row.__list || '')}" data-index="${idx}">Open</button></td></tr>`).join('')}</tbody></table></div>`;
}

function PageHeader(title, lead, actions = '') {
  return `<div class="page-header"><div><h2>${esc(title)}</h2><p>${esc(lead)}</p></div><div class="page-actions">${actions}</div></div>`;
}

function DetailDrawer() {
  if (!state.detail) return '';
  return `<div class="drawer-backdrop" data-action="close-detail"></div><aside class="drawer"><div class="drawer-head"><h3>${esc(state.detail.title)}</h3><button class="icon" data-action="close-detail" type="button">x</button></div><div class="drawer-body">${JsonViewer(state.detail.value)}</div></aside>`;
}

function Sidebar() {
  let currentGroup = '';
  const nav = NAV.map(([group, label, id]) => {
    const groupLabel = group !== currentGroup ? `<div class="group-label">${esc(GROUP_LABELS[group][state.lang] || GROUP_LABELS[group].en)}</div>` : '';
    currentGroup = group;
    return `${groupLabel}<button type="button" data-route="${esc(id)}" aria-current="${state.page === id ? 'page' : 'false'}">${esc(label)}</button>`;
  }).join('');
  return `<aside class="sidebar"><div class="brand"><div class="mark" aria-hidden="true"></div><div><h1>${esc(t('appTitle'))}</h1><p>${esc(t('appSubtitle'))}</p></div></div><nav class="nav">${nav}</nav><div class="sidebar-footer">${StatusBadge(state.session ? t('active') : t('signedOut'), state.session ? 'ok' : 'planned')}<span class="muted">${esc(state.bootstrap?.config?.data_dir || './hippocore-data')}</span></div></aside>`;
}

function Topbar() {
  const tenants = state.bootstrap?.tenants || [];
  const tenantOptions = [`<option value="">${esc(t('noTenant'))}</option>`].concat(
    tenants.map((tenant) => `<option value="${esc(tenant.id)}" ${tenant.id === state.tenant ? 'selected' : ''}>${esc(tenant.id)} - ${esc(tenant.name)}</option>`),
  ).join('');
  return `<header class="topbar"><div><div class="breadcrumbs">Control Plane / ${esc(pageTitle())}</div><div class="kpi-line">${StatusBadge(t('currentTenant') + ': ' + (state.tenant || '-'), state.tenant ? 'ok' : 'planned')}${StatusBadge(t('session') + ': ' + (state.session ? t('active') : t('signedOut')), state.session ? 'ok' : 'planned')}</div></div><div class="topbar-actions"><select class="tenant-select" id="globalTenant">${tenantOptions}</select><select id="languageSelect" aria-label="${esc(t('language'))}"><option value="en" ${state.lang === 'en' ? 'selected' : ''}>EN</option><option value="pt" ${state.lang === 'pt' ? 'selected' : ''}>PT</option></select><button class="icon" type="button" data-action="refresh" title="${esc(t('refresh'))}">R</button><button type="button" data-action="logout">${esc(t('logout'))}</button></div></header>`;
}

function AppShell(content) {
  return `<div class="app-shell">${Sidebar()}<div class="main">${Topbar()}<main class="content">${content}</main></div>${DetailDrawer()}${state.toast ? `<div class="toast">${esc(state.toast)}</div>` : ''}</div>`;
}

// Pages ----------------------------------------------------------------------

function pageTitle() {
  const item = NAV.find(([, , id]) => id === state.page);
  return item ? item[1] : 'Dashboard';
}

function stats() {
  return state.bootstrap?.stats || {};
}

function DashboardPage() {
  const s = stats();
  const quick = [
    ['Create tenant', 'tenants'],
    ['Create collection', 'collections'],
    [t('openSql'), 'sql-editor'],
    [t('openDataExplorer'), 'data-explorer'],
    ['Ingest memory/document', 'ingestion-recall'],
    [t('rotateKey'), 'service-keys'],
  ].map(([label, target]) => `<button type="button" data-route="${esc(target)}"><span>${esc(label)}</span><span>-></span></button>`).join('');
  return `${PageHeader('Dashboard', t('dashboardLead'), ActionButton(t('refresh'), 'refresh'))}<section class="grid cols-4">${StatCard(t('serverStatus'), state.session ? 'Online' : 'Signed out', 'GET /health')}${StatCard(t('currentTenant'), state.tenant || '-', 'Tenant isolation scope')}${StatCard('Tenants', s.tenants ?? '-')}${StatCard('Collections', s.collections ?? '-')}${StatCard('Records', s.records ?? '-')}${StatCard('Memories', s.memories ?? '-')}${StatCard('Documents', s.documents ?? '-')}${StatCard('Files', s.files ?? '-')}${StatCard('Graph edges', s.graph_edges ?? '-')}${StatCard('Audit log', s.audit_log_bytes != null ? `${s.audit_log_bytes} B` : '-', `${s.audit_records ?? 0} records`)}${StatCard(t('dataDir'), state.bootstrap?.config?.data_dir || '-', 'local-first')}${StatCard(t('lastOperation'), state.lastOperation, '')}</section><section class="grid cols-2"><div class="panel"><h3>Quick actions</h3><div class="actions-list">${quick}</div></div><div class="panel"><h3>${esc(t('objectCounts'))}</h3>${JsonViewer(s)}</div></section>`;
}

function DataExplorerPage() {
  const collections = state.collections || [];
  const rows = annotateRows(activeExplorerRows(), activeExplorerListName());
  const tabs = Tabs([
    { id: 'logical', label: 'Logical View' },
    { id: 'json', label: 'Raw JSON' },
    { id: 'physical', label: 'Physical Info' },
  ], state.tab, 'tab');
  const main = state.tab === 'json'
    ? JsonViewer(rows)
    : state.tab === 'physical'
      ? PhysicalInfo(rows)
      : DataTable(rows, logicalColumns(rows), 'No logical data selected');
  return `${PageHeader('Data Explorer', t('dataExplorerLead'), ActionButton(t('refresh'), 'refresh'))}<div class="layout-split"><aside class="side-panel"><strong>${esc(t('tenants'))}</strong><div class="tree">${tenantTree(collections)}</div><div class="notice">Collection is a logical namespace. Record is JSON-first structured data projected into context.</div></aside><section class="panel">${tabs}${main}</section></div>`;
}

function tenantTree(collections) {
  const tenants = state.bootstrap?.tenants || [];
  if (tenants.length === 0) return EmptyState('No tenants', 'Create a tenant to start browsing data.');
  return tenants.map((tenant) => {
    const nested = tenant.id === state.tenant
      ? `<div class="nested">${collections.map((col) => `<button type="button" data-action="choose-collection" data-collection="${esc(col.name)}">${esc(col.name)}</button>`).join('') || '<span class="muted">No collections</span>'}</div>`
      : '';
    return `<div><button type="button" data-action="choose-tenant" data-tenant="${esc(tenant.id)}">${esc(tenant.id)} <span class="muted">${esc(tenant.name)}</span></button>${nested}</div>`;
  }).join('');
}

function activeExplorerListName() {
  if (state.page === 'records') return 'records';
  if (state.page === 'memories') return 'memories';
  if (state.page === 'documents') return 'documents';
  if (state.page === 'files') return 'files';
  return 'records';
}

function activeExplorerRows() {
  return state.lists[activeExplorerListName()] || [];
}

function annotateRows(rows, list) {
  return (rows || []).map((row) => Object.assign({ __list: list }, row));
}

function logicalColumns(rows) {
  if (!rows || rows.length === 0) return [];
  const keys = ['id', 'collection', 'table', 'name', 'memory_type', 'media_type', 'version']
    .filter((key) => rows.some((row) => row[key] != null));
  return keys.map((key) => ({ key, label: key }));
}

function PhysicalInfo(rows) {
  const payload = {
    layer: 'physical/internal',
    selected_tenant: state.tenant || null,
    selected_collection: state.collection || null,
    visible_rows: rows.length,
    stats: stats(),
    notes: ['WAL, snapshot, chunks, audit, and index status are operational objects, not logical tables.'],
  };
  return JsonViewer(payload);
}

function SqlEditorPage() {
  const examples = [
    "select * from systems limit 10",
    "select * from systems where engine = 'postgresql' limit 5",
    "select * from records where payload.engine = 'postgresql'",
  ];
  const result = state.sqlError
    ? `<pre class="result-error">${esc(state.sqlError)}</pre>`
    : state.sqlTab === 'json'
      ? JsonViewer(state.sqlResult)
      : DataTable(annotateRows(state.sqlResult?.rows || [], 'sql'), logicalColumns(state.sqlResult?.rows || []), 'No SQL results');
  return `${PageHeader('SQL Editor', t('sqlLead'), ActionButton(t('run'), 'run-sql', 'primary'))}<section class="sql-editor"><div class="panel"><label class="field"><span>SQL</span><textarea id="sqlInput">select * from systems limit 10</textarea></label><div class="page-actions">${ActionButton(t('run'), 'run-sql', 'primary')}${StatusBadge(state.tenant ? `tenant: ${state.tenant}` : t('noTenant'), state.tenant ? 'ok' : 'planned')}</div><div class="notice warning">Only SELECT * FROM table with equality filters and LIMIT is implemented.</div></div><aside class="panel"><h3>${esc(t('sqlExamples'))}</h3><div class="actions-list">${examples.map((ex) => `<button type="button" data-action="use-sql" data-sql="${esc(ex)}">${esc(ex)}</button>`).join('')}</div></aside></section><section class="panel">${Tabs([{ id: 'table', label: 'Table result' }, { id: 'json', label: 'JSON result' }], state.sqlTab, 'sql-tab')}${result}</section>`;
}

function CollectionsPage() {
  const rows = annotateRows(state.collections || [], 'collections');
  return `${PageHeader('Collections', t('collectionHelp'), ActionButton(t('createCollection'), 'create-collection', 'primary'))}<section class="grid cols-2"><div class="panel">${DataTable(rows, [{ key: 'name', label: 'name' }, { key: 'description', label: 'description' }, { key: 'tenant_id', label: 'tenant' }], 'No collections')}</div><div class="panel"><h3>${esc(t('createCollection'))}</h3><div class="form-grid">${FormField(t('collection'), 'collectionName', state.collection)}${FormField(t('collectionDescription'), 'collectionDescription')}</div><div class="page-actions">${ActionButton(t('save'), 'create-collection', 'primary')}</div></div></section>`;
}

function ObjectListPage(kind) {
  const rows = annotateRows(state.lists[kind] || [], kind);
  const labels = { records: t('records'), memories: t('memories'), documents: t('documents'), files: t('files') };
  return `${PageHeader(labels[kind], `Tenant-scoped ${labels[kind].toLowerCase()} list.`, ActionButton(t('refresh'), 'refresh'))}<section class="panel">${DataTable(rows, logicalColumns(rows), `No ${labels[kind].toLowerCase()}`)}</section>`;
}

function IngestionRecallPage() {
  const kindStatus = { Memory: 'Implemented', Document: 'Implemented', Record: 'Implemented', File: 'Planned' };
  return `${PageHeader('Ingestion & Recall', t('ingestLead'))}<section class="grid cols-2"><div class="panel"><h3>Guided ingestion</h3><div class="radio-grid">${['Memory', 'Document', 'Record', 'File'].map((kind) => `<label class="radio-card"><input type="radio" name="ingestType" value="${kind}" ${kind === 'Memory' ? 'checked' : ''} /><strong>${kind}</strong><span>${kindStatus[kind]}</span></label>`).join('')}</div><div class="form-grid">${FormField(t('tenantId'), 'ingestTenant', state.tenant)}${FormField(t('collection'), 'ingestCollection', state.collection)}${FormField(t('table'), 'ingestTable', 'data')}${TextAreaField('Content / JSON payload', 'ingestContent', '')}</div><div class="page-actions">${ActionButton(t('storeMemory'), 'store-memory', 'primary')}${ActionButton(t('storeDocument'), 'store-document')}${ActionButton(t('storeRecord'), 'store-record')}</div></div><div class="panel"><h3>Recall and build_context</h3><div class="form-grid">${FormField(t('tenantId'), 'recallTenant', state.tenant)}${FormField(t('collection'), 'recallCollection', state.collection)}${FormField(t('recallQuery'), 'recallInput', '', 'text', 'class="wide"')}</div><div class="page-actions">${ActionButton(t('recall'), 'run-recall', 'primary')}${ActionButton(t('buildContext'), 'build-context')}</div><div id="recallResult">${JsonViewer(state.lists.recallResult || { status: 'ready' })}</div></div></section>`;
}

function GraphPage() {
  const rows = annotateRows(state.lists.graph || [], 'graph');
  return `${PageHeader('Graph', t('graphLead'), ActionButton(t('refresh'), 'refresh'))}<section class="grid cols-2"><div class="panel">${DataTable(rows, [{ key: 'id', label: 'id' }, { key: 'from_id', label: 'from' }, { key: 'to_id', label: 'to' }, { key: 'relation', label: 'relation' }], 'No graph edges')}</div><div class="panel"><h3>Planned graph workbench</h3>${StatusBadge(t('planned'), 'planned')}<p class="muted">Add-edge and graph traversal UI are planned. Current page lists graph edges from /admin/graph-edges.</p></div></section>`;
}

function ApiReferencePage() {
  const groups = [
    ['Health', [['GET', '/health', 'Implemented', 'curl http://localhost:8080/health']]],
    ['Admin', [['GET', '/admin/bootstrap', 'Implemented', 'curl -H "x-admin-session: SESSION" http://localhost:8080/admin/bootstrap']]],
    ['Tenants', [['GET', '/admin/tenants', 'Implemented', 'curl -H "x-admin-session: SESSION" http://localhost:8080/admin/tenants'], ['POST', '/admin/tenants', 'Implemented', 'curl -X POST -H "x-admin-session: SESSION" -d \'{"id":"acme","name":"Acme"}\' http://localhost:8080/admin/tenants']]],
    ['Collections', [['GET', '/admin/collections', 'Implemented', 'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/collections?tenant_id=acme"']]],
    ['Records', [['GET', '/admin/records', 'Implemented', 'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/records?tenant_id=acme"'], ['POST', '/admin/records', 'Planned', 'record insert endpoint is planned']]],
    ['Memories', [['GET', '/admin/memories', 'Implemented', 'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/memories?tenant_id=acme"'], ['POST', '/admin/tenants/:tid/memories', 'Implemented', 'curl -X POST -H "x-admin-session: SESSION" -d \'{"collection":"support","text":"fact","memory_type":"semantic"}\' http://localhost:8080/admin/tenants/acme/memories']]],
    ['Documents', [['GET', '/admin/documents', 'Implemented', 'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/documents?tenant_id=acme"'], ['POST', '/admin/tenants/:tid/documents', 'Implemented', 'curl -X POST -H "x-admin-session: SESSION" -d \'{"collection":"support","text":"document"}\' http://localhost:8080/admin/tenants/acme/documents']]],
    ['SQL', [['POST', '/admin/sql', 'Implemented', 'curl -X POST -H "x-admin-session: SESSION" -d \'{"tenant_id":"acme","sql":"select * from systems limit 5"}\' http://localhost:8080/admin/sql']]],
    ['Recall & Context', [['POST', '/admin/tenants/:tid/recall', 'Implemented', 'curl -X POST -H "x-admin-session: SESSION" -d \'{"query":"postgresql","top_k":5}\' http://localhost:8080/admin/tenants/acme/recall'], ['POST', '/admin/tenants/:tid/context', 'Implemented', 'curl -X POST -H "x-admin-session: SESSION" -d \'{"query":"postgresql","max_tokens":1024}\' http://localhost:8080/admin/tenants/acme/context']]],
    ['Files', [['GET', '/admin/files', 'Implemented', 'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/files?tenant_id=acme"'], ['POST', '/admin/files', 'Planned', 'file upload/import endpoint is planned']]],
    ['Graph', [['GET', '/admin/graph-edges', 'Implemented', 'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/graph-edges?tenant_id=acme"'], ['POST', '/tenants/:tid/graph/traverse', 'Implemented', 'service API with X-Api-Key']]],
    ['Service Keys', [['POST', '/admin/api-key/rotate', 'Implemented', 'curl -X POST -H "x-admin-session: SESSION" -d \'{"length":32}\' http://localhost:8080/admin/api-key/rotate']]],
  ];
  const panels = groups.map(([group, endpoints]) => `<div class="panel"><h3>${esc(group)}</h3>${endpoints.map(([method, path, status, curl], idx) => CodeBlock(`${method} ${path} ${StatusBadge(status, status === 'Implemented' ? 'ok' : 'planned')}`, curl, `<button type="button" data-action="copy-code" data-code="${esc(curl)}">Copy</button>`)).join('')}</div>`).join('');
  return `${PageHeader('API Reference', t('apiReferenceLead'))}<section class="grid cols-2">${panels}</section>`;
}

function TenantsPage() {
  const rows = annotateRows(state.bootstrap?.tenants || [], 'tenants');
  return `${PageHeader('Tenants', 'Tenant is the top-level isolation boundary.', ActionButton(t('createTenant'), 'create-tenant', 'primary'))}<section class="grid cols-2"><div class="panel">${DataTable(rows, [{ key: 'id', label: 'id' }, { key: 'name', label: 'name' }], 'No tenants')}</div><div class="panel"><h3>${esc(t('createTenant'))}</h3><div class="form-grid">${FormField(t('tenantId'), 'tenantFormId')}${FormField(t('name'), 'tenantFormName')}</div><div class="page-actions">${ActionButton(t('save'), 'create-tenant', 'primary')}</div></div></section>`;
}

function ServiceKeysPage() {
  const providers = annotateRows(state.providers || [], 'providers');
  return `${PageHeader('Service Keys', t('providerLead'), ActionButton(t('rotateKey'), 'rotate-key', 'danger'))}<section class="grid cols-2"><div class="panel"><h3>Service API key</h3>${StatusBadge(state.bootstrap?.config?.api_key_set ? 'Configured' : 'Missing', state.bootstrap?.config?.api_key_set ? 'ok' : 'error')}<p class="muted">Length: ${esc(state.bootstrap?.config?.api_key_length ?? '-')}</p><div class="page-actions">${ActionButton(t('rotateKey'), 'rotate-key', 'danger')}</div></div><div class="panel"><h3>LLM Providers</h3>${DataTable(providers, [{ key: 'id', label: 'id' }, { key: 'kind', label: 'kind' }, { key: 'model', label: 'model' }, { key: 'api_key_set', label: 'key' }], 'No providers')}</div></section><section class="grid cols-2"><div class="panel"><h3>Provider registry</h3><div class="form-grid">${FormField('Provider id', 'providerId', 'ollama')}${FormField('Kind', 'providerKind', 'ollama')}${FormField('Base URL', 'providerUrl', 'http://localhost:11434')}${FormField('Model', 'providerModel', 'llama3.2')}${FormField('API key', 'providerKey', '', 'password')}</div><div class="page-actions">${ActionButton(t('save'), 'save-provider', 'primary')}${ActionButton('Validate', 'validate-provider')}</div></div><div class="panel"><h3>Provider result</h3>${JsonViewer(state.providerResult || { status: 'ready' })}</div></section>`;
}

function ObservabilityPage() {
  const s = stats();
  const physical = [
    ['Health', 'Implemented', 'GET /health'],
    ['Stats', 'Implemented', '/admin/bootstrap stats'],
    ['Chunks', 'Planned', `${s.chunks ?? 0} chunks counted; list endpoint planned`],
    ['WAL entries', 'Implemented', String(s.wal_entries ?? '-')],
    ['Snapshot details', 'Planned', 'snapshot inspector'],
    ['Audit log', 'Implemented', `${s.audit_log_bytes ?? 0} B`],
    ['Index status', 'Planned', 'index health panel'],
    ['Recent errors/logs', 'Planned', 'log stream'],
  ];
  return `${PageHeader('Observability', t('observabilityLead'), ActionButton(t('refresh'), 'refresh'))}<section class="grid cols-3">${StatCard(t('health'), state.session ? 'OK' : '-', 'server route available')}${StatCard('Disk bytes', s.disk_bytes ?? '-')}${StatCard('Indexed entries', s.indexed_entries ?? '-')}</section><section class="panel">${DataTable(physical.map((row, idx) => ({ id: idx, name: row[0], status: row[1], detail: row[2], __list: 'observability' })), [{ key: 'name', label: 'object' }, { key: 'status', label: 'status' }, { key: 'detail', label: 'detail' }], 'No observability data')}</section>`;
}

function SettingsPage() {
  return `${PageHeader('Settings', 'Local admin preferences and planned runtime configuration.') }<section class="grid cols-2"><div class="panel"><h3>Interface</h3><div class="form-grid">${FormField(t('language'), 'settingsLanguage', state.lang)}</div></div><div class="panel"><h3>Runtime configuration</h3>${StatusBadge(t('planned'), 'planned')}<p class="muted">Embedding provider runtime, reindex controls, and table catalog settings are planned.</p></div></section>`;
}

function LoginPage() {
  return `<div class="login-screen"><form class="login-card" id="loginForm"><div class="mark" aria-hidden="true"></div><h1>${esc(t('appTitle'))}</h1><p>${esc(t('loginLead'))}</p>${FormField(t('adminUser'), 'adminUser', state.username)}${FormField(t('adminPassword'), 'adminPass', '', 'password')}<label class="field"><span>${esc(t('language'))}</span><select id="loginLang"><option value="en" ${state.lang === 'en' ? 'selected' : ''}>EN</option><option value="pt" ${state.lang === 'pt' ? 'selected' : ''}>PT</option></select></label><button class="primary" type="submit">${esc(t('login'))}</button></form>${state.toast ? `<div class="toast">${esc(state.toast)}</div>` : ''}</div>`;
}

function renderPage() {
  if (!state.session) return LoginPage();
  if (state.page === 'dashboard') return AppShell(DashboardPage());
  if (state.page === 'data-explorer') return AppShell(DataExplorerPage());
  if (state.page === 'sql-editor') return AppShell(SqlEditorPage());
  if (state.page === 'collections') return AppShell(CollectionsPage());
  if (['records', 'memories', 'documents', 'files'].includes(state.page)) return AppShell(ObjectListPage(state.page));
  if (state.page === 'ingestion-recall') return AppShell(IngestionRecallPage());
  if (state.page === 'graph') return AppShell(GraphPage());
  if (state.page === 'api-reference') return AppShell(ApiReferencePage());
  if (state.page === 'tenants') return AppShell(TenantsPage());
  if (state.page === 'service-keys') return AppShell(ServiceKeysPage());
  if (state.page === 'observability') return AppShell(ObservabilityPage());
  if (state.page === 'settings') return AppShell(SettingsPage());
  return AppShell(DashboardPage());
}

function render() {
  document.documentElement.lang = state.lang;
  root.innerHTML = renderPage();
}

// Data loading ----------------------------------------------------------------

async function loadBootstrap() {
  if (!state.session) return;
  state.bootstrap = await request('/admin/bootstrap');
  if (!state.tenant && state.bootstrap.tenants.length > 0) {
    state.tenant = state.bootstrap.tenants[0].id;
  }
  await Promise.all([loadCollections(), loadProviders()]);
  persist();
}

async function loadCollections() {
  if (!state.session) return;
  const params = state.tenant ? `?${new URLSearchParams({ tenant_id: state.tenant })}` : '';
  state.collections = await request(`/admin/collections${params}`);
  if (!state.collection && state.collections.length > 0) {
    state.collection = state.collections[0].name;
  }
}

async function loadProviders() {
  state.providers = await request('/admin/llm-providers');
}

async function loadList(kind) {
  if (!state.tenant) return;
  const params = new URLSearchParams({ tenant_id: state.tenant });
  if (state.collection && ['memories', 'documents', 'records', 'files'].includes(kind)) {
    params.set('collection', state.collection);
  }
  state.lists[kind] = await request(`/admin/${kind}?${params}`);
}

async function hydratePage() {
  await loadBootstrap();
  const listPages = ['records', 'memories', 'documents', 'files'];
  if (state.page === 'data-explorer') await loadList('records');
  if (listPages.includes(state.page)) await loadList(state.page);
  if (state.page === 'graph') {
    if (state.tenant) {
      state.lists.graph = await request(`/admin/graph-edges?${new URLSearchParams({ tenant_id: state.tenant })}`);
    }
  }
  render();
}

// Actions ---------------------------------------------------------------------

async function login(event) {
  event.preventDefault();
  const username = document.getElementById('adminUser').value.trim();
  const password = document.getElementById('adminPass').value;
  state.username = username;
  const data = await publicRequest('/admin/login', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ username, password }),
  });
  state.session = data.session;
  state.lastOperation = 'Admin login succeeded.';
  persist();
  await hydratePage();
}

async function createTenant() {
  const id = document.getElementById('tenantFormId')?.value.trim();
  const name = document.getElementById('tenantFormName')?.value.trim() || id;
  if (!id) throw new Error('tenant id required');
  await request('/admin/tenants', { method: 'POST', body: JSON.stringify({ id, name }) });
  state.tenant = id;
  state.lastOperation = `Created tenant ${id}.`;
  await hydratePage();
}

async function createCollection() {
  const name = document.getElementById('collectionName')?.value.trim();
  const description = document.getElementById('collectionDescription')?.value.trim();
  if (!state.tenant) throw new Error('tenant required');
  if (!name) throw new Error('collection name required');
  await request(`/admin/tenants/${encodeURIComponent(state.tenant)}/collections`, {
    method: 'POST',
    body: JSON.stringify({ name, description }),
  });
  state.collection = name;
  state.lastOperation = `Created collection ${name}.`;
  await hydratePage();
}

async function runSql() {
  const sql = document.getElementById('sqlInput')?.value.trim();
  if (!state.tenant) throw new Error('tenant required');
  state.sqlError = '';
  try {
    state.sqlResult = await request('/admin/sql', {
      method: 'POST',
      body: JSON.stringify({ tenant_id: state.tenant, sql }),
    });
    state.lastOperation = `SQL returned ${state.sqlResult.row_count} row(s).`;
  } catch (err) {
    state.sqlResult = null;
    state.sqlError = err.message || String(err);
    state.lastOperation = `${t('error')}: ${state.sqlError}`;
  }
  render();
}

async function storeMemory() {
  const tenant = document.getElementById('ingestTenant')?.value.trim() || state.tenant;
  const collection = document.getElementById('ingestCollection')?.value.trim() || state.collection;
  const text = document.getElementById('ingestContent')?.value.trim();
  if (!tenant || !collection || !text) throw new Error('tenant, collection, and content required');
  state.lists.ingestResult = await request(`/admin/tenants/${encodeURIComponent(tenant)}/memories`, {
    method: 'POST',
    body: JSON.stringify({ collection, text, memory_type: 'semantic' }),
  });
  state.lastOperation = 'Stored memory.';
  await hydratePage();
}

async function storeDocument() {
  const tenant = document.getElementById('ingestTenant')?.value.trim() || state.tenant;
  const collection = document.getElementById('ingestCollection')?.value.trim() || state.collection;
  const text = document.getElementById('ingestContent')?.value.trim();
  if (!tenant || !collection || !text) throw new Error('tenant, collection, and content required');
  state.lists.ingestResult = await request(`/admin/tenants/${encodeURIComponent(tenant)}/documents`, {
    method: 'POST',
    body: JSON.stringify({ collection, text }),
  });
  state.lastOperation = 'Stored document.';
  await hydratePage();
}

async function storeRecord() {
  const tenant = document.getElementById('ingestTenant')?.value.trim() || state.tenant;
  const collection = document.getElementById('ingestCollection')?.value.trim() || state.collection;
  const table = document.getElementById('ingestTable')?.value.trim() || 'data';
  const raw = document.getElementById('ingestContent')?.value.trim();
  if (!tenant || !collection || !raw) throw new Error('tenant, collection, and JSON payload required');
  let payload;
  try {
    payload = JSON.parse(raw);
  } catch {
    throw new Error('Content must be a valid JSON object for records');
  }
  if (typeof payload !== 'object' || Array.isArray(payload) || payload === null) {
    throw new Error('Record payload must be a JSON object');
  }
  state.lists.ingestResult = await request(`/admin/tenants/${encodeURIComponent(tenant)}/records`, {
    method: 'POST',
    body: JSON.stringify({ collection, table, payload }),
  });
  state.lastOperation = 'Stored record.';
  await hydratePage();
}

async function runRecall(kind) {
  const tenant = document.getElementById('recallTenant')?.value.trim() || state.tenant;
  const collection = document.getElementById('recallCollection')?.value.trim() || state.collection;
  const query = document.getElementById('recallInput')?.value.trim();
  if (!tenant || !query) throw new Error('tenant and query required');
  const path = kind === 'context' ? 'context' : 'recall';
  const body = kind === 'context'
    ? { query, collection: collection || null, max_tokens: 1024, include_related: true }
    : { query, collection: collection || null, top_k: 5 };
  state.lists.recallResult = await request(`/admin/tenants/${encodeURIComponent(tenant)}/${path}`, {
    method: 'POST',
    body: JSON.stringify(body),
  });
  state.lastOperation = kind === 'context' ? 'Built context.' : 'Recall completed.';
  render();
}

async function rotateKey() {
  const result = await request('/admin/api-key/rotate', {
    method: 'POST',
    body: JSON.stringify({ length: 32 }),
  });
  state.detail = { title: 'Rotated service key', value: result };
  state.lastOperation = 'Rotated service API key.';
  await hydratePage();
}

function providerBody() {
  return {
    id: document.getElementById('providerId')?.value.trim(),
    kind: document.getElementById('providerKind')?.value.trim(),
    base_url: document.getElementById('providerUrl')?.value.trim(),
    model: document.getElementById('providerModel')?.value.trim(),
    api_key: document.getElementById('providerKey')?.value.trim() || null,
    is_default: true,
  };
}

async function saveProvider() {
  state.providerResult = await request('/admin/llm-providers', {
    method: 'POST',
    body: JSON.stringify(providerBody()),
  });
  state.lastOperation = 'Saved provider configuration.';
  await hydratePage();
}

async function validateProvider() {
  state.providerResult = await request('/admin/llm-providers/validate', {
    method: 'POST',
    body: JSON.stringify(providerBody()),
  });
  state.lastOperation = 'Validated provider configuration locally.';
  render();
}

function openDetail(list, index) {
  const rows = list === 'sql'
    ? state.sqlResult?.rows || []
    : list === 'collections'
      ? state.collections || []
      : list === 'tenants'
        ? state.bootstrap?.tenants || []
        : list === 'graph'
          ? state.lists.graph || []
          : list === 'providers'
            ? state.providers || []
            : state.lists[list] || [];
  const value = rows[Number(index)];
  if (value) {
    state.detail = { title: value.id || value.name || list, value };
    render();
  }
}

function copyCode(value) {
  navigator.clipboard?.writeText(value).then(() => showToast(t('copied')));
}

async function handleAction(target) {
  const action = target.dataset.action;
  if (!action) return;
  if (action === 'refresh') await hydratePage();
  if (action === 'logout') {
    state.session = '';
    state.bootstrap = null;
    persist();
    render();
  }
  if (action === 'choose-tenant') {
    state.tenant = target.dataset.tenant;
    state.collection = '';
    await hydratePage();
  }
  if (action === 'choose-collection') {
    state.collection = target.dataset.collection;
    persist();
    render();
  }
  if (action === 'tab') {
    state.tab = target.dataset.tab;
    render();
  }
  if (action === 'sql-tab') {
    state.sqlTab = target.dataset.tab;
    render();
  }
  if (action === 'use-sql') {
    document.getElementById('sqlInput').value = target.dataset.sql;
  }
  if (action === 'run-sql') await runSql();
  if (action === 'create-tenant') await createTenant();
  if (action === 'create-collection') await createCollection();
  if (action === 'store-memory') await storeMemory();
  if (action === 'store-document') await storeDocument();
  if (action === 'store-record') await storeRecord();
  if (action === 'run-recall') await runRecall('recall');
  if (action === 'build-context') await runRecall('context');
  if (action === 'rotate-key') await rotateKey();
  if (action === 'save-provider') await saveProvider();
  if (action === 'validate-provider') await validateProvider();
  if (action === 'detail') openDetail(target.dataset.list, target.dataset.index);
  if (action === 'close-detail') {
    state.detail = null;
    render();
  }
  if (action === 'copy-code') copyCode(target.dataset.code);
}

root.addEventListener('click', (event) => {
  const routeTarget = event.target.closest('[data-route]');
  if (routeTarget) {
    route(routeTarget.dataset.route);
    return;
  }
  const actionTarget = event.target.closest('[data-action]');
  if (actionTarget) {
    handleAction(actionTarget).catch(showError);
  }
});

root.addEventListener('submit', (event) => {
  if (event.target.id === 'loginForm') {
    login(event).catch(showError);
  }
});

root.addEventListener('change', (event) => {
  if (event.target.id === 'globalTenant') {
    state.tenant = event.target.value;
    state.collection = '';
    persist();
    hydratePage().catch(showError);
  }
  if (event.target.id === 'languageSelect' || event.target.id === 'loginLang') {
    state.lang = event.target.value;
    persist();
    render();
  }
});

render();

if (state.session) {
  hydratePage().catch((err) => {
    state.session = '';
    showError(err);
    render();
  });
}

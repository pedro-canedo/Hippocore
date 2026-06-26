const $ = (id) => document.getElementById(id);

const els = {
  loginForm: $('loginForm'),
  language: $('language'),
  adminUser: $('adminUser'),
  adminPass: $('adminPass'),
  sessionStatus: $('sessionStatus'),
  dataDir: $('dataDir'),
  tenantCount: $('tenantCount'),
  objectCounts: $('objectCounts'),
  statusOutput: $('statusOutput'),
  tenants: $('tenants'),
  collections: $('collections'),
  objectsOutput: $('objectsOutput'),
  tenantId: $('tenantId'),
  tenantName: $('tenantName'),
  collection: $('collection'),
  collectionDescription: $('collectionDescription'),
  memoryText: $('memoryText'),
  documentText: $('documentText'),
  recallQuery: $('recallQuery'),
  recordSql: $('recordSql'),
  providerId: $('providerId'),
  providerKind: $('providerKind'),
  providerUrl: $('providerUrl'),
  providerModel: $('providerModel'),
  providerKey: $('providerKey'),
  providerOutput: $('providerOutput'),
  resultOutput: $('resultOutput'),
};

const messages = {
  en: {
    adminPass: 'admin password',
    adminUser: 'admin user',
    apiKey: 'API key',
    baseUrl: 'Base URL',
    buildContext: 'Build context',
    catalog: 'Catalog',
    collection: 'Collection',
    collectionDescription: 'Collection description',
    collections: 'Collections',
    createCollection: 'Create collection',
    createTenant: 'Create tenant',
    dataDir: 'Data dir',
    documentPlaceholder: 'Paste source document text',
    documents: 'Documents',
    documentText: 'Document text',
    files: 'Files',
    ingestionRecall: 'Ingestion & Recall',
    kind: 'Kind',
    kindPlaceholder: 'ollama or openrouter',
    llmProviders: 'LLM Providers',
    login: 'Log in',
    memories: 'Memories',
    memoryText: 'Memory text',
    model: 'Model',
    objects: 'Objects',
    operationalStatus: 'Operational Status',
    optional: 'optional',
    providerId: 'Provider id',
    providersAfterLogin: 'Providers are loaded after login.',
    ready: 'Ready.',
    recallQuery: 'Recall query',
    records: 'Records',
    recordsQuery: 'Records query',
    refresh: 'Refresh',
    restrictedSql: 'Restricted SQL',
    result: 'Result',
    rotateKey: 'Rotate service API key',
    runRecall: 'Run recall',
    runRecordsQuery: 'Run records query',
    saveProvider: 'Save provider',
    selectTenant: 'Select a tenant and object type.',
    session: 'Session',
    signedOut: 'signed out',
    storeDocument: 'Store document',
    storeMemory: 'Store memory',
    tagline: 'Data catalog, ingestion, restricted records query, LLM providers, and service keys.',
    tenantId: 'Tenant id',
    tenantName: 'New tenant name',
    tenants: 'Tenants',
    validate: 'Validate',
  },
  pt: {
    adminPass: 'senha admin',
    adminUser: 'usuario admin',
    apiKey: 'chave de API',
    baseUrl: 'URL base',
    buildContext: 'Montar contexto',
    catalog: 'Catalogo',
    collection: 'Colecao',
    collectionDescription: 'Descricao da colecao',
    collections: 'Colecoes',
    createCollection: 'Criar colecao',
    createTenant: 'Criar tenant',
    dataDir: 'Diretorio de dados',
    documentPlaceholder: 'Cole o texto do documento fonte',
    documents: 'Documentos',
    documentText: 'Texto do documento',
    files: 'Arquivos',
    ingestionRecall: 'Ingestao e recall',
    kind: 'Tipo',
    kindPlaceholder: 'ollama ou openrouter',
    llmProviders: 'Providers LLM',
    login: 'Entrar',
    memories: 'Memorias',
    memoryText: 'Texto da memoria',
    model: 'Modelo',
    objects: 'Objetos',
    operationalStatus: 'Status operacional',
    optional: 'opcional',
    providerId: 'ID do provider',
    providersAfterLogin: 'Providers aparecem apos o login.',
    ready: 'Pronto.',
    recallQuery: 'Consulta de recall',
    records: 'Records',
    recordsQuery: 'Query de records',
    refresh: 'Atualizar',
    restrictedSql: 'SQL restrito',
    result: 'Resultado',
    rotateKey: 'Rotacionar API key de servico',
    runRecall: 'Executar recall',
    runRecordsQuery: 'Executar query de records',
    saveProvider: 'Salvar provider',
    selectTenant: 'Selecione um tenant e um tipo de objeto.',
    session: 'Sessao',
    signedOut: 'desconectado',
    storeDocument: 'Salvar documento',
    storeMemory: 'Salvar memoria',
    tagline: 'Catalogo de dados, ingestao, consulta restrita de records, providers LLM e chaves de servico.',
    tenantId: 'ID do tenant',
    tenantName: 'Nome do novo tenant',
    tenants: 'Tenants',
    validate: 'Validar',
  },
  es: {
    adminPass: 'contrasena admin',
    adminUser: 'usuario admin',
    apiKey: 'clave API',
    baseUrl: 'URL base',
    buildContext: 'Construir contexto',
    catalog: 'Catalogo',
    collection: 'Coleccion',
    collectionDescription: 'Descripcion de la coleccion',
    collections: 'Colecciones',
    createCollection: 'Crear coleccion',
    createTenant: 'Crear tenant',
    dataDir: 'Directorio de datos',
    documentPlaceholder: 'Pega el texto del documento fuente',
    documents: 'Documentos',
    documentText: 'Texto del documento',
    files: 'Archivos',
    ingestionRecall: 'Ingestion y recall',
    kind: 'Tipo',
    kindPlaceholder: 'ollama u openrouter',
    llmProviders: 'Providers LLM',
    login: 'Entrar',
    memories: 'Memorias',
    memoryText: 'Texto de memoria',
    model: 'Modelo',
    objects: 'Objetos',
    operationalStatus: 'Estado operativo',
    optional: 'opcional',
    providerId: 'ID del provider',
    providersAfterLogin: 'Los providers se cargan despues del login.',
    ready: 'Listo.',
    recallQuery: 'Consulta de recall',
    records: 'Records',
    recordsQuery: 'Query de records',
    refresh: 'Actualizar',
    restrictedSql: 'SQL restringido',
    result: 'Resultado',
    rotateKey: 'Rotar API key de servicio',
    runRecall: 'Ejecutar recall',
    runRecordsQuery: 'Ejecutar query de records',
    saveProvider: 'Guardar provider',
    selectTenant: 'Selecciona un tenant y un tipo de objeto.',
    session: 'Sesion',
    signedOut: 'desconectado',
    storeDocument: 'Guardar documento',
    storeMemory: 'Guardar memoria',
    tagline: 'Catalogo de datos, ingestion, consulta restringida de records, providers LLM y claves de servicio.',
    tenantId: 'ID del tenant',
    tenantName: 'Nombre del nuevo tenant',
    tenants: 'Tenants',
    validate: 'Validar',
  },
};

const state = {
  session: localStorage.getItem('hippocore.adminSession') || '',
  lang: localStorage.getItem('hippocore.lang') || 'en',
  tenant: '',
};

els.language.value = state.lang;
els.adminUser.value = localStorage.getItem('hippocore.adminUser') || 'admin';
els.providerId.value = 'ollama';
els.providerKind.value = 'ollama';
els.providerUrl.value = 'http://localhost:11434';
els.providerModel.value = 'llama3.2';

function t(key) {
  return messages[state.lang]?.[key] || messages.en[key] || key;
}

function applyLanguage() {
  document.documentElement.lang = state.lang;
  for (const node of document.querySelectorAll('[data-i18n]')) {
    node.textContent = t(node.dataset.i18n);
  }
  for (const node of document.querySelectorAll('[data-i18n-placeholder]')) {
    node.placeholder = t(node.dataset.i18nPlaceholder);
  }
  if (!state.session) {
    els.sessionStatus.textContent = t('signedOut');
    els.objectsOutput.textContent = t('selectTenant');
    els.providerOutput.textContent = t('providersAfterLogin');
    els.resultOutput.textContent = t('ready');
  }
}

function show(target, value) {
  target.textContent = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
}

async function request(path, options = {}) {
  document.body.classList.add('loading');
  const headers = Object.assign(
    { 'content-type': 'application/json', 'x-admin-session': state.session },
    options.headers || {},
  );
  try {
    const res = await fetch(path, Object.assign({}, options, { headers }));
    const text = await res.text();
    const body = text ? JSON.parse(text) : null;
    if (!res.ok) {
      throw new Error(body?.error || text || `HTTP ${res.status}`);
    }
    return body;
  } finally {
    document.body.classList.remove('loading');
  }
}

function saveSession() {
  localStorage.setItem('hippocore.adminSession', state.session);
  localStorage.setItem('hippocore.adminUser', els.adminUser.value.trim());
  localStorage.setItem('hippocore.lang', state.lang);
}

async function login(event) {
  event.preventDefault();
  const body = {
    username: els.adminUser.value.trim(),
    password: els.adminPass.value,
  };
  const data = await fetch('/admin/login', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  }).then(async (res) => {
    const text = await res.text();
    const parsed = text ? JSON.parse(text) : null;
    if (!res.ok) throw new Error(parsed?.error || text || `HTTP ${res.status}`);
    return parsed;
  });
  state.session = data.session;
  saveSession();
  await loadBootstrap();
}

async function loadBootstrap() {
  if (!state.session) {
    els.sessionStatus.textContent = t('signedOut');
    return;
  }
  const data = await request('/admin/bootstrap');
  els.sessionStatus.textContent = 'active';
  els.dataDir.textContent = data.config.data_dir;
  els.tenantCount.textContent = String(data.tenants.length);
  els.objectCounts.textContent = `${data.stats.documents} docs, ${data.stats.memories} memories, ${data.stats.records} records, ${data.stats.files} files`;
  show(els.statusOutput, data.stats);
  renderTenants(data.tenants);
  await loadProviders();
}

function renderTenants(tenants) {
  els.tenants.innerHTML = '';
  for (const tenant of tenants) {
    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = `${tenant.id} - ${tenant.name}`;
    button.onclick = async () => {
      state.tenant = tenant.id;
      els.tenantId.value = tenant.id;
      await loadCollections();
    };
    els.tenants.appendChild(button);
  }
}

async function loadCollections() {
  if (!state.tenant) return;
  const params = new URLSearchParams({ tenant_id: state.tenant });
  const collections = await request(`/admin/collections?${params}`);
  els.collections.innerHTML = '';
  for (const collection of collections) {
    const chip = document.createElement('span');
    chip.className = 'chip';
    chip.textContent = collection.description
      ? `${collection.name} - ${collection.description}`
      : collection.name;
    els.collections.appendChild(chip);
  }
}

async function loadObjects(kind) {
  const tenant = els.tenantId.value.trim() || state.tenant;
  if (!tenant) throw new Error('tenant is required');
  const params = new URLSearchParams({ tenant_id: tenant });
  if (els.collection.value.trim()) params.set('collection', els.collection.value.trim());
  const items = await request(`/admin/${kind}?${params}`);
  show(els.objectsOutput, items);
}

async function createTenant() {
  const body = {
    id: els.tenantId.value.trim(),
    name: els.tenantName.value.trim() || els.tenantId.value.trim(),
  };
  show(els.resultOutput, await request('/admin/tenants', {
    method: 'POST',
    body: JSON.stringify(body),
  }));
  await loadBootstrap();
}

async function createCollection() {
  const tenant = els.tenantId.value.trim();
  const body = {
    name: els.collection.value.trim(),
    description: els.collectionDescription.value.trim(),
  };
  show(els.resultOutput, await request(`/admin/tenants/${encodeURIComponent(tenant)}/collections`, {
    method: 'POST',
    body: JSON.stringify(body),
  }));
  await loadBootstrap();
}

async function storeMemory() {
  const tenant = els.tenantId.value.trim();
  const body = {
    collection: els.collection.value.trim(),
    text: els.memoryText.value.trim(),
    memory_type: 'semantic',
  };
  show(els.resultOutput, await request(`/admin/tenants/${encodeURIComponent(tenant)}/memories`, {
    method: 'POST',
    body: JSON.stringify(body),
  }));
}

async function storeDocument() {
  const tenant = els.tenantId.value.trim();
  const body = {
    collection: els.collection.value.trim(),
    text: els.documentText.value.trim(),
  };
  show(els.resultOutput, await request(`/admin/tenants/${encodeURIComponent(tenant)}/documents`, {
    method: 'POST',
    body: JSON.stringify(body),
  }));
}

async function runRecall() {
  const tenant = els.tenantId.value.trim();
  const body = {
    query: els.recallQuery.value.trim(),
    collection: els.collection.value.trim() || null,
    top_k: 5,
  };
  show(els.resultOutput, await request(`/admin/tenants/${encodeURIComponent(tenant)}/recall`, {
    method: 'POST',
    body: JSON.stringify(body),
  }));
}

async function buildContext() {
  const tenant = els.tenantId.value.trim();
  const body = {
    query: els.recallQuery.value.trim(),
    max_tokens: 1024,
    include_related: true,
  };
  show(els.resultOutput, await request(`/admin/tenants/${encodeURIComponent(tenant)}/context`, {
    method: 'POST',
    body: JSON.stringify(body),
  }));
}

async function runRecordSql() {
  const body = {
    tenant_id: els.tenantId.value.trim() || state.tenant,
    sql: els.recordSql.value.trim(),
  };
  show(els.resultOutput, await request('/admin/query-records', {
    method: 'POST',
    body: JSON.stringify(body),
  }));
}

function providerBody() {
  return {
    id: els.providerId.value.trim(),
    kind: els.providerKind.value.trim(),
    base_url: els.providerUrl.value.trim(),
    model: els.providerModel.value.trim(),
    api_key: els.providerKey.value.trim() || null,
    is_default: true,
  };
}

async function loadProviders() {
  show(els.providerOutput, await request('/admin/llm-providers'));
}

async function saveProvider() {
  show(els.resultOutput, await request('/admin/llm-providers', {
    method: 'POST',
    body: JSON.stringify(providerBody()),
  }));
  await loadProviders();
}

async function validateProvider() {
  show(els.resultOutput, await request('/admin/llm-providers/validate', {
    method: 'POST',
    body: JSON.stringify(providerBody()),
  }));
}

async function rotateKey() {
  show(els.resultOutput, await request('/admin/api-key/rotate', {
    method: 'POST',
    body: JSON.stringify({ length: 32 }),
  }));
  await loadBootstrap();
}

function bind(id, fn) {
  $(id).addEventListener('click', () => fn().catch((err) => show(els.resultOutput, String(err))));
}

els.loginForm.addEventListener('submit', (event) => login(event).catch((err) => show(els.resultOutput, String(err))));
bind('refresh', loadBootstrap);
bind('rotateKey', rotateKey);
bind('createTenant', createTenant);
bind('createCollection', createCollection);
bind('storeMemory', storeMemory);
bind('storeDocument', storeDocument);
bind('runRecall', runRecall);
bind('buildContext', buildContext);
bind('runRecordSql', runRecordSql);
bind('saveProvider', saveProvider);
bind('validateProvider', validateProvider);

els.language.addEventListener('change', () => {
  state.lang = els.language.value;
  saveSession();
  applyLanguage();
});

for (const button of document.querySelectorAll('button[data-kind]')) {
  button.addEventListener('click', () => loadObjects(button.dataset.kind).catch((err) => show(els.resultOutput, String(err))));
}

applyLanguage();

if (state.session) {
  loadBootstrap().catch((err) => show(els.resultOutput, String(err)));
}

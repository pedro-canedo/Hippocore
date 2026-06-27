/* Hippocore Control Plane — Liquid Glass Edition */
const root = document.getElementById('app');

const NAV = [
  ['operate', 'dashboard',        'dashboard'],
  ['operate', 'dataExplorer',     'data-explorer'],
  ['operate', 'sqlEditor',        'sql-editor'],
  ['data',    'collections',      'collections'],
  ['data',    'tables',           'tables'],
  ['data',    'records',          'records'],
  ['data',    'memories',         'memories'],
  ['data',    'documents',        'documents'],
  ['data',    'files',            'files'],
  ['context', 'ingestionRecall',  'ingestion-recall'],
  ['context', 'graph',            'graph'],
  ['admin',   'apiReference',     'api-reference'],
  ['admin',   'prompts',          'prompts'],
  ['admin',   'documentation',    'documentation'],
  ['admin',   'tenants',          'tenants'],
  ['admin',   'integrations',     'integrations'],
  ['admin',   'observability',    'observability'],
  ['admin',   'settings',         'settings'],
];

const EXPLORER_KINDS = [
  ['records', 'records'],
  ['memories', 'memories'],
  ['documents', 'documents'],
  ['files', 'files'],
];

const GROUP_LABELS = {
  operate: 'workspace',
  data:    'data',
  context: 'context',
  admin:   'admin',
};

const I18N = {
  en: {
    active: 'active',
    adminPassword: 'admin password',
    adminUser: 'admin user',
    apiReferenceLead: 'HTTP surface grouped by domain. Implemented endpoints are callable today; Planned entries describe the product direction.',
    appSubtitle: 'Local-first memory & context database',
    appTitle: 'Hippocore',
    buildContext: 'Build context',
    collection: 'Collection',
    collectionDescription: 'Collection description',
    collectionHelp: 'Collections are logical namespaces inside a tenant. Records, memories, documents, and file-derived context live under them.',
    contextBuilt: 'Context built successfully.',
    contextResult: 'Context result',
    content: 'Content',
    contentPlaceholder: 'Enter the knowledge you want Hippocore to retrieve later',
    copied: 'Copied!',
    copyId: 'Copy ID',
    createCollection: 'Create collection',
    createTenant: 'Create tenant',
    currentTenant: 'Tenant',
    dataDir: 'Data directory',
    dataExplorerLead: 'Browse logical records and physical context objects without mixing storage internals into the main workflow.',
    dashboardLead: 'Overview of the active local database, tenant scope, object counts, and next actions.',
    documentation: 'Documentation',
    documents: 'Documents',
    documentSingular: 'Document',
    empty: 'Nothing here yet.',
    error: 'Error',
    files: 'Files',
    fileSingular: 'File',
    getStarted: 'Get started',
    graph: 'Graph',
    graphLead: 'Inspect graph edges between context items. Multi-hop traversal is available through the service API.',
    health: 'Health',
    ingestLead: 'Store context and immediately test recall / build_context in the selected tenant.',
    ingestionRecall: 'Ingestion & Recall',
    ingestRequired: 'Tenant, collection, and content are required.',
    implemented: 'Implemented',
    itemsDropped: 'Items dropped',
    itemsIncluded: 'Items included',
    jsonPayload: 'JSON payload',
    language: 'Language',
    lastOperation: 'Last operation',
    login: 'Sign in',
    loginLead: 'Use the admin credentials configured for this local server.',
    logout: 'Sign out',
    memories: 'Memories',
    memorySingular: 'Memory',
    memoryStored: 'Memory stored. Run recall to retrieve it.',
    memoryStoredToast: 'Memory stored successfully.',
    memoryText: 'Memory text',
    name: 'Name',
    noTenant: 'Select tenant',
    noRecallResults: 'No recall results',
    noRecallResultsDesc: 'Try a broader query or ingest relevant context into this collection.',
    objectCounts: 'Object counts',
    observabilityLead: 'Operational view over health, stats, storage, WAL, snapshot, audit, and index status.',
    openDataExplorer: 'Open Data Explorer',
    openFileUploader: 'Open file uploader',
    openSql: 'Open SQL Editor',
    planned: 'Planned',
    providerLead: 'Manage local LLM provider registry and rotate the service API key used by automation.',
    refresh: 'Refresh',
    records: 'Records',
    recordSingular: 'Record',
    recordJsonInvalid: 'Content must be valid JSON for records.',
    recordObjectRequired: 'Record payload must be a JSON object.',
    recordRequired: 'Tenant, collection, and a JSON payload are required.',
    recordStored: 'Record stored.',
    recall: 'Run recall',
    recallCompleted: 'Recall completed.',
    recallQuery: 'Recall query',
    recallReady: 'Enter a natural-language query to inspect retrieved context.',
    recallRequired: 'Tenant and query are required.',
    recallResults: 'Recall results',
    rawJson: 'Raw JSON',
    rotateKey: 'Rotate key',
    run: 'Run',
    save: 'Save',
    serverStatus: 'Server status',
    session: 'Session',
    signedOut: 'signed out',
    sqlExamples: 'Examples',
    sqlLead: 'Run the supported read-only SQL-like record query layer with tenant isolation enforced by the backend.',
    storeDocument: 'Store document',
    storeContext: 'Store context',
    documentStored: 'Document stored and chunked. Run recall to retrieve its chunks.',
    documentStoredToast: 'Document stored and chunked.',
    storeMemory: 'Store memory',
    storeRecord: 'Store record',
    storeType: 'Type',
    table: 'Table',
    tenant: 'Tenant',
    tenantId: 'Tenant id',
    tenants: 'Tenants',
    title: 'Title',
    tokens: 'Tokens',
    aboutTables: 'About tables',
    addOrUpdateProvider: 'Add or update provider',
    admin: 'Admin',
    apiExposure: 'API exposure',
    apiKeyConfigured: 'API key configured',
    apiKeyNotSet: 'API key not set',
    apiReference: 'API Reference',
    auditLog: 'Audit log',
    auditRecords: 'Audit records',
    cancel: 'Cancel',
    chunkCount: 'Chunk count',
    collectionCreated: 'Collection "{name}" created.',
    collectionCreatedNext: 'Collection "{name}" created. Next: store your first memory or record.',
    collectionCreateNotice: 'After creating a collection, store records, memories, and documents through Ingestion & Recall or the API.',
    collectionNameRequired: 'Collection name is required.',
    collectionSelectBeforeCreate: 'Select a tenant in the topbar before creating a collection.',
    collectionOptional: 'Collection (optional)',
    collections: 'Collections',
    context: 'AI Context',
    copy: 'Copy',
    copySnippet: 'Copy snippet',
    copiedToClipboard: 'Copied to clipboard.',
    createCollectionStepDesc: 'Organize data into logical namespaces',
    createTenantStepDesc: 'Isolate your data with a named tenant',
    dashboard: 'Dashboard',
    data: 'Data',
    dataExplorer: 'Data Explorer',
    dataExplorerNotice: 'Collections are logical namespaces. Records are JSON-first structured data projected into context.',
    dataTypes: 'Data types',
    delete: 'Delete',
    deleteAll: 'Delete all',
    deletePromptConfirm: 'Delete prompt "{name}"?',
    deleteTableConfirm: 'Delete table "{name}" and all its records? This cannot be undone.',
    detail: 'Details',
    diskBytes: 'Disk bytes',
    docsGettingStarted: 'Getting Started',
    docsGuides: 'Guides',
    docsLead: 'Complete reference for Hippocore DB: concepts, guides, API, and architecture.',
    docsNotFound: 'Not found',
    docsReference: 'Reference',
    docsSectionNotFound: 'Section not found.',
    dropFiles: 'Drag and drop files here, or click to browse',
    editPrompt: 'Edit prompt',
    emptyDefault: 'No data to display.',
    errorMessage: 'Error: {message}',
    fileCountChunks: 'chunks',
    fileCountRows: 'rows',
    filesLead: 'Upload PDF, CSV, TXT, MD, or JSON. Hippocore stores each file as searchable documents or structured records.',
    generateLabels: 'Generate labels',
    graphEdges: 'Graph edges',
    graphEmptyDesc: 'Graph edges are created by explicit API operations between context items.',
    graphWorkbench: 'Graph workbench',
    graphWorkbenchLead: 'Interactive edge creation is planned. Multi-hop traversal is available through POST /tenants/:tid/graph/traverse.',
    healthEndpoint: 'Health endpoint',
    indexHealth: 'Index health',
    indexedEntries: 'Indexed entries',
    ingestContext: 'Ingest context',
    ingestContextStepDesc: 'Store memories, documents, and records',
    ingestItems: 'Ingest {type}',
    integrations: 'Integrations',
    integrationsLead: 'Configure the LLM brain, API exposure, and infrastructure integration.',
    isolationScope: 'isolation scope',
    keyLength: 'length',
    llmBrain: 'LLM brain',
    llmBrainLead: 'Configure the language model used by context recall and the Chat API.',
    logStream: 'Log stream',
    logicalView: 'Logical View',
    newPrompt: 'New prompt',
    newServiceKeyTitle: 'New service key: copy it now because it is shown once',
    noCollectionSelected: 'No collection selected',
    noCollections: 'No collections',
    noCollectionsYet: 'No collections yet',
    noDataSelected: 'No data selected',
    noDataSelectedDesc: 'Choose a tenant and collection, then select a data type to browse.',
    noGraphEdges: 'No graph edges',
    noObservabilityData: 'No observability data',
    noItems: 'No {type}',
    noProviders: 'No providers configured',
    noProvidersDesc: 'Add a provider using the form.',
    noPrompts: 'No prompts yet',
    noPromptsDesc: 'Create a prompt template for the Chat API.',
    noResults: 'No results',
    noResultsDesc: 'Run a SQL query to see results here.',
    noTables: 'No tables yet',
    noTablesDesc: 'Tables are created automatically when you store a record with a table name or upload CSV data.',
    noTenantSelected: 'No tenant selected',
    noTenants: 'No tenants yet',
    objectListLead: '{type} scoped to the active tenant.',
    objectListEmpty: 'No {type} exist in this tenant yet. Use Ingestion & Recall to create the first item.',
    observability: 'Observability',
    offline: 'Offline',
    online: 'Online',
    physicalInfo: 'Physical Info',
    physicalInfoNote: 'WAL, snapshot, chunks, audit, and index status are operational objects, not logical tables.',
    ping: 'Ping',
    plannedDetail: 'Planned',
    promptContentHelp: 'Content: use {{context}} and {{query}} as placeholders',
    defaultPrompt: 'You are a helpful AI assistant.\n\nContext:\n{{context}}\n\nAnswer the following question:\n{{query}}',
    promptDeleteDone: 'Prompt "{name}" deleted.',
    promptDescriptionOptional: 'Description (optional)',
    promptGlobalTenant: 'Tenant id (leave blank for global)',
    promptIdOptional: 'Prompt id (optional)',
    promptNameContentRequired: 'Name and content are required.',
    promptLibrary: 'Prompt library',
    promptSaved: 'Prompt "{name}" saved.',
    prompts: 'Prompts',
    promptsLead: 'Reusable system prompt templates for the RAG to LLM pipeline.',
    providerApiKeyOptional: 'API key (optional)',
    providerBaseUrl: 'Base URL',
    providerId: 'Provider id',
    providerIdRequired: 'Enter a provider id to test.',
    providerKind: 'Kind (ollama / openrouter / openai)',
    providerModel: 'Model',
    providerPinged: 'Provider "{id}" pinged.',
    providerSaved: 'Provider configuration saved.',
    providerSavedToast: 'Provider saved.',
    providerValidated: 'Provider validated locally.',
    question: 'Question',
    questionPlaceholder: 'Ask anything...',
    questionRequired: 'Enter a question.',
    quickActions: 'Quick actions',
    recallAndContext: 'Recall and Context',
    rawStats: 'Raw stats',
    response: 'Response',
    runRecallStepDesc: 'Query your context with hybrid search',
    runtimeConfiguration: 'Runtime configuration',
    runtimeConfigurationLead: 'Embedding runtime, reindex controls, hybrid_alpha tuning, and richer table settings are planned.',
    selectTenantFirst: 'Select a tenant first',
    selectTenantTopbar: 'Select a tenant in the topbar first.',
    searchCollections: 'Search collections',
    send: 'Send',
    serviceApiKeyLead: 'The service API key authenticates tenant-facing endpoints and can be rotated at any time.',
    serviceKeys: 'Service Keys',
    serviceKeyRotated: 'Service API key rotated.',
    serviceKeyRotatedToast: 'Key rotated. Copy it from the detail drawer.',
    settings: 'Settings',
    settingsLead: 'Interface preferences and planned runtime configuration.',
    settingsLanguageLead: 'Change language from the topbar. Runtime preferences will be added here later.',
    signedIn: 'Signed in successfully.',
    snapshotDetails: 'Snapshot details',
    structuredLogStream: 'Structured log stream',
    sqlEditor: 'SQL Editor',
    sqlErrorMessage: 'SQL error: {message}',
    sqlQuery: 'SQL query',
    sqlReturned: 'SQL returned {count} row(s).',
    sqlSupportNotice: 'Supported syntax: SELECT * FROM table with equality filters and LIMIT.',
    storeFirstRecord: 'Store record',
    storedFiles: 'Stored files',
    tableName: 'Table name',
    tableDeleted: 'Table "{name}" deleted.',
    tableResult: 'Table',
    tableRecords: 'Records',
    tables: 'Tables',
    tablesAboutLead: 'Tables are logical namespaces inside a collection for structured Records. They are created with the first record using that table name.',
    tablesDeleteNotice: 'Deleting a table permanently removes its records. Collections, Memories, and Documents are unaffected.',
    tablesLead: 'Logical namespaces inside a collection that group structured records.',
    tablesQueryNotice: 'Query tables in SQL Editor with: SELECT * FROM table_name LIMIT 10',
    tenantCreated: 'Tenant "{id}" created. Now create a collection.',
    tenantCreatedNext: 'Tenant "{id}" created. Next: create a collection.',
    tenantIdRequired: 'Tenant id is required.',
    tenantRequired: 'Tenant is required.',
    tenantLead: 'Tenants are the top-level isolation boundary for data, memories, and queries.',
    tenantPurpose: 'A tenant isolates data, collections, queries, and API keys. Create one per customer, environment, or use case.',
    testChat: 'Test chat (RAG to LLM)',
    upload: 'Upload',
    uploadCollectionRequired: 'Collection is required. Fill the collection field above the upload area.',
    uploadComplete: '{name} uploaded successfully.',
    uploadLastOperation: 'Uploaded {name} as {kind}.',
    uploadNetworkError: 'Network error during upload.',
    unknown: 'Unknown',
    uploadMapping: 'PDF -> chunked Document | CSV -> SQL-queryable Records | JSON array -> Records | JSON object, TXT, and MD -> Document',
    apiCurlExample: 'curl example: recall',
    composeSnippet: 'Docker Compose snippet',
    traefikComment: 'Add these labels to the hippocore service in docker-compose.yml',
    traefikDomain: 'Domain (for example, api.example.com)',
    traefikGenerator: 'Traefik config generator',
    traefikPath: 'Path prefix (for example, /api)',
    validate: 'Validate',
    walEntries: 'WAL entries',
    workspace: 'Workspace',
    apiStatus: 'API status',
    complete: 'Complete',
    continueSetup: 'Continue setup',
    dataOverview: 'Data overview',
    openObservability: 'Open Observability',
    operationalStatus: 'Operational status',
    ready: 'Ready',
    setupComplete: 'Workspace ready',
    setupCompleteLead: 'The initial database, query, and AI context workflows are ready to use.',
    setupLead: 'Follow the next incomplete step to prepare a usable AI-native database.',
    setupTitle: 'Workspace setup',
    stepAddData: 'Add your first data',
    stepAddDataDesc: 'Store a record, memory, document, or file',
    stepRunSql: 'Run a SQL query',
    stepRunSqlDesc: 'Inspect structured records with the SQL-like layer',
    stepTestRecall: 'Test recall and context',
    stepTestRecallDesc: 'Confirm which knowledge Hippocore retrieves for AI',
    welcome: 'Welcome to Hippocore',
    welcomeDesc: 'A local-first, AI-native memory and context database. Start by creating your first tenant to isolate your data.',
    noTenantDesc: 'You have no tenants yet. A tenant is the top-level isolation boundary for all your data, memories, and queries. Create one to begin.',
    noCollectionDesc: 'No collections in this tenant. Collections are logical namespaces — create one to start storing records, memories, and documents.',
  },
  pt: {
    active: 'ativa',
    adminPassword: 'senha admin',
    adminUser: 'usuario admin',
    apiReferenceLead: 'Superficie HTTP agrupada por dominio. Endpoints Implemented existem hoje; Planned indica direcao do produto.',
    appSubtitle: 'Banco local-first de memoria e contexto',
    appTitle: 'Hippocore',
    buildContext: 'Montar contexto',
    collection: 'Collection',
    collectionDescription: 'Descricao da collection',
    collectionHelp: 'Collections sao namespaces logicos dentro de um tenant. Records, memorias, documentos e contexto de arquivos vivem nelas.',
    contextBuilt: 'Contexto montado com sucesso.',
    contextResult: 'Resultado do contexto',
    content: 'Conteudo',
    contentPlaceholder: 'Digite o conhecimento que o Hippocore deve recuperar depois',
    copied: 'Copiado!',
    copyId: 'Copiar ID',
    createCollection: 'Criar collection',
    createTenant: 'Criar tenant',
    currentTenant: 'Tenant',
    dataDir: 'Diretorio de dados',
    dataExplorerLead: 'Navegue dados logicos e objetos fisicos de contexto sem misturar detalhes internos no fluxo principal.',
    dashboardLead: 'Visao geral do banco local ativo, escopo do tenant, contagens de objetos e proximas acoes.',
    documentation: 'Documentacao',
    documents: 'Documentos',
    documentSingular: 'Documento',
    empty: 'Nada para mostrar ainda.',
    error: 'Erro',
    files: 'Arquivos',
    fileSingular: 'Arquivo',
    getStarted: 'Comecar agora',
    graph: 'Grafo',
    graphLead: 'Inspecione arestas entre itens de contexto. Travessia multi-hop esta disponivel pela API de servico.',
    health: 'Saude',
    ingestLead: 'Armazene contexto e teste recall/build_context no tenant selecionado.',
    ingestionRecall: 'Ingestao e Recall',
    ingestRequired: 'Tenant, collection e conteudo sao obrigatorios.',
    implemented: 'Implementado',
    itemsDropped: 'Itens descartados',
    itemsIncluded: 'Itens incluidos',
    jsonPayload: 'Payload JSON',
    language: 'Idioma',
    lastOperation: 'Ultima operacao',
    login: 'Entrar',
    loginLead: 'Use as credenciais admin configuradas para este servidor local.',
    logout: 'Sair',
    memories: 'Memorias',
    memorySingular: 'Memoria',
    memoryStored: 'Memoria armazenada. Execute recall para recupera-la.',
    memoryStoredToast: 'Memoria armazenada com sucesso.',
    memoryText: 'Texto da memoria',
    name: 'Nome',
    noTenant: 'Selecionar tenant',
    noRecallResults: 'Nenhum resultado de recall',
    noRecallResultsDesc: 'Tente uma consulta mais ampla ou ingira contexto relevante nesta collection.',
    objectCounts: 'Contagens',
    observabilityLead: 'Visao operacional de health, stats, storage, WAL, snapshot, audit e status de indice.',
    openDataExplorer: 'Abrir Data Explorer',
    openFileUploader: 'Abrir upload de arquivos',
    openSql: 'Abrir SQL Editor',
    planned: 'Planejado',
    providerLead: 'Gerencie providers LLM locais e rotacione a API key usada por automacoes.',
    refresh: 'Atualizar',
    records: 'Records',
    recordSingular: 'Record',
    recordJsonInvalid: 'O conteudo deve ser JSON valido para records.',
    recordObjectRequired: 'O payload do record deve ser um objeto JSON.',
    recordRequired: 'Tenant, collection e payload JSON sao obrigatorios.',
    recordStored: 'Record armazenado.',
    recall: 'Executar recall',
    recallCompleted: 'Recall concluido.',
    recallQuery: 'Consulta de recall',
    recallReady: 'Digite uma pergunta em linguagem natural para inspecionar o contexto recuperado.',
    recallRequired: 'Tenant e consulta sao obrigatorios.',
    recallResults: 'Resultados do recall',
    rawJson: 'JSON bruto',
    rotateKey: 'Rotacionar chave',
    run: 'Executar',
    save: 'Salvar',
    serverStatus: 'Status do servidor',
    session: 'Sessao',
    signedOut: 'desconectada',
    sqlExamples: 'Exemplos',
    sqlLead: 'Execute a camada read-only de query estilo SQL com isolamento de tenant aplicado pelo backend.',
    storeDocument: 'Salvar documento',
    storeContext: 'Armazenar contexto',
    documentStored: 'Documento armazenado e dividido em chunks. Execute recall para recupera-los.',
    documentStoredToast: 'Documento armazenado e dividido em chunks.',
    storeMemory: 'Salvar memoria',
    storeRecord: 'Salvar record',
    storeType: 'Tipo',
    table: 'Tabela',
    tenant: 'Tenant',
    tenantId: 'Tenant id',
    tenants: 'Tenants',
    title: 'Titulo',
    tokens: 'Tokens',
    aboutTables: 'Sobre tables',
    addOrUpdateProvider: 'Adicionar ou atualizar provider',
    admin: 'Admin',
    apiExposure: 'Exposicao da API',
    apiKeyConfigured: 'API key configurada',
    apiKeyNotSet: 'API key nao configurada',
    apiReference: 'Referencia da API',
    auditLog: 'Log de auditoria',
    auditRecords: 'Registros de auditoria',
    cancel: 'Cancelar',
    chunkCount: 'Quantidade de chunks',
    collectionCreated: 'Collection "{name}" criada.',
    collectionCreatedNext: 'Collection "{name}" criada. Proximo passo: armazene a primeira memoria ou record.',
    collectionCreateNotice: 'Depois de criar a collection, armazene records, memorias e documentos por Ingestao e Recall ou pela API.',
    collectionNameRequired: 'O nome da collection e obrigatorio.',
    collectionSelectBeforeCreate: 'Selecione um tenant no topbar antes de criar uma collection.',
    collectionOptional: 'Collection (opcional)',
    collections: 'Collections',
    context: 'Contexto IA',
    copy: 'Copiar',
    copySnippet: 'Copiar trecho',
    copiedToClipboard: 'Copiado para a area de transferencia.',
    createCollectionStepDesc: 'Organize dados em namespaces logicos',
    createTenantStepDesc: 'Isole seus dados em um tenant identificado',
    dashboard: 'Dashboard',
    data: 'Dados',
    dataExplorer: 'Explorador de Dados',
    dataExplorerNotice: 'Collections sao namespaces logicos. Records sao dados JSON estruturados e projetados para contexto.',
    dataTypes: 'Tipos de dados',
    delete: 'Excluir',
    deleteAll: 'Excluir todos',
    deletePromptConfirm: 'Excluir o prompt "{name}"?',
    deleteTableConfirm: 'Excluir a table "{name}" e todos os records? Esta acao nao pode ser desfeita.',
    detail: 'Detalhes',
    diskBytes: 'Bytes em disco',
    docsGettingStarted: 'Primeiros passos',
    docsGuides: 'Guias',
    docsLead: 'Referencia completa do Hippocore DB: conceitos, guias, API e arquitetura.',
    docsNotFound: 'Nao encontrado',
    docsReference: 'Referencia',
    docsSectionNotFound: 'Secao nao encontrada.',
    dropFiles: 'Arraste arquivos aqui ou clique para selecionar',
    editPrompt: 'Editar prompt',
    emptyDefault: 'Nenhum dado para exibir.',
    errorMessage: 'Erro: {message}',
    fileCountChunks: 'chunks',
    fileCountRows: 'linhas',
    filesLead: 'Envie PDF, CSV, TXT, MD ou JSON. O Hippocore armazena cada arquivo como documentos pesquisaveis ou records estruturados.',
    generateLabels: 'Gerar labels',
    graphEdges: 'Arestas do grafo',
    graphEmptyDesc: 'Arestas sao criadas por operacoes explicitas da API entre itens de contexto.',
    graphWorkbench: 'Workbench do grafo',
    graphWorkbenchLead: 'A criacao interativa de arestas esta planejada. A travessia multi-hop esta disponivel em POST /tenants/:tid/graph/traverse.',
    healthEndpoint: 'Endpoint de health',
    indexHealth: 'Saude do indice',
    indexedEntries: 'Entradas indexadas',
    ingestContext: 'Ingerir contexto',
    ingestContextStepDesc: 'Armazene memorias, documentos e records',
    ingestItems: 'Ingerir {type}',
    integrations: 'Integracoes',
    integrationsLead: 'Configure o cerebro LLM, exposicao da API e integracao de infraestrutura.',
    isolationScope: 'escopo de isolamento',
    keyLength: 'tamanho',
    llmBrain: 'Cerebro LLM',
    llmBrainLead: 'Configure o modelo de linguagem usado por recall de contexto e pela Chat API.',
    logStream: 'Stream de logs',
    logicalView: 'Visao Logica',
    newPrompt: 'Novo prompt',
    newServiceKeyTitle: 'Nova service key: copie agora, pois ela sera exibida uma unica vez',
    noCollectionSelected: 'Nenhuma collection selecionada',
    noCollections: 'Nenhuma collection',
    noCollectionsYet: 'Nenhuma collection ainda',
    noDataSelected: 'Nenhum dado selecionado',
    noDataSelectedDesc: 'Escolha tenant e collection e depois selecione o tipo de dado.',
    noGraphEdges: 'Nenhuma aresta de grafo',
    noObservabilityData: 'Nenhum dado de observabilidade',
    noItems: 'Nenhum item em {type}',
    noProviders: 'Nenhum provider configurado',
    noProvidersDesc: 'Adicione um provider pelo formulario.',
    noPrompts: 'Nenhum prompt ainda',
    noPromptsDesc: 'Crie um template de prompt para a Chat API.',
    noResults: 'Nenhum resultado',
    noResultsDesc: 'Execute uma query SQL para ver resultados.',
    noTables: 'Nenhuma table ainda',
    noTablesDesc: 'Tables sao criadas ao armazenar um record com nome de table ou ao enviar dados CSV.',
    noTenantSelected: 'Nenhum tenant selecionado',
    noTenants: 'Nenhum tenant ainda',
    objectListLead: '{type} no escopo do tenant ativo.',
    objectListEmpty: 'Ainda nao existem {type} neste tenant. Use Ingestao e Recall para criar o primeiro item.',
    observability: 'Observabilidade',
    offline: 'Offline',
    online: 'Online',
    physicalInfo: 'Informacoes Fisicas',
    physicalInfoNote: 'WAL, snapshot, chunks, auditoria e status de indice sao objetos operacionais, nao tables logicas.',
    ping: 'Testar conexao',
    plannedDetail: 'Planejado',
    promptContentHelp: 'Conteudo: use {{context}} e {{query}} como placeholders',
    defaultPrompt: 'Voce e um assistente de IA util.\n\nContexto:\n{{context}}\n\nResponda a seguinte pergunta:\n{{query}}',
    promptDeleteDone: 'Prompt "{name}" excluido.',
    promptDescriptionOptional: 'Descricao (opcional)',
    promptGlobalTenant: 'Tenant id (vazio para global)',
    promptIdOptional: 'Prompt id (opcional)',
    promptNameContentRequired: 'Nome e conteudo sao obrigatorios.',
    promptLibrary: 'Biblioteca de prompts',
    promptSaved: 'Prompt "{name}" salvo.',
    prompts: 'Prompts',
    promptsLead: 'Templates reutilizaveis de system prompt para o pipeline RAG para LLM.',
    providerApiKeyOptional: 'API key (opcional)',
    providerBaseUrl: 'URL base',
    providerId: 'Id do provider',
    providerIdRequired: 'Informe o id do provider para testar.',
    providerKind: 'Tipo (ollama / openrouter / openai)',
    providerModel: 'Modelo',
    providerPinged: 'Provider "{id}" testado.',
    providerSaved: 'Configuracao do provider salva.',
    providerSavedToast: 'Provider salvo.',
    providerValidated: 'Provider validado localmente.',
    question: 'Pergunta',
    questionPlaceholder: 'Pergunte qualquer coisa...',
    questionRequired: 'Digite uma pergunta.',
    quickActions: 'Acoes rapidas',
    recallAndContext: 'Recall e Contexto',
    rawStats: 'Stats brutos',
    response: 'Resposta',
    runRecallStepDesc: 'Consulte seu contexto com busca hibrida',
    runtimeConfiguration: 'Configuracao de runtime',
    runtimeConfigurationLead: 'Runtime de embeddings, reindex, ajuste de hybrid_alpha e configuracoes ricas de tables estao planejados.',
    selectTenantFirst: 'Selecione um tenant primeiro',
    selectTenantTopbar: 'Selecione um tenant no topbar primeiro.',
    searchCollections: 'Buscar collections',
    send: 'Enviar',
    serviceApiKeyLead: 'A service API key autentica endpoints de tenants e pode ser rotacionada a qualquer momento.',
    serviceKeys: 'Chaves de Servico',
    serviceKeyRotated: 'Service API key rotacionada.',
    serviceKeyRotatedToast: 'Chave rotacionada. Copie no drawer de detalhes.',
    settings: 'Configuracoes',
    settingsLead: 'Preferencias da interface e configuracoes futuras de runtime.',
    settingsLanguageLead: 'Altere o idioma pelo topbar. Preferencias de runtime serao adicionadas aqui.',
    signedIn: 'Login realizado com sucesso.',
    snapshotDetails: 'Detalhes do snapshot',
    structuredLogStream: 'Stream de logs estruturados',
    sqlEditor: 'Editor SQL',
    sqlErrorMessage: 'Erro SQL: {message}',
    sqlQuery: 'Query SQL',
    sqlReturned: 'SQL retornou {count} linha(s).',
    sqlSupportNotice: 'Sintaxe suportada: SELECT * FROM table com filtros de igualdade e LIMIT.',
    storeFirstRecord: 'Armazenar record',
    storedFiles: 'Arquivos armazenados',
    tableName: 'Nome da table',
    tableDeleted: 'Table "{name}" excluida.',
    tableResult: 'Tabela',
    tableRecords: 'Records',
    tables: 'Tables',
    tablesAboutLead: 'Tables sao namespaces logicos dentro de uma collection para Records estruturados. Sao criadas com o primeiro record que usa aquele nome.',
    tablesDeleteNotice: 'Excluir uma table remove seus records permanentemente. Collections, Memories e Documents nao sao afetados.',
    tablesLead: 'Namespaces logicos dentro de uma collection que agrupam records estruturados.',
    tablesQueryNotice: 'Consulte tables no Editor SQL com: SELECT * FROM table_name LIMIT 10',
    tenantCreated: 'Tenant "{id}" criado. Agora crie uma collection.',
    tenantCreatedNext: 'Tenant "{id}" criado. Proximo passo: crie uma collection.',
    tenantIdRequired: 'O id do tenant e obrigatorio.',
    tenantRequired: 'Tenant e obrigatorio.',
    tenantLead: 'Tenants sao o limite superior de isolamento para dados, memorias e queries.',
    tenantPurpose: 'Um tenant isola dados, collections, queries e API keys. Crie um por cliente, ambiente ou caso de uso.',
    testChat: 'Testar chat (RAG para LLM)',
    upload: 'Upload',
    uploadCollectionRequired: 'Collection e obrigatoria. Preencha o campo acima da area de upload.',
    uploadComplete: 'Upload de {name} concluido.',
    uploadLastOperation: 'Upload de {name} como {kind}.',
    uploadNetworkError: 'Erro de rede durante o upload.',
    unknown: 'Desconhecido',
    uploadMapping: 'PDF -> Document em chunks | CSV -> Records consultaveis por SQL | array JSON -> Records | objeto JSON, TXT e MD -> Document',
    apiCurlExample: 'exemplo curl: recall',
    composeSnippet: 'Trecho de Docker Compose',
    traefikComment: 'Adicione estas labels ao servico hippocore no docker-compose.yml',
    traefikDomain: 'Dominio (por exemplo, api.example.com)',
    traefikGenerator: 'Gerador de configuracao Traefik',
    traefikPath: 'Prefixo de path (por exemplo, /api)',
    validate: 'Validar',
    walEntries: 'Entradas no WAL',
    workspace: 'Workspace',
    apiStatus: 'Status da API',
    complete: 'Concluido',
    continueSetup: 'Continuar configuracao',
    dataOverview: 'Visao dos dados',
    openObservability: 'Abrir Observabilidade',
    operationalStatus: 'Status operacional',
    ready: 'Pronta',
    setupComplete: 'Workspace pronto',
    setupCompleteLead: 'Os fluxos iniciais de banco, query e contexto de IA estao prontos para uso.',
    setupLead: 'Siga o primeiro passo incompleto para preparar um banco AI-native utilizavel.',
    setupTitle: 'Configuracao do workspace',
    stepAddData: 'Adicionar o primeiro dado',
    stepAddDataDesc: 'Armazene record, memoria, documento ou arquivo',
    stepRunSql: 'Executar uma query SQL',
    stepRunSqlDesc: 'Inspecione records estruturados com a camada SQL-like',
    stepTestRecall: 'Testar recall e contexto',
    stepTestRecallDesc: 'Confirme qual conhecimento o Hippocore recupera para IA',
    welcome: 'Bem-vindo ao Hippocore',
    welcomeDesc: 'Um banco de dados local-first de memoria e contexto para aplicacoes de IA. Comece criando seu primeiro tenant.',
    noTenantDesc: 'Voce ainda nao tem tenants. Um tenant isola dados, colecoes e consultas. Crie o primeiro para comecar.',
    noCollectionDesc: 'Nenhuma collection neste tenant. Collections sao namespaces logicos — crie uma para comecar a armazenar records, memorias e documentos.',
  },
};

/* ─── Documentation content ──────────────────────────────────────────────── */
const DOCS = {
  overview: {
    en: { title: 'Overview', content: `
<p class="lead">Hippocore is a local-first, embedded, AI-native memory and context database written in Rust. It combines vector search, BM25 text search, structured records, and a temporal memory model into a single local database — no cloud dependency, no external vector service.</p>
<h3>Core capabilities</h3>
<ul>
  <li><strong>Hybrid recall</strong> — Reciprocal Rank Fusion (RRF) over vector (cosine) + text (BM25) signals</li>
  <li><strong>Confidence weighting</strong> — memories with explicit confidence scores are boosted or penalised</li>
  <li><strong>Min-score threshold</strong> — filter noise before top-k truncation</li>
  <li><strong>Chunk deduplication</strong> — keep only the best chunk per parent document</li>
  <li><strong>MMR reranking</strong> — Maximal Marginal Relevance for result diversity</li>
  <li><strong>Tenant isolation</strong> — every query is scoped to one tenant; cross-tenant leakage is impossible</li>
  <li><strong>Temporal model</strong> — valid_from / valid_until, supersedure chains, as-of queries</li>
  <li><strong>Graph edges</strong> — typed edges between context items, multi-hop traversal via API</li>
  <li><strong>SQL-like query</strong> — SELECT * FROM table WHERE field = value LIMIT n</li>
  <li><strong>WAL + atomic snapshot</strong> — crash-safe persistence with compaction</li>
</ul>
<h3>Design philosophy</h3>
<ul>
  <li>Local-first — runs entirely on your machine with no external dependencies</li>
  <li>AI-native — purpose-built for RAG, LLM context assembly, and agent memory</li>
  <li>Correctness over cleverness — deterministic embedder, typed errors, no silent failures</li>
  <li>Composable — REST API + CLI + embedded library, all sharing the same core</li>
</ul>` },
    pt: { title: 'Visao Geral', content: `
<p class="lead">Hippocore é um banco de dados local-first, embarcado e nativo para IA escrito em Rust. Combina busca vetorial, BM25, records estruturados e um modelo temporal de memorias em um único banco local.</p>
<h3>Capacidades principais</h3>
<ul>
  <li><strong>Recall hibrido</strong> — Reciprocal Rank Fusion (RRF) sobre vetores (cosseno) + texto (BM25)</li>
  <li><strong>Confidence weighting</strong> — memorias com score de confianca sao impulsionadas ou penalizadas</li>
  <li><strong>Min-score threshold</strong> — filtra ruido antes de top-k</li>
  <li><strong>Chunk deduplication</strong> — mantém apenas o melhor chunk por documento pai</li>
  <li><strong>MMR reranking</strong> — Maximal Marginal Relevance para diversidade de resultados</li>
  <li><strong>Isolamento de tenant</strong> — toda query é escopada por tenant; vazamento é impossivel</li>
  <li><strong>Modelo temporal</strong> — valid_from / valid_until, cadeias de supersedure, consultas as-of</li>
</ul>` },
  },
  quickstart: {
    en: { title: 'Quickstart', content: `
<h3>1. Start the server</h3>
<pre>hippocore serve --db ./data --admin-password mypassword</pre>
<h3>2. Log into this Control Plane</h3>
<p>Open <code>http://localhost:8080/admin</code> and sign in with your admin credentials.</p>
<h3>3. Create a tenant</h3>
<p>Go to <strong>Tenants</strong> in the sidebar, fill in an id and name, click Save. The tenant you create isolates all data from other tenants.</p>
<h3>4. Create a collection</h3>
<p>Go to <strong>Collections</strong>, select your tenant in the topbar, and create a collection. Collections are logical namespaces for your data.</p>
<h3>5. Store a memory</h3>
<p>Go to <strong>Ingestion & Recall</strong>, choose <em>Memory</em>, fill in your tenant, collection, and content, click Store memory.</p>
<h3>6. Run recall</h3>
<p>Still in <strong>Ingestion & Recall</strong>, enter a query in the Recall section and click Run recall. You'll see ranked results from your stored context.</p>
<h3>CLI quickstart</h3>
<pre>hippocore put-memory --db ./data --tenant acme --collection support --text "PostgreSQL uses MVCC for concurrency."
hippocore recall --db ./data --tenant acme --query "how does postgres handle concurrency" --top-k 5</pre>` },
    pt: { title: 'Quickstart', content: `
<h3>1. Iniciar o servidor</h3>
<pre>hippocore serve --db ./data --admin-password minhasenha</pre>
<h3>2. Fazer login no Control Plane</h3>
<p>Abra <code>http://localhost:8080/admin</code> e entre com suas credenciais admin.</p>
<h3>3. Criar um tenant</h3>
<p>Va em <strong>Tenants</strong> no sidebar, preencha id e nome, clique em Salvar.</p>
<h3>4. Criar uma collection</h3>
<p>Va em <strong>Collections</strong>, selecione seu tenant no topbar, e crie uma collection.</p>
<h3>5. Armazenar uma memoria</h3>
<p>Va em <strong>Ingestion & Recall</strong>, escolha <em>Memory</em>, preencha tenant, collection e conteudo, clique em Salvar memoria.</p>
<h3>6. Executar recall</h3>
<p>Ainda em <strong>Ingestion & Recall</strong>, insira uma consulta e clique em Executar recall. Voce vera resultados ranqueados do seu contexto.</p>` },
  },
  concepts: {
    en: { title: 'Concepts', content: `
<h3>Tenant</h3>
<p>The top-level isolation boundary. Every query, record, memory, document, file, and graph edge belongs to exactly one tenant. Cross-tenant data access is impossible by design.</p>
<h3>Collection</h3>
<p>A logical namespace inside a tenant. Think of it as a folder or schema. Records, memories, documents, and files all live under a collection. You can use collections to separate different products, customers, or data categories.</p>
<h3>Record</h3>
<p>Structured JSON data stored in a named table inside a collection. Queryable via the SQL-like interface. Records are the relational-style layer of Hippocore.</p>
<h3>Memory</h3>
<p>A semantic fact or knowledge unit with a text body and an optional confidence score. Memories are embedded and indexed for hybrid recall. They support temporal validity (valid_from / valid_until) and supersedure chains to model evolving knowledge.</p>
<h3>Document</h3>
<p>A longer piece of content (paragraph, article, report) that is automatically chunked into overlapping segments. Each chunk is embedded and stored as a DocumentChunk for recall. The parent document aggregates all chunks.</p>
<h3>File</h3>
<p>A binary or text file ingested from disk. PDFs, source code files, and text files are parsed and chunked automatically. A FileObject tracks the original file metadata while the content becomes searchable chunks.</p>
<h3>Chunk</h3>
<p>A segment derived from a Document or File. The unit of retrieval — each chunk has its own embedding and is scored independently during recall. Chunk deduplication (dedup_chunks) can collapse multiple chunks from the same parent into one result.</p>
<h3>Recall</h3>
<p>The process of querying the database for the most relevant context items given a natural-language query. Hippocore supports vector-only, text-only, and hybrid (RRF) recall modes with optional confidence weighting, min-score filter, chunk deduplication, and MMR reranking.</p>
<h3>Build Context</h3>
<p>Assembles a token-budget-aware context block from recall results, ready to be injected into an LLM prompt. Returns a formatted string with source attribution and stays within the requested token budget.</p>
<h3>Graph Edge</h3>
<p>A typed, directed relationship between two context items (memories, chunks, records). Enables multi-hop traversal and graph-aware context assembly.</p>
<h3>API Key</h3>
<p>The service API key authenticates calls to the tenant-facing REST API (not the admin API). Rotate it from Service Keys without restarting the server.</p>` },
    pt: { title: 'Conceitos', content: `
<h3>Tenant</h3>
<p>Fronteira de isolamento de topo. Todo dado, memoria, documento e query pertence a exatamente um tenant. Acesso cross-tenant é impossivel por design.</p>
<h3>Collection</h3>
<p>Namespace logico dentro de um tenant. Records, memorias, documentos e arquivos vivem dentro de uma collection.</p>
<h3>Record</h3>
<p>Dado JSON estruturado armazenado em uma tabela nomeada dentro de uma collection. Consultavel via SQL-like interface.</p>
<h3>Memory</h3>
<p>Um fato semantico com texto e score de confianca opcional. Memorias sao embarcadas e indexadas para recall hibrido, com validade temporal e cadeias de supersedure.</p>
<h3>Document</h3>
<p>Conteudo mais longo (paragrafo, artigo) automaticamente dividido em chunks sobrepostos. Cada chunk é embarcado e indexado para recall.</p>
<h3>Recall</h3>
<p>Processo de consultar o banco para os itens de contexto mais relevantes dado uma query em linguagem natural. Suporta modos vector-only, text-only e hybrid (RRF).</p>` },
  },
  'data-model': {
    en: { title: 'Data Model', content: `
<h3>Storage layers</h3>
<p>Hippocore has two physically separate layers:</p>
<ul>
  <li><strong>WAL + Snapshot</strong> — all mutations go through a JSON-lines write-ahead log. A background compaction process writes atomic snapshots. Recovery loads the snapshot then replays any WAL entries written after it.</li>
  <li><strong>In-memory index</strong> — a flat vector store + inverted text index rebuilt from the snapshot + WAL at startup. All reads hit the index; writes go through the storage layer then update the index.</li>
</ul>
<h3>On-disk files</h3>
<pre>data_dir/
  wal.log        JSON-lines WAL (one operation per line)
  snapshot.json  Atomic snapshot of the full database state
  meta.json      Database version and config metadata
  audit.log      Audit records for all recall/context calls</pre>
<h3>Object identity</h3>
<p>Every object has a UUID id, a tenant_id, a collection, and timestamps (created_at, updated_at, version). The database manages these — caller-supplied values are overwritten.</p>
<h3>Temporal model</h3>
<p>Memories support <code>valid_from</code> and <code>valid_until</code> (epoch ms). Passing <code>as_of</code> in a recall request filters to only objects valid at that instant. Supersedure chains link updated memories so the old version is still queryable with <code>include_superseded: true</code>.</p>` },
    pt: { title: 'Modelo de Dados', content: `
<h3>Camadas de armazenamento</h3>
<ul>
  <li><strong>WAL + Snapshot</strong> — todas as mutacoes passam por um write-ahead log JSON-lines. Compactacao escreve snapshots atomicos.</li>
  <li><strong>Indice em memoria</strong> — store vetorial plano + indice invertido de texto, reconstruidos na inicializacao.</li>
</ul>
<h3>Arquivos em disco</h3>
<pre>data_dir/
  wal.log        WAL JSON-lines
  snapshot.json  Snapshot atomico do estado completo
  meta.json      Versao e metadados
  audit.log      Registros de auditoria</pre>
<h3>Modelo temporal</h3>
<p>Memorias suportam <code>valid_from</code> e <code>valid_until</code>. Passe <code>as_of</code> no recall para filtrar pelo instante.</p>` },
  },
  'sql-guide': {
    en: { title: 'SQL Guide', content: `
<p class="lead">Hippocore supports a read-only SQL-like query layer over record tables. It is not a full SQL engine — it covers the most common analytical patterns without the complexity of a full query planner.</p>
<h3>Supported syntax</h3>
<pre>SELECT * FROM table_name [WHERE field = 'value'] [LIMIT n]</pre>
<h3>Examples</h3>
<pre>-- All records in the "systems" table
select * from systems limit 10

-- Filter by a field value
select * from systems where engine = 'postgresql' limit 5

-- Filter by a nested payload field
select * from records where payload.engine = 'postgresql'</pre>
<h3>Notes</h3>
<ul>
  <li>Only <code>SELECT *</code> is supported — specific column selection is not yet implemented.</li>
  <li>Equality filters only — range queries are not supported.</li>
  <li>The tenant is always the currently selected tenant in the Control Plane (or <code>tenant_id</code> in the API body).</li>
  <li>Table names correspond to the <code>table</code> field set when storing a record.</li>
  <li>Records are stored as JSON payloads; fields referenced in WHERE are matched against top-level payload keys.</li>
</ul>
<h3>HTTP API</h3>
<pre>POST /admin/sql
{
  "tenant_id": "acme",
  "sql": "select * from systems limit 5"
}</pre>` },
    pt: { title: 'Guia SQL', content: `
<p class="lead">Hippocore suporta uma camada read-only de query estilo SQL sobre tabelas de records.</p>
<h3>Sintaxe suportada</h3>
<pre>SELECT * FROM tabela [WHERE campo = 'valor'] [LIMIT n]</pre>
<h3>Exemplos</h3>
<pre>select * from systems limit 10
select * from systems where engine = 'postgresql' limit 5</pre>
<h3>Notas</h3>
<ul>
  <li>Apenas <code>SELECT *</code> — selecao de colunas nao implementada.</li>
  <li>Apenas filtros de igualdade.</li>
  <li>O tenant é sempre o tenant selecionado no Control Plane.</li>
</ul>` },
  },
  'rag-guide': {
    en: { title: 'RAG / Context Guide', content: `
<p class="lead">Hippocore is designed as a context database for Retrieval-Augmented Generation (RAG). This guide explains the full retrieval pipeline.</p>
<h3>Recall pipeline</h3>
<ol>
  <li><strong>Embed the query</strong> — the query text is embedded using the built-in deterministic embedder (or a caller-supplied vector).</li>
  <li><strong>Score candidates</strong> — in Hybrid mode, cosine similarity (vector) and BM25 (text) scores are computed for all tenant-scoped candidates. In Vector or Text mode, only one signal is used.</li>
  <li><strong>RRF fusion</strong> — Reciprocal Rank Fusion combines the vector and text rank lists into a single fused score.</li>
  <li><strong>Confidence weighting</strong> — Memory items with an explicit confidence score get a proportional boost or penalty.</li>
  <li><strong>Sort</strong> — results are sorted by final score descending.</li>
  <li><strong>Min-score filter</strong> — results below <code>min_score</code> are dropped (if set).</li>
  <li><strong>Chunk deduplication</strong> — if <code>dedup_chunks: true</code>, only the best chunk per parent document survives.</li>
  <li><strong>MMR reranking</strong> — if <code>mmr: true</code>, results are reranked to maximise relevance + diversity.</li>
  <li><strong>Top-k truncation</strong> — final list is trimmed to <code>top_k</code>.</li>
</ol>
<h3>RecallResult fields</h3>
<ul>
  <li><code>score</code> — final combined score used for ranking</li>
  <li><code>vector_score</code> — raw cosine similarity component</li>
  <li><code>text_score</code> — raw BM25 component</li>
  <li><code>reason</code> — human-readable explanation of the score composition</li>
  <li><code>matched_terms</code> — query tokens that appeared in the result text</li>
  <li><code>confidence</code> — underlying memory confidence, if rated</li>
</ul>
<h3>Build context</h3>
<p>After recall, <code>build_context</code> assembles a prompt-ready string within a token budget:</p>
<pre>POST /admin/tenants/:tid/context
{
  "query": "how does postgres handle concurrency?",
  "max_tokens": 2048,
  "collection": "support",
  "include_related": true
}</pre>` },
    pt: { title: 'Guia RAG / Contexto', content: `
<p class="lead">Hippocore é um banco de contexto para RAG. O pipeline de retrieval inclui: embed, score (vetor + BM25), fusao RRF, confidence weighting, min-score filter, chunk dedup, MMR reranking e top-k.</p>
<h3>Campos do RecallResult</h3>
<ul>
  <li><code>score</code> — score final usado para ranking</li>
  <li><code>vector_score</code> — componente cosseno</li>
  <li><code>text_score</code> — componente BM25</li>
  <li><code>reason</code> — explicacao legivel da composicao do score</li>
  <li><code>matched_terms</code> — tokens da query que apareceram no texto do resultado</li>
</ul>` },
  },
  'api-auth': {
    en: { title: 'API Authentication', content: `
<h3>Admin API</h3>
<p>All <code>/admin/*</code> endpoints require a session token passed as the <code>x-admin-session</code> header. Obtain a session via <code>POST /admin/login</code>.</p>
<pre>curl -X POST http://localhost:8080/admin/login \\
  -H "Content-Type: application/json" \\
  -d '{"username": "admin", "password": "yourpassword"}'

# Response: {"session": "SESSION_TOKEN_HERE"}

# Use the session in subsequent requests:
curl http://localhost:8080/admin/bootstrap \\
  -H "x-admin-session: SESSION_TOKEN_HERE"</pre>
<h3>Service API</h3>
<p>Tenant-facing endpoints (e.g. <code>/tenants/:tid/recall</code>) use a service API key in the <code>X-Api-Key</code> header. Rotate the key from the Service Keys page.</p>
<pre>curl -X POST http://localhost:8080/tenants/acme/recall \\
  -H "X-Api-Key: YOUR_SERVICE_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{"query": "postgresql concurrency", "top_k": 5}'</pre>
<h3>Security notes</h3>
<ul>
  <li>The admin API is designed for local/trusted use only — do not expose it publicly.</li>
  <li>The service API key is opaque and should be treated as a secret.</li>
  <li>Sessions expire on server restart (in-memory only).</li>
</ul>` },
    pt: { title: 'Autenticacao de API', content: `
<h3>Admin API</h3>
<p>Todos os endpoints <code>/admin/*</code> exigem o header <code>x-admin-session</code>. Obtenha uma sessao via <code>POST /admin/login</code>.</p>
<h3>Service API</h3>
<p>Endpoints de tenant usam a API key de servico no header <code>X-Api-Key</code>. Rotacione a chave na pagina Service Keys.</p>` },
  },
  'cli-guide': {
    en: { title: 'CLI Guide', content: `
<p class="lead">The <code>hippocore</code> CLI provides full access to the database without running a server.</p>
<h3>Core commands</h3>
<pre>hippocore --help
hippocore stats --db ./data
hippocore inspect --db ./data [--tenant acme]

# Tenants
hippocore tenants list --db ./data
hippocore tenants create --db ./data --id acme --name "Acme Corp"

# Collections
hippocore collections list --db ./data --tenant acme
hippocore collections create --db ./data --tenant acme --name support --description "Customer support"

# Memories
hippocore put-memory --db ./data --tenant acme --collection support --text "fact here"
hippocore recall --db ./data --tenant acme --query "your query" --top-k 5

# Advanced recall flags
hippocore recall --db ./data --tenant acme --query "query" \\
  --min-score 0.3 \\
  --dedup-chunks \\
  --mmr --mmr-lambda 0.6

# Server
hippocore serve --db ./data --admin-password secret --port 8080</pre>
<h3>Output format</h3>
<p>Most commands support <code>--json</code> for machine-readable output.</p>` },
    pt: { title: 'Guia CLI', content: `
<p class="lead">O CLI <code>hippocore</code> fornece acesso completo ao banco sem servidor.</p>
<pre>hippocore tenants list --db ./data
hippocore tenants create --db ./data --id acme --name "Acme"
hippocore collections list --db ./data --tenant acme
hippocore put-memory --db ./data --tenant acme --collection support --text "fato"
hippocore recall --db ./data --tenant acme --query "consulta" --top-k 5
hippocore recall --db ./data --tenant acme --query "consulta" \\
  --min-score 0.3 --dedup-chunks --mmr</pre>` },
  },
  architecture: {
    en: { title: 'Architecture', content: `
<h3>Module boundaries</h3>
<ul>
  <li><code>config / errors / model</code> — configuration, typed errors, entity types</li>
  <li><code>storage</code> — WAL + atomic snapshot, recovery, compaction. Knows operations and bytes, not scoring.</li>
  <li><code>memory</code> — deterministic embedder + chunker/tokenizer. Never panics.</li>
  <li><code>index</code> — in-memory vector store + inverted text index. Cosine + TF-IDF. No disk access.</li>
  <li><code>query</code> — Filter, SearchMode, scoring/normalization, hybrid RRF fusion. Tenant isolation enforced here.</li>
  <li><code>lib.rs (Hippocore)</code> — the only orchestrator of storage + index. Public API.</li>
  <li><code>cli</code> — clap parser + handlers.</li>
  <li><code>hippocore-server</code> — axum HTTP server. Admin + service API.</li>
</ul>
<h3>Key invariants</h3>
<ul>
  <li><code>unsafe</code> is <code>forbid</code>den in all crates — no unsafe Rust.</li>
  <li>Tenant isolation is enforced at the query layer — it cannot be bypassed by callers.</li>
  <li>Pure functions (cosine similarity, tokenizer) never panic on bad input.</li>
  <li>All public items have doc comments; typed errors cover all failure modes.</li>
</ul>
<h3>Write path</h3>
<pre>caller → Hippocore::put_memory() → validate → WAL.append(op) → index.insert(entry) → return id</pre>
<h3>Read path</h3>
<pre>caller → Hippocore::recall() → QueryRequest → query::execute(index, embed, req) → Vec&lt;RecallResult&gt;</pre>` },
    pt: { title: 'Arquitetura', content: `
<h3>Fronteiras de modulo</h3>
<ul>
  <li><code>storage</code> — WAL + snapshot atomico, recovery, compactacao</li>
  <li><code>memory</code> — embedder deterministico + chunker</li>
  <li><code>index</code> — store vetorial + indice invertido em memoria</li>
  <li><code>query</code> — Filter, SearchMode, fusao RRF. Isolamento de tenant aplicado aqui.</li>
  <li><code>lib.rs</code> — orquestrador. API publica.</li>
</ul>
<h3>Invariantes</h3>
<ul>
  <li><code>unsafe</code> é proibido em todos os crates.</li>
  <li>Isolamento de tenant é aplicado na camada de query.</li>
</ul>` },
  },
};

const DOCS_NAV = [
  ['start',    'docsGettingStarted', ['overview','quickstart']],
  ['concepts', 'concepts',          ['concepts','data-model']],
  ['guides',   'docsGuides',         ['sql-guide','rag-guide','cli-guide']],
  ['reference','docsReference',      ['api-auth','architecture']],
];

const DOCS_LABELS = {
  overview:     { en: 'Overview',           pt: 'Visao Geral' },
  quickstart:   { en: 'Quickstart',         pt: 'Quickstart'  },
  concepts:     { en: 'Concepts',           pt: 'Conceitos'   },
  'data-model': { en: 'Data Model',         pt: 'Modelo de Dados' },
  'sql-guide':  { en: 'SQL Guide',          pt: 'Guia SQL'    },
  'rag-guide':  { en: 'RAG / Context Guide',pt: 'Guia RAG'    },
  'api-auth':   { en: 'API Authentication', pt: 'Autenticacao API' },
  'cli-guide':  { en: 'CLI Guide',          pt: 'Guia CLI'    },
  architecture: { en: 'Architecture',       pt: 'Arquitetura' },
};

/* ─── State ──────────────────────────────────────────────────────────────── */
const state = {
  session:    localStorage.getItem('hippocore.adminSession') || '',
  username:   localStorage.getItem('hippocore.adminUser') || 'admin',
  lang:       localStorage.getItem('hippocore.lang') || 'en',
  page:       localStorage.getItem('hippocore.page') || 'dashboard',
  tenant:     localStorage.getItem('hippocore.tenant') || '',
  collection: localStorage.getItem('hippocore.collection') || '',
  tab:        'logical',
  sqlTab:     'table',
  explorerKind: 'records',
  explorerSearch: '',
  recallTab:  'results',
  recallKind: 'recall',
  recallQuery:'',
  ingestType: 'Memory',
  ingestDraft:{ collection: '', table: 'data', content: '' },
  docSection: 'overview',
  bootstrap:  null,
  collections:[],
  lists:      {},
  providers:  [],
  providerResult: null,
  sqlResult:  null,
  sqlError:   '',
  detail:     null,
  toast:      '',
  lastOperation: '',
  uploadProgress: 0,
  uploadResult: null,
  apiInfo: null,
  traefikDomain: '',
  traefikPath: '/api',
  traefikConfig: '',
  editingPrompt: null,
  chatResult: '',
};

/* ─── Helpers ────────────────────────────────────────────────────────────── */
function t(key) {
  return (I18N[state.lang] && I18N[state.lang][key]) || I18N.en[key] || key;
}

function tf(key, values = {}) {
  return Object.entries(values).reduce(
    (message, [name, value]) => message.replaceAll(`{${name}}`, String(value)),
    t(key),
  );
}

function message(key, values = {}) {
  return { key, values };
}

function messageText(value) {
  return value && typeof value === 'object' && value.key
    ? tf(value.key, value.values)
    : String(value || '');
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
    try { body = JSON.parse(text); } catch (_) { body = text; }
  }
  if (!res.ok) throw new Error((body && body.error) || text || `HTTP ${res.status}`);
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
  setTimeout(() => { state.toast = ''; render(); }, 2800);
}

function showError(err) {
  state.lastOperation = message('errorMessage', { message: err.message || err });
  showToast(state.lastOperation);
}

function captureFormState() {
  return [...root.querySelectorAll('input[id], textarea[id], select[id]')]
    .filter((field) => field.type !== 'file' && !['languageSelect', 'loginLang', 'settingsLanguage'].includes(field.id))
    .map((field) => ({ id: field.id, type: field.type, value: field.value, checked: field.checked }));
}

function restoreFormState(fields) {
  fields.forEach((saved) => {
    const field = document.getElementById(saved.id);
    if (!field) return;
    if (saved.type === 'checkbox' || saved.type === 'radio') field.checked = saved.checked;
    else field.value = saved.value;
  });
}

/* ─── Base Components ────────────────────────────────────────────────────── */
function StatusBadge(label, tone = 'info') {
  return `<span class="status-badge ${tone}">${esc(label)}</span>`;
}

function ActionButton(label, action, tone = '') {
  return `<button class="${tone}" data-action="${esc(action)}" type="button">${esc(label)}</button>`;
}

function StatCard(label, value, note = '') {
  return `<div class="stat"><span>${esc(label)}</span><strong>${esc(value)}</strong>${note ? `<span>${esc(note)}</span>` : ''}</div>`;
}

function EmptyState(title, message, actions = '') {
  return `<div class="empty-state"><h3>${esc(title)}</h3><p>${esc(message)}</p>${actions ? `<div class="page-actions">${actions}</div>` : ''}</div>`;
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

function CollectionDatalist(id) {
  const names = [...new Set((state.collections || []).map((collection) => collection.name).filter(Boolean))];
  return `<datalist id="${esc(id)}">${names.map((name) => `<option value="${esc(name)}"></option>`).join('')}</datalist>`;
}

function Tabs(tabs, active, action) {
  return `<div class="tabs">${tabs.map((tab) => `<button type="button" data-action="${action}" data-tab="${esc(tab.id)}" aria-selected="${tab.id === active}">${esc(tab.label)}</button>`).join('')}</div>`;
}

function DataTable(rows, columns, emptyTitle = t('empty'), emptyDesc = '', emptyAction = '') {
  if (!rows || rows.length === 0) {
    return EmptyState(emptyTitle, emptyDesc || t('emptyDefault'), emptyAction);
  }
  return `<div class="table-wrap"><table><thead><tr>${columns.map((c) => `<th>${esc(c.label)}</th>`).join('')}<th></th></tr></thead><tbody>${rows.map((row, idx) => `<tr>${columns.map((c) => `<td>${esc(c.render ? c.render(row) : row[c.key])}</td>`).join('')}<td><button class="ghost" type="button" data-action="detail" data-list="${esc(row.__list || '')}" data-index="${idx}">${esc(t('detail'))}</button></td></tr>`).join('')}</tbody></table></div>`;
}

function PageHeader(title, lead, actions = '') {
  return `<div class="page-header"><div><h2>${esc(title)}</h2><p>${esc(lead)}</p></div>${actions ? `<div class="page-actions">${actions}</div>` : ''}</div>`;
}

function DetailDrawer() {
  if (!state.detail) return '';
  const title = state.detail.titleKey ? t(state.detail.titleKey) : state.detail.title;
  return `<div class="drawer-backdrop" data-action="close-detail"></div><aside class="drawer"><div class="drawer-head"><h3>${esc(title)}</h3><button class="icon" data-action="close-detail" type="button">✕</button></div><div class="drawer-body">${JsonViewer(state.detail.value)}</div></aside>`;
}

function RecallCards(items) {
  if (!Array.isArray(items) || items.length === 0) {
    return EmptyState(t('noRecallResults'), t('noRecallResultsDesc'));
  }
  return `<div class="recall-results">${items.map((item) => {
    const score = Number.isFinite(Number(item.score)) ? Number(item.score).toFixed(3) : '—';
    const terms = Array.isArray(item.matched_terms) ? item.matched_terms : [];
    return `<article class="recall-card">
      <div class="recall-card-head">
        <div class="recall-card-badges">${StatusBadge(item.kind || t('unknown'))} ${StatusBadge(`score ${score}`, 'ok')}</div>
        <button class="ghost" type="button" data-action="copy-code" data-code="${esc(item.id)}">${esc(t('copyId'))}</button>
      </div>
      <p>${esc(item.text || '')}</p>
      ${terms.length > 0 ? `<div class="matched-terms">${terms.map((term) => `<span>${esc(term)}</span>`).join('')}</div>` : ''}
      <code class="result-id">${esc(item.id)}</code>
    </article>`;
  }).join('')}</div>`;
}

function ContextResult(result) {
  if (!result || typeof result !== 'object') {
    return EmptyState(t('contextResult'), t('recallReady'));
  }
  return `<div class="context-result">
    <div class="context-metrics">
      ${StatCard(t('tokens'), result.token_count ?? 0)}
      ${StatCard(t('itemsIncluded'), result.items_included ?? 0)}
      ${StatCard(t('itemsDropped'), result.items_dropped ?? 0)}
    </div>
    <pre class="context-text">${esc(result.text || '')}</pre>
  </div>`;
}

function RecallResultView() {
  const result = state.lists.recallResult;
  const tabs = Tabs([
    { id: 'results', label: state.recallKind === 'context' ? t('contextResult') : t('recallResults') },
    { id: 'json', label: t('rawJson') },
  ], state.recallTab, 'recall-tab');
  if (!result) {
    return `${tabs}${EmptyState(t('recallResults'), t('recallReady'))}`;
  }
  const content = state.recallTab === 'json'
    ? JsonViewer(result)
    : state.recallKind === 'context'
      ? ContextResult(result)
      : RecallCards(result);
  return `${tabs}${content}`;
}

/* ─── App Shell ──────────────────────────────────────────────────────────── */
function Sidebar() {
  let currentGroup = '';
  const nav = NAV.map(([group, labelKey, id]) => {
    const groupLabel = group !== currentGroup
      ? `<div class="group-label">${esc(t(GROUP_LABELS[group]))}</div>`
      : '';
    currentGroup = group;
    return `${groupLabel}<button type="button" data-route="${esc(id)}" aria-current="${state.page === id ? 'page' : 'false'}">${esc(t(labelKey))}</button>`;
  }).join('');

  const tenantCount = state.bootstrap?.tenants?.length ?? 0;
  const footerBadge = state.session
    ? StatusBadge(t('active'), 'ok')
    : StatusBadge(t('signedOut'), 'error');
  const next = setupProgress().next;
  const setupAction = next
    ? `<button class="sidebar-next" type="button" data-action="${esc(next.action)}"><span>${esc(t('continueSetup'))}</span><strong>${esc(next.title)}</strong></button>`
    : '';

  return `<aside class="sidebar"><div class="brand"><div class="mark" aria-hidden="true"></div><div><h1>${esc(t('appTitle'))}</h1><p>${esc(t('appSubtitle'))}</p></div></div><nav class="nav">${nav}</nav>${setupAction}<div class="sidebar-footer">${footerBadge}<span class="muted">${esc(state.bootstrap?.config?.data_dir || './hippocore-data')}</span></div></aside>`;
}

function Topbar() {
  const tenants = state.bootstrap?.tenants || [];
  const tenantOptions = [`<option value="">${esc(t('noTenant'))}</option>`].concat(
    tenants.map((t2) => `<option value="${esc(t2.id)}" ${t2.id === state.tenant ? 'selected' : ''}>${esc(t2.id)} — ${esc(t2.name)}</option>`),
  ).join('');

  const tenantBadge = state.tenant
    ? StatusBadge(`${t('currentTenant')}: ${state.tenant}`, 'ok')
    : StatusBadge(t('noTenant'), 'planned');

  return `<header class="topbar">
    <div>
      <div class="breadcrumbs">Hippocore / ${esc(pageTitle())}</div>
      <div class="kpi-line">${tenantBadge}${state.session ? StatusBadge(t('active'), 'ok') : StatusBadge(t('signedOut'), 'error')}</div>
    </div>
    <div class="topbar-actions">
      <select class="tenant-select" id="globalTenant">${tenantOptions}</select>
      <select id="languageSelect" aria-label="${esc(t('language'))}">
        <option value="en" ${state.lang === 'en' ? 'selected' : ''}>EN</option>
        <option value="pt" ${state.lang === 'pt' ? 'selected' : ''}>PT</option>
      </select>
      <button class="icon" type="button" data-action="refresh" title="${esc(t('refresh'))}">↻</button>
      <button type="button" data-action="logout">${esc(t('logout'))}</button>
    </div>
  </header>`;
}

function AppShell(content) {
  return `<div class="app-shell">${Sidebar()}<div class="main">${Topbar()}<main class="content">${content}</main></div>${DetailDrawer()}${state.toast ? `<div class="toast">${esc(messageText(state.toast))}</div>` : ''}</div>`;
}

/* ─── Pages ──────────────────────────────────────────────────────────────── */
function pageTitle() {
  const item = NAV.find(([, , id]) => id === state.page);
  return item ? t(item[1]) : t('dashboard');
}

function stats() {
  return state.bootstrap?.stats || {};
}

function onboardingStorageKey(step) {
  const database = state.bootstrap?.config?.data_dir || 'default';
  return `hippocore.onboarding.${database}.${step}`;
}

function completeOnboardingStep(step) {
  localStorage.setItem(onboardingStorageKey(step), 'true');
}

function setupProgress() {
  const s = stats();
  const dataCount = ['records', 'memories', 'documents', 'files']
    .reduce((total, key) => total + Number(s[key] || 0), 0);
  const steps = [
    { id: 'tenant', title: t('createTenant'), description: t('createTenantStepDesc'), action: 'go-tenants', done: Number(s.tenants || 0) > 0 },
    { id: 'collection', title: t('createCollection'), description: t('createCollectionStepDesc'), action: 'go-collections', done: Number(s.collections || 0) > 0 },
    { id: 'data', title: t('stepAddData'), description: t('stepAddDataDesc'), action: 'go-ingest', done: dataCount > 0 },
    { id: 'sql', title: t('stepRunSql'), description: t('stepRunSqlDesc'), action: 'go-sql', done: localStorage.getItem(onboardingStorageKey('sql')) === 'true' },
    { id: 'recall', title: t('stepTestRecall'), description: t('stepTestRecallDesc'), action: 'go-ingest', done: Number(s.audit_records || 0) > 0 || localStorage.getItem(onboardingStorageKey('recall')) === 'true' },
  ];
  return {
    steps,
    completed: steps.filter((step) => step.done).length,
    next: steps.find((step) => !step.done) || null,
  };
}

function SetupChecklist() {
  const progress = setupProgress();
  const complete = progress.next === null;
  const rows = progress.steps.map((step, index) => {
    const current = progress.next?.id === step.id;
    return `<div class="setup-step" data-state="${step.done ? 'done' : current ? 'current' : 'pending'}">
      <span class="setup-marker">${step.done ? '✓' : index + 1}</span>
      <div><strong>${esc(step.title)}</strong><span>${esc(step.description)}</span></div>
      ${current ? ActionButton(step.title, step.action, 'primary') : step.done ? StatusBadge(t('complete'), 'ok') : ''}
    </div>`;
  }).join('');
  return `<section class="setup-panel">
    <div class="setup-head"><div><h2>${esc(complete ? t('setupComplete') : t('setupTitle'))}</h2><p>${esc(complete ? t('setupCompleteLead') : t('setupLead'))}</p></div>${StatusBadge(`${progress.completed}/${progress.steps.length}`, complete ? 'ok' : 'info')}</div>
    <div class="setup-steps">${rows}</div>
  </section>`;
}

function DashboardPage() {
  const s = stats();
  const operational = `<section><h3 class="section-title">${esc(t('operationalStatus'))}</h3><div class="grid cols-4">
    ${StatCard(t('serverStatus'), state.session ? t('online') : t('offline'), 'GET /health')}
    ${StatCard(t('currentTenant'), state.tenant || '—', t('isolationScope'))}
    ${StatCard(t('dataDir'), state.bootstrap?.config?.data_dir || '—', 'local-first')}
    ${StatCard(t('apiStatus'), state.session ? t('ready') : t('offline'), '/admin/bootstrap')}
  </div></section>`;

  const dataCards = `<section><h3 class="section-title">${esc(t('dataOverview'))}</h3><div class="grid cols-4">
    ${StatCard(t('collections'), s.collections ?? 0)}
    ${StatCard(t('records'), s.records ?? '—')}
    ${StatCard(t('memories'), s.memories ?? '—')}
    ${StatCard(t('documents'), s.documents ?? '—')}
    ${StatCard(t('files'), s.files ?? '—')}
    ${StatCard(t('graphEdges'), s.graph_edges ?? '—')}
    ${StatCard(t('auditRecords'), s.audit_records ?? '—', `${s.audit_log_bytes ?? 0} B`)}
    ${StatCard(t('walEntries'), s.wal_entries ?? '—')}
  </div></section>`;

  const quick = [
    [t('createTenant'),    'go-tenants'],
    [t('createCollection'),'go-collections'],
    [t('openSql'),          'go-sql'],
    [t('openDataExplorer'), 'go-explorer'],
    [t('ingestContext'),    'go-ingest'],
    [t('rotateKey'),       'go-keys'],
    [t('openObservability'),'go-observability'],
    [t('documentation'),   'go-docs'],
  ].map(([label, action]) => `<button type="button" data-action="${esc(action)}"><span>${esc(label)}</span><span class="qa-arrow">→</span></button>`).join('');

  const lastOp = state.lastOperation
    ? `<div class="notice" style="margin-top:0">${esc(messageText(state.lastOperation))}</div>`
    : '';

  return `${PageHeader(t('dashboard'), t('dashboardLead'), ActionButton(t('refresh'), 'refresh'))}
    ${SetupChecklist()}
    ${operational}
    ${dataCards}
    <section class="panel"><h3>${esc(t('quickActions'))}</h3><div class="actions-list dashboard-actions">${quick}</div></section>
    ${lastOp}`;
}

function DataExplorerPage() {
  const collections = state.collections || [];
  const kind = activeExplorerListName();
  const rows = annotateRows(activeExplorerRows(), kind);
  const kindLabel = t(EXPLORER_KINDS.find(([id]) => id === kind)?.[1] || kind);
  const tabs = Tabs([
    { id: 'logical',  label: t('logicalView') },
    { id: 'json',     label: t('rawJson') },
    { id: 'physical', label: t('physicalInfo') },
  ], state.tab, 'tab');
  const hasTenant = !!state.tenant;
  const hasCollection = !!state.collection && collections.some((item) => item.name === state.collection);
  const emptyAction = ActionButton(tf('ingestItems', { type: kindLabel }), 'ingest-explorer-kind', 'primary');
  const view = rows.length === 0 && hasCollection
    ? EmptyState(tf('noItems', { type: kindLabel }), tf('objectListEmpty', { type: kindLabel }), emptyAction)
    : state.tab === 'json'
      ? JsonViewer(rows)
      : state.tab === 'physical'
        ? PhysicalInfo(rows, kind)
        : DataTable(rows, explorerLogicalColumns(rows, kind), t('noDataSelected'), t('noDataSelectedDesc'), emptyAction);
  const main = !hasTenant
    ? EmptyState(t('noTenantSelected'), t('noTenantDesc'), ActionButton(t('createTenant'), 'go-tenants', 'primary'))
    : !hasCollection
      ? EmptyState(t('noCollectionSelected'), t('noCollectionDesc'), ActionButton(t('createCollection'), 'go-collections', 'primary'))
      : `<div class="explorer-selection"><div><strong>${esc(state.collection)}</strong><span>${esc(kindLabel)}</span></div>${StatusBadge(String(rows.length), 'info')}</div>${tabs}${view}`;
  return `${PageHeader(t('dataExplorer'), t('dataExplorerLead'), ActionButton(t('refresh'), 'refresh'))}
    <div class="layout-split">
      <aside class="side-panel">
        <div class="explorer-side-head"><strong>${esc(t('tenants'))}</strong>${ActionButton(`+ ${t('collection')}`, 'go-collections')}</div>
        <label class="field"><span>${esc(t('searchCollections'))}</span><input id="explorerCollectionSearch" type="search" value="${esc(state.explorerSearch)}" placeholder="${esc(t('searchCollections'))}" /></label>
        <div class="tree">${tenantTree(collections)}</div>
        <div class="notice">${esc(t('dataExplorerNotice'))}</div>
      </aside>
      <section class="panel">${main}</section>
    </div>`;
}

function tenantTree(collections) {
  const tenants = state.bootstrap?.tenants || [];
  if (tenants.length === 0) {
    return EmptyState(t('noTenants'), t('noTenantDesc'), ActionButton(t('createTenant'), 'go-tenants', 'primary'));
  }
  const search = state.explorerSearch.trim().toLowerCase();
  const matchedCollections = collections.filter((collection) => collection.name.toLowerCase().includes(search));
  return tenants.map((tenant) => {
    const nested = tenant.id === state.tenant
      ? `<div class="nested">${collections.map((collection) => {
          const selected = collection.name === state.collection;
          const hidden = search && !collection.name.toLowerCase().includes(search) ? 'hidden' : '';
          const kinds = selected
            ? `<div class="explorer-kinds"><span class="tree-caption">${esc(t('dataTypes'))}</span>${EXPLORER_KINDS.map(([kind, labelKey]) => `<button type="button" class="tree-kind" data-action="choose-explorer-kind" data-kind="${esc(kind)}" aria-current="${state.explorerKind === kind ? 'page' : 'false'}"><span>${esc(t(labelKey))}</span><span class="tree-count">${state.lists[kind]?.length ?? 0}</span></button>`).join('')}</div>`
            : '';
          return `<div class="tree-collection" data-collection-node="${esc(collection.name.toLowerCase())}" ${hidden}><button type="button" data-action="choose-collection" data-collection="${esc(collection.name)}" aria-current="${selected ? 'page' : 'false'}"><span>${esc(collection.name)}</span></button>${kinds}</div>`;
        }).join()}<span class="muted tree-empty" data-search-empty ${matchedCollections.length > 0 ? 'hidden' : ''}>${esc(t('noCollectionsYet'))}</span></div>`
      : '';
    return `<div class="tree-tenant"><button type="button" data-action="choose-tenant" data-tenant="${esc(tenant.id)}" aria-current="${tenant.id === state.tenant ? 'page' : 'false'}"><span>${esc(tenant.id)}</span><span class="muted">${esc(tenant.name)}</span></button>${nested}</div>`;
  }).join('');
}

function activeExplorerListName() {
  const listPages = ['records','memories','documents','files'];
  return state.page === 'data-explorer'
    ? state.explorerKind
    : listPages.includes(state.page) ? state.page : 'records';
}

function activeExplorerRows() {
  return state.lists[activeExplorerListName()] || [];
}

function annotateRows(rows, list) {
  return (rows || []).map((row) => Object.assign({ __list: list }, row));
}

function logicalColumns(rows) {
  if (!rows || rows.length === 0) return [];
  const keys = ['id','collection','table','name','memory_type','media_type','version']
    .filter((key) => rows.some((row) => row[key] != null));
  return keys.map((key) => ({ key, label: key }));
}

function displayCell(value) {
  if (value === null || value === undefined) return '';
  return typeof value === 'object' ? JSON.stringify(value) : String(value);
}

function explorerLogicalColumns(rows, kind) {
  if (kind === 'records') {
    const payloadKeys = [...new Set(rows.flatMap((row) => Object.keys(row.payload || {})))].sort();
    const base = [
      { key: 'id', label: 'id' },
      { key: 'table', label: 'table' },
    ];
    return base.concat(payloadKeys.map((key) => ({
      key: `payload.${key}`,
      label: ['id', 'table'].includes(key) ? `payload.${key}` : key,
      render: (row) => displayCell(row.payload?.[key]),
    })));
  }
  if (kind === 'memories') {
    return [{key:'id',label:'id'},{key:'memory_type',label:'memory_type'},{key:'text',label:'text'},{key:'confidence',label:'confidence'}];
  }
  if (kind === 'documents') {
    return [{key:'id',label:'id'},{key:'text',label:'text'},{key:'version',label:'version'}];
  }
  return [{key:'id',label:'id'},{key:'name',label:'name'},{key:'media_type',label:'media_type'},{key:'size_bytes',label:'size_bytes'}];
}

function PhysicalInfo(rows, kind) {
  const columns = [
    { key: 'id', label: 'id' },
    { key: '__kind', label: 'kind', render: () => kind },
    { key: 'lineage', label: 'lineage', render: (row) => displayCell(row.document_id || row.source || row.path || '') },
    { key: 'source', label: 'source', render: (row) => displayCell(row.source) },
    { key: 'metadata', label: 'metadata', render: (row) => displayCell(row.metadata) },
    { key: 'created_at', label: 'created_at' },
    { key: 'updated_at', label: 'updated_at' },
    { key: 'version', label: 'version' },
  ];
  return `<div class="notice explorer-physical-note">${esc(t('physicalInfoNote'))}</div>${DataTable(rows, columns, t('noDataSelected'), t('noDataSelectedDesc'))}`;
}

function SqlEditorPage() {
  const examples = [
    'select * from systems limit 10',
    "select * from systems where engine = 'postgresql' limit 5",
    "select * from records where payload.engine = 'postgresql'",
  ];
  const result = state.sqlError
    ? `<pre class="result-error">${esc(state.sqlError)}</pre>`
    : state.sqlTab === 'json'
      ? JsonViewer(state.sqlResult)
      : DataTable(annotateRows(state.sqlResult?.rows || [], 'sql'), logicalColumns(state.sqlResult?.rows || []), t('noResults'), t('noResultsDesc'));
  return `${PageHeader(t('sqlEditor'), t('sqlLead'), ActionButton(t('run'), 'run-sql', 'primary'))}
    <section class="sql-editor">
      <div class="panel">
        <label class="field">
          <span>${esc(t('sqlQuery'))}</span>
          <textarea id="sqlInput" style="font-family:monospace;font-size:13px">select * from systems limit 10</textarea>
        </label>
        <div class="page-actions" style="margin-top:10px">
          ${ActionButton(t('run'), 'run-sql', 'primary')}
          ${state.tenant ? StatusBadge(`tenant: ${state.tenant}`, 'ok') : StatusBadge(t('noTenant'), 'planned')}
        </div>
        <div class="notice warning" style="margin-top:10px">${esc(t('sqlSupportNotice'))}</div>
      </div>
      <aside class="panel">
        <h3>${esc(t('sqlExamples'))}</h3>
        <div class="actions-list">${examples.map((ex) => `<button type="button" data-action="use-sql" data-sql="${esc(ex)}" style="font-family:monospace;font-size:12px">${esc(ex)}</button>`).join('')}</div>
      </aside>
    </section>
    <section class="panel">
      ${Tabs([{ id:'table', label:t('tableResult') },{ id:'json', label:'JSON' }], state.sqlTab, 'sql-tab')}
      ${result}
    </section>`;
}

function CollectionsPage() {
  const rows = annotateRows(state.collections || [], 'collections');
  const hasTenant = !!state.tenant;
  return `${PageHeader(t('collections'), t('collectionHelp'), hasTenant ? ActionButton(t('createCollection'), 'create-collection', 'primary') : '')}
    <section class="grid cols-2">
      <div class="panel">
        ${!hasTenant
          ? EmptyState(t('selectTenantFirst'), t('noTenantDesc'), ActionButton(t('createTenant'), 'go-tenants', 'primary'))
          : DataTable(rows, [{key:'name',label:'name'},{key:'description',label:'description'},{key:'tenant_id',label:'tenant'}],
              t('noCollections'), t('noCollectionDesc'),
              ActionButton(t('createCollection'), 'focus-collection-form'))}
      </div>
      <div class="panel">
        <h3>${esc(t('createCollection'))}</h3>
        ${!hasTenant ? `<div class="notice warning">${esc(t('collectionSelectBeforeCreate'))}</div>` : ''}
        <div class="form-grid" style="margin-top:12px">
          ${FormField(t('collection'), 'collectionName', state.collection)}
          ${FormField(t('collectionDescription'), 'collectionDescription')}
        </div>
        <div class="page-actions" style="margin-top:12px">
          ${ActionButton(t('save'), 'create-collection', 'primary')}
        </div>
        <div class="notice" style="margin-top:14px">${esc(t('collectionCreateNotice'))}</div>
      </div>
    </section>`;
}

function TablesPage() {
  const hasTenant = !!state.tenant;
  const hasCollection = !!state.collection;
  const tables = state.lists.tables || [];
  const tableHtml = tables.length === 0
    ? EmptyState(
        t('noTables'),
        t('noTablesDesc'),
        ActionButton(t('storeFirstRecord'), 'go-ingest')
      )
    : `<div class="table-wrap"><table>
        <thead><tr><th>${esc(t('tableName'))}</th><th>${esc(t('tableRecords'))}</th><th></th></tr></thead>
        <tbody>${tables.map((row) =>
          `<tr>
            <td><strong>${esc(row.name)}</strong></td>
            <td>${esc(String(row.record_count))}</td>
            <td><button class="ghost" type="button" data-action="delete-table" data-name="${esc(row.name)}" style="color:var(--red)">${esc(t('deleteAll'))}</button></td>
          </tr>`
        ).join('')}</tbody>
      </table></div>`;
  return `${PageHeader(t('tables'), t('tablesLead'), ActionButton(t('refresh'), 'refresh'))}
    <section class="grid cols-2">
      <div class="panel">
        ${!hasTenant
          ? EmptyState(t('noTenantSelected'), t('noTenantDesc'), ActionButton(t('createTenant'), 'go-tenants', 'primary'))
          : !hasCollection
            ? EmptyState(t('noCollectionSelected'), t('noCollectionDesc'), ActionButton(t('collections'), 'go-collections', 'primary'))
            : tableHtml
        }
      </div>
      <div class="panel">
        <h3>${esc(t('aboutTables'))}</h3>
        <p style="color:var(--muted);font-size:13.5px;line-height:1.6">${esc(t('tablesAboutLead'))}</p>
        <div class="notice" style="margin-top:12px">${esc(t('tablesDeleteNotice'))}</div>
        <div class="notice warning" style="margin-top:10px">${esc(t('tablesQueryNotice'))}</div>
      </div>
    </section>`;
}

function ObjectListPage(kind) {
  const rows = annotateRows(state.lists[kind] || [], kind);
  const labels = { records: t('records'), memories: t('memories'), documents: t('documents'), files: t('files') };
  const label = labels[kind];
  const hasTenant = !!state.tenant;
  return `${PageHeader(label, tf('objectListLead', { type: label }), ActionButton(t('refresh'), 'refresh'))}
    <section class="panel">
      ${!hasTenant
        ? EmptyState(t('noTenantSelected'), t('noTenantDesc'), ActionButton(t('createTenant'), 'go-tenants', 'primary'))
        : DataTable(rows, logicalColumns(rows), tf('noItems', { type: label }), tf('objectListEmpty', { type: label }),
            ActionButton(tf('ingestItems', { type: label }), 'go-ingest'))}
    </section>`;
}

function DropZone() {
  const types = [
    ['.pdf', `${t('documentSingular')} (${t('fileCountChunks')})`],
    ['.txt', t('documentSingular')],
    ['.md',  t('documentSingular')],
    ['.csv', `${t('records')} (SQL)`],
    ['.json',`${t('documentSingular')} / ${t('records')}`],
  ];
  const badges = types.map(([ext, kind]) =>
    `<span class="file-type-badge" title="${esc(kind)}">${esc(ext)}</span>`
  ).join('');
  const progress = state.uploadProgress > 0
    ? `<div class="drop-zone-progress"><div class="drop-zone-progress-bar" style="width:${state.uploadProgress}%"></div></div>`
    : '';
  return `<div class="drop-zone" id="dropZone" data-action="drop-zone-click">
    <p class="drop-zone-hint">${esc(t('dropFiles'))}</p>
    <div class="drop-zone-types">${badges}</div>
    ${progress}
    <input type="file" id="fileInput" style="display:none" multiple accept=".pdf,.txt,.md,.csv,.json" />
  </div>`;
}

function FilesPage() {
  const rows = annotateRows(state.lists.files || [], 'files');
  const hasTenant = !!state.tenant;
  const result = state.uploadResult
    ? `<div class="upload-result">✓ ${esc(state.uploadResult.name)} → ${esc(state.uploadResult.kind)} (${state.uploadResult.count} ${esc(t(state.uploadResult.kind === 'records' ? 'fileCountRows' : 'fileCountChunks'))})</div>`
    : '';
  return `${PageHeader(t('files'), t('filesLead'), ActionButton(t('refresh'), 'refresh'))}
    <section class="grid cols-2">
      <div class="panel">
        <h3>${esc(t('upload'))}</h3>
        ${!hasTenant
          ? EmptyState(t('noTenantSelected'), t('noTenantDesc'), ActionButton(t('createTenant'), 'go-tenants', 'primary'))
          : `${FormField(t('collection'), 'fileCollection', state.collection)}
             ${DropZone()}
             ${result}`
        }
        <div class="notice" style="margin-top:12px">${esc(t('uploadMapping'))}</div>
      </div>
      <div class="panel">
        <h3>${esc(t('storedFiles'))}</h3>
        ${DataTable(
          rows,
          [{key:'name',label:'name'},{key:'media_type',label:'type'},{key:'size_bytes',label:'bytes'},{key:'collection',label:'collection'}],
          t('empty'),
          t('dropFiles'),
          ''
        )}
      </div>
    </section>`;
}

function IngestionRecallPage() {
  const hasTenant = !!state.tenant;
  if (!hasTenant) {
    return `${PageHeader(t('ingestionRecall'), t('ingestLead'))}
      ${EmptyState(t('noTenant'), t('noTenantDesc'), ActionButton(t('createTenant'), 'go-tenants', 'primary'))}`;
  }

  const typeLabels = {
    Memory: t('memorySingular'),
    Document: t('documentSingular'),
    Record: t('recordSingular'),
    File: t('fileSingular'),
  };
  const draft = state.ingestDraft;
  const collection = draft.collection || state.collection;
  const isRecord = state.ingestType === 'Record';
  const isFile = state.ingestType === 'File';
  const contentPlaceholder = isRecord
    ? '{"engine":"postgresql","status":"online"}'
    : t('contentPlaceholder');
  const storeAction = state.ingestType === 'Memory'
    ? ActionButton(t('storeMemory'), 'store-memory', 'primary')
    : state.ingestType === 'Document'
      ? ActionButton(t('storeDocument'), 'store-document', 'primary')
      : state.ingestType === 'Record'
        ? ActionButton(t('storeRecord'), 'store-record', 'primary')
        : ActionButton(t('openFileUploader'), 'go-files', 'primary');

  return `${PageHeader(t('ingestionRecall'), t('ingestLead'))}
    <section class="grid cols-2">
      <div class="panel">
        <h3>${esc(t('storeContext'))}</h3>
        <div class="radio-grid" style="margin-bottom:14px">
          ${['Memory','Document','Record','File'].map((kind) => `
            <label class="radio-card">
              <input type="radio" name="ingestType" value="${kind}" ${kind === state.ingestType ? 'checked' : ''} />
              <strong>${esc(typeLabels[kind])}</strong>
              <span>${esc(t('implemented'))}</span>
            </label>`).join('')}
        </div>
        <div class="form-grid">
          ${FormField(t('tenantId'), 'ingestTenant', state.tenant)}
          ${FormField(t('collection'), 'ingestCollection', collection, 'text', 'list="knownCollections" autocomplete="off"')}
          ${isRecord ? FormField(t('table'), 'ingestTable', draft.table || 'data') : ''}
          ${isFile ? '' : TextAreaField(isRecord ? t('jsonPayload') : t('content'), 'ingestContent', draft.content, `placeholder="${esc(contentPlaceholder)}"`)}
        </div>
        <div class="page-actions" style="margin-top:12px">
          ${storeAction}
        </div>
      </div>
      <div class="panel">
        <h3>${esc(t('recall'))} &amp; build_context</h3>
        <div class="form-grid" style="margin-bottom:12px">
          ${FormField(t('tenantId'), 'recallTenant', state.tenant)}
          ${FormField(t('collection'), 'recallCollection', state.collection, 'text', 'list="knownCollections" autocomplete="off"')}
          ${FormField(t('recallQuery'), 'recallInput', state.recallQuery, 'text', `placeholder="${esc(t('recallReady'))}"`)}
        </div>
        <div class="page-actions" style="margin-bottom:14px">
          ${ActionButton(t('recall'), 'run-recall', 'primary')}
          ${ActionButton(t('buildContext'), 'build-context')}
        </div>
        <div id="recallResult">${RecallResultView()}</div>
      </div>
    </section>
    ${CollectionDatalist('knownCollections')}`;
}

function GraphPage() {
  const rows = annotateRows(state.lists.graph || [], 'graph');
  return `${PageHeader(t('graph'), t('graphLead'), ActionButton(t('refresh'), 'refresh'))}
    <section class="grid cols-2">
      <div class="panel">
        ${DataTable(rows, [{key:'id',label:'id'},{key:'from_id',label:'from'},{key:'to_id',label:'to'},{key:'relation',label:'relation'}], t('noGraphEdges'), t('graphEmptyDesc'))}
      </div>
      <div class="panel">
        <h3>${esc(t('graphWorkbench'))}</h3>
        ${StatusBadge(t('planned'), 'planned')}
        <p style="color:var(--muted);font-size:13.5px;margin-top:10px">${esc(t('graphWorkbenchLead'))}</p>
      </div>
    </section>`;
}

function ApiReferencePage() {
  const groups = [
    [t('health'), [['GET','/health',true,'curl http://localhost:8080/health']]],
    [t('admin'), [['GET','/admin/bootstrap',true,'curl -H "x-admin-session: SESSION" http://localhost:8080/admin/bootstrap'],['POST','/admin/login',true,'curl -X POST -H "Content-Type: application/json" -d \'{"username":"admin","password":"pw"}\' http://localhost:8080/admin/login']]],
    [t('tenants'), [['GET','/admin/tenants',true,'curl -H "x-admin-session: SESSION" http://localhost:8080/admin/tenants'],['POST','/admin/tenants',true,'curl -X POST -H "x-admin-session: SESSION" -d \'{"id":"acme","name":"Acme"}\' http://localhost:8080/admin/tenants']]],
    [t('collections'), [['GET','/admin/collections',true,'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/collections?tenant_id=acme"'],['POST','/admin/tenants/:tid/collections',true,'curl -X POST -H "x-admin-session: SESSION" -d \'{"name":"support","description":"Support data"}\' http://localhost:8080/admin/tenants/acme/collections']]],
    [t('records'), [['GET','/admin/records',true,'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/records?tenant_id=acme"'],['POST','/admin/tenants/:tid/records',true,'curl -X POST -H "x-admin-session: SESSION" -d \'{"collection":"support","table":"systems","payload":{"engine":"pg"}}\' http://localhost:8080/admin/tenants/acme/records']]],
    [t('memories'), [['GET','/admin/memories',true,'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/memories?tenant_id=acme"'],['POST','/admin/tenants/:tid/memories',true,'curl -X POST -H "x-admin-session: SESSION" -d \'{"collection":"support","text":"fact","memory_type":"semantic"}\' http://localhost:8080/admin/tenants/acme/memories']]],
    [t('documents'), [['GET','/admin/documents',true,'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/documents?tenant_id=acme"'],['POST','/admin/tenants/:tid/documents',true,'curl -X POST -H "x-admin-session: SESSION" -d \'{"collection":"support","text":"document text here"}\' http://localhost:8080/admin/tenants/acme/documents']]],
    ['SQL', [['POST','/admin/sql',true,'curl -X POST -H "x-admin-session: SESSION" -d \'{"tenant_id":"acme","sql":"select * from systems limit 5"}\' http://localhost:8080/admin/sql']]],
    [t('recallAndContext'), [
      ['POST','/admin/tenants/:tid/recall',true,'curl -X POST -H "x-admin-session: SESSION" -d \'{"query":"postgresql","top_k":5,"min_score":0.2,"dedup_chunks":true,"mmr":true}\' http://localhost:8080/admin/tenants/acme/recall'],
      ['POST','/admin/tenants/:tid/context',true,'curl -X POST -H "x-admin-session: SESSION" -d \'{"query":"postgresql","max_tokens":1024}\' http://localhost:8080/admin/tenants/acme/context'],
    ]],
    [t('files'), [['GET','/admin/files',true,'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/files?tenant_id=acme"'],['POST','/admin/tenants/:tid/files',true,'multipart form-data: collection + file']]],
    [t('graph'), [['GET','/admin/graph-edges',true,'curl -H "x-admin-session: SESSION" "http://localhost:8080/admin/graph-edges?tenant_id=acme"'],['POST','/tenants/:tid/graph/traverse',true,'service API with X-Api-Key']]],
    [t('serviceKeys'), [['POST','/admin/api-key/rotate',true,'curl -X POST -H "x-admin-session: SESSION" -d \'{"length":32}\' http://localhost:8080/admin/api-key/rotate']]],
  ];
  const panels = groups.map(([group, endpoints]) =>
    `<div class="panel"><h3>${esc(group)}</h3>${endpoints.map(([method, path, implemented, curl]) =>
      CodeBlock(`${method} ${path} ${StatusBadge(t(implemented ? 'implemented' : 'planned'), implemented ? 'ok' : 'planned')}`, curl,
        `<button type="button" class="ghost" data-action="copy-code" data-code="${esc(curl)}">${esc(t('copy'))}</button>`)
    ).join('')}</div>`
  ).join('');
  return `${PageHeader(t('apiReference'), t('apiReferenceLead'))}<section class="grid cols-2">${panels}</section>`;
}

function DocumentationPage() {
  const lang = state.lang;
  const docId = state.docSection || 'overview';
  const doc = DOCS[docId];
  const content = doc
    ? (doc[lang] || doc.en)
    : { title: t('docsNotFound'), content: `<p>${esc(t('docsSectionNotFound'))}</p>` };

  const navItems = DOCS_NAV.map(([group, groupLabel, ids]) => {
    const items = ids.map((id) => {
      const label = (DOCS_LABELS[id] && DOCS_LABELS[id][lang]) || DOCS_LABELS[id]?.en || id;
      return `<button type="button" data-action="doc-section" data-section="${esc(id)}" aria-selected="${id === docId}">${esc(label)}</button>`;
    }).join('');
    return `<div class="docs-group-label">${esc(t(groupLabel))}</div>${items}`;
  }).join('');

  return `${PageHeader(t('documentation'), t('docsLead'))}
    <div class="docs-layout">
      <nav class="docs-nav">${navItems}</nav>
      <article class="docs-content">
        <h2>${esc(content.title)}</h2>
        ${content.content}
      </article>
    </div>`;
}

function TenantsPage() {
  const rows = annotateRows(state.bootstrap?.tenants || [], 'tenants');
  const hasTenants = rows.length > 0;
  return `${PageHeader(t('tenants'), t('tenantLead'), ActionButton(t('createTenant'), 'create-tenant', 'primary'))}
    <section class="grid cols-2">
      <div class="panel">
        ${!hasTenants
          ? EmptyState(t('noTenants'), t('noTenantDesc'), ActionButton(t('createTenant'), 'focus-tenant-form', 'primary'))
          : DataTable(rows, [{key:'id',label:'id'},{key:'name',label:'name'}],t('noTenants'))}
      </div>
      <div class="panel">
        <h3>${esc(t('createTenant'))}</h3>
        <p style="color:var(--muted);font-size:13px;margin:0 0 14px">${esc(t('tenantPurpose'))}</p>
        <div class="form-grid">
          ${FormField(t('tenantId'), 'tenantFormId', '', 'text', 'placeholder="e.g. acme, dev, prod"')}
          ${FormField(t('name'), 'tenantFormName', '', 'text', 'placeholder="e.g. Acme Corp, Development"')}
        </div>
        <div class="page-actions" style="margin-top:12px">
          ${ActionButton(t('save'), 'create-tenant', 'primary')}
        </div>
      </div>
    </section>`;
}

function IntegrationsPage() {
  const providers = annotateRows(state.providers || [], 'providers');
  const apiKey = state.bootstrap?.config?.api_key_set;
  const keyLen = state.bootstrap?.config?.api_key_length ?? '—';
  const apiInfo = state.apiInfo || {};
  const baseUrl = apiInfo.base_url || 'http://localhost:8080';
  const keyHint = apiInfo.api_key_hint || '****...';
  const traefikDomain = state.traefikDomain || '';
  const traefikPath  = state.traefikPath  || '/api';

  const traefikYaml = traefikDomain
    ? `# ${t('traefikComment')}
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.hippocore.rule=Host(\`${esc(traefikDomain)}\`) && PathPrefix(\`${esc(traefikPath)}\`)"
  - "traefik.http.routers.hippocore.entrypoints=websecure"
  - "traefik.http.routers.hippocore.tls.certresolver=letsencrypt"
  - "traefik.http.middlewares.hippocore-strip.stripprefix.prefixes=${esc(traefikPath)}"
  - "traefik.http.routers.hippocore.middlewares=hippocore-strip"
  - "traefik.http.services.hippocore.loadbalancer.server.port=8080"` : '';

  const composeSnippet = `services:
  hippocore:
    image: hippocore:latest
    environment:
      - HIPPOCORE_ADMIN_PASSWORD=\${HIPPOCORE_ADMIN_PASSWORD}
      - HIPPOCORE_API_KEY=\${HIPPOCORE_API_KEY}
      - HIPPOCORE_OLLAMA_URL=http://ollama:11434
      - HIPPOCORE_OLLAMA_MODEL=llama3.2
    volumes:
      - ./hippocore-data:/data
    ports:
      - "8080:8080"`;

  return `${PageHeader(t('integrations'), t('integrationsLead'), ActionButton(t('refresh'), 'refresh'))}
    <section class="grid cols-2">
      <div class="panel">
        <h3>${esc(t('llmBrain'))}</h3>
        <p style="color:var(--muted);font-size:13px;margin:0 0 12px">${esc(t('llmBrainLead'))}</p>
        ${DataTable(providers, [{key:'id',label:'id'},{key:'kind',label:'kind'},{key:'model',label:'model'},{key:'api_key_set',label:'key set'}], t('noProviders'), t('noProvidersDesc'))}
      </div>
      <div class="panel">
        <h3>${esc(t('addOrUpdateProvider'))}</h3>
        <div class="form-grid">
          ${FormField(t('providerId'), 'providerId', 'ollama')}
          ${FormField(t('providerKind'), 'providerKind', 'ollama')}
          ${FormField(t('providerBaseUrl'), 'providerUrl', 'http://localhost:11434')}
          ${FormField(t('providerModel'), 'providerModel', 'llama3.2')}
          ${FormField(t('providerApiKeyOptional'), 'providerKey', '', 'password')}
        </div>
        <div class="page-actions" style="margin-top:12px">
          ${ActionButton(t('save'), 'save-provider', 'primary')}
          ${ActionButton(t('validate'), 'validate-provider')}
          ${ActionButton(t('ping'), 'ping-provider')}
        </div>
        ${state.providerResult ? `<div class="notice ${state.providerResult.ok === true ? '' : state.providerResult.ok === false ? 'warning' : ''}" style="margin-top:12px">
          ${state.providerResult.ok === true ? '✓ ' : state.providerResult.ok === false ? '✗ ' : ''}
          ${esc(state.providerResult.message ?? JSON.stringify(state.providerResult))}
          ${state.providerResult.latency_ms !== undefined ? ` (${state.providerResult.latency_ms}ms)` : ''}
        </div>` : ''}
      </div>
    </section>
    <section class="grid cols-2">
      <div class="panel">
        <h3>${esc(t('apiExposure'))}</h3>
        <p style="color:var(--muted);font-size:13px;margin:0 0 12px">${esc(t('serviceApiKeyLead'))}</p>
        <div style="display:flex;gap:8px;align-items:center;margin-bottom:10px">
          ${StatusBadge(apiKey ? t('apiKeyConfigured') : t('apiKeyNotSet'), apiKey ? 'ok' : 'error')}
          <span style="color:var(--muted);font-size:12px">${esc(t('keyLength'))}: ${esc(String(keyLen))}</span>
        </div>
        <label class="field"><span>${esc(t('providerBaseUrl'))}</span><input value="${esc(baseUrl)}" readonly onclick="this.select()"></label>
        <label class="field" style="margin-top:8px"><span>API key hint</span><input value="${esc(keyHint)}" readonly onclick="this.select()"></label>
        <div class="notice" style="margin-top:10px;font-size:12px">
          <strong>${esc(t('apiCurlExample'))}:</strong><br>
          <code>curl -X POST ${esc(baseUrl)}/tenants/MY_TENANT/recall \\<br>&nbsp;&nbsp;-H "X-Api-Key: YOUR_KEY" \\<br>&nbsp;&nbsp;-H "Content-Type: application/json" \\<br>&nbsp;&nbsp;-d '{"query":"what is...","collection":"main"}'</code>
        </div>
        <div class="page-actions" style="margin-top:14px">
          ${ActionButton(t('rotateKey'), 'rotate-key', 'danger')}
        </div>
      </div>
      <div class="panel">
        <h3>${esc(t('traefikGenerator'))}</h3>
        <div class="form-grid">
          ${FormField(t('traefikDomain'), 'traefikDomain', traefikDomain)}
          ${FormField(t('traefikPath'), 'traefikPath', traefikPath || '/api')}
        </div>
        <div class="page-actions" style="margin-top:10px">
          ${ActionButton(t('generateLabels'), 'generate-traefik', 'primary')}
        </div>
        ${traefikYaml ? `<pre style="background:var(--glass-md);border-radius:8px;padding:12px;font-size:11.5px;overflow-x:auto;margin-top:12px;white-space:pre-wrap">${esc(traefikYaml)}</pre>
          <div class="page-actions" style="margin-top:6px">${ActionButton(t('copy'), 'copy-traefik')}</div>` : ''}
      </div>
    </section>
    <section class="panel">
      <h3>${esc(t('composeSnippet'))}</h3>
      <pre style="background:var(--glass-md);border-radius:8px;padding:14px;font-size:12px;overflow-x:auto;white-space:pre-wrap">${esc(composeSnippet)}</pre>
      <div class="page-actions" style="margin-top:8px">${ActionButton(t('copySnippet'), 'copy-compose')}</div>
    </section>`;
}

function PromptsPage() {
  const prompts = annotateRows(state.lists.prompts || [], 'prompts');
  const editing = state.editingPrompt;
  return `${PageHeader(t('prompts'), t('promptsLead'), ActionButton(t('refresh'), 'refresh'))}
    <section class="grid cols-2">
      <div class="panel">
        <h3>${esc(t('promptLibrary'))}</h3>
        ${DataTable(
          prompts,
          [{key:'name',label:'name'},{key:'description',label:'description'},{key:'tenant_id',label:'tenant'}],
          t('noPrompts'),
          t('noPromptsDesc')
        )}
      </div>
      <div class="panel">
        <h3>${esc(editing ? t('editPrompt') : t('newPrompt'))}</h3>
        <div class="form-grid">
          ${FormField(t('name'), 'promptName', editing?.name || '')}
          ${FormField(t('promptDescriptionOptional'), 'promptDesc', editing?.description || '')}
          ${FormField(t('promptGlobalTenant'), 'promptTenant', editing?.tenant_id || '')}
        </div>
        <label class="field" style="margin-top:8px">
          <span>${esc(t('promptContentHelp'))}</span>
          <textarea id="promptContent" rows="8" style="font-family:monospace;font-size:12.5px;resize:vertical">${esc(editing?.content || t('defaultPrompt'))}</textarea>
        </label>
        <div class="page-actions" style="margin-top:12px">
          ${ActionButton(t('save'), 'save-prompt', 'primary')}
          ${editing ? ActionButton(t('cancel'), 'cancel-edit-prompt') : ''}
          ${editing ? ActionButton(t('delete'), 'delete-prompt', 'danger') : ''}
        </div>
      </div>
    </section>
    <section class="panel">
      <h3>${esc(t('testChat'))}</h3>
      <div class="grid cols-2" style="gap:12px">
        <div>
          ${FormField(t('tenant'), 'chatTenant', state.tenant || '')}
          ${FormField(t('collectionOptional'), 'chatCollection', state.collection || '')}
          ${FormField(t('promptIdOptional'), 'chatPromptId', editing?.id || '')}
          <label class="field"><span>${esc(t('question'))}</span><textarea id="chatQuery" rows="3" placeholder="${esc(t('questionPlaceholder'))}"></textarea></label>
          <div class="page-actions" style="margin-top:8px">${ActionButton(t('send'), 'test-chat', 'primary')}</div>
        </div>
        <div>
          <h4 style="margin:0 0 8px;font-size:13px;color:var(--muted)">${esc(t('response'))}</h4>
          <div id="chatOutput" style="background:var(--glass-md);border-radius:8px;padding:12px;min-height:120px;font-size:13.5px;line-height:1.6;white-space:pre-wrap;word-break:break-word">${esc(state.chatResult || '')}</div>
        </div>
      </div>
    </section>`;
}

function ObservabilityPage() {
  const s = stats();
  const physical = [
    [t('healthEndpoint'),  t('implemented'), 'GET /health -> { status: "ok" }'],
    ['Stats',              t('implemented'), '/admin/bootstrap'],
    [t('walEntries'),      t('implemented'), String(s.wal_entries ?? '—')],
    [t('auditLog'),        t('implemented'), `${s.audit_records ?? 0} / ${s.audit_log_bytes ?? 0} B`],
    [t('indexedEntries'),  t('implemented'), String(s.indexed_entries ?? '—')],
    [t('diskBytes'),       t('implemented'), String(s.disk_bytes ?? '—')],
    [t('chunkCount'),      t('planned'),     String(s.chunks ?? 0)],
    [t('snapshotDetails'), t('planned'),     t('snapshotDetails')],
    [t('indexHealth'),     t('planned'),     t('indexHealth')],
    [t('logStream'),       t('planned'),     t('structuredLogStream')],
  ];
  return `${PageHeader(t('observability'), t('observabilityLead'), ActionButton(t('refresh'), 'refresh'))}
    <section class="grid cols-3">
      ${StatCard(t('health'), state.session ? 'OK' : '—', '/health')}
      ${StatCard(t('diskBytes'), s.disk_bytes ?? '—')}
      ${StatCard(t('indexedEntries'), s.indexed_entries ?? '—')}
    </section>
    <section class="panel">
      ${DataTable(
        physical.map((row, idx) => ({ id: idx, name: row[0], status: row[1], detail: row[2], __list: 'observability' })),
        [{key:'name',label:'metric'},{key:'status',label:'status'},{key:'detail',label:'detail'}],
        t('noObservabilityData')
      )}
    </section>`;
}

function SettingsPage() {
  return `${PageHeader(t('settings'), t('settingsLead'))}
    <section class="grid cols-2">
      <div class="panel">
        <h3>Interface</h3>
        <div class="form-grid">
          ${FormField(t('language'), 'settingsLanguage', state.lang, 'text', 'placeholder="en or pt"')}
        </div>
        <div class="notice" style="margin-top:12px">${esc(t('settingsLanguageLead'))}</div>
      </div>
      <div class="panel">
        <h3>${esc(t('runtimeConfiguration'))}</h3>
        ${StatusBadge(t('planned'), 'planned')}
        <p style="color:var(--muted);font-size:13.5px;margin-top:10px">${esc(t('runtimeConfigurationLead'))}</p>
      </div>
    </section>`;
}

function LoginPage() {
  return `<div class="login-screen"><form class="login-card" id="loginForm">
    <div class="mark" aria-hidden="true"></div>
    <h1>${esc(t('appTitle'))}</h1>
    <p>${esc(t('appSubtitle'))}</p>
    <p style="margin-top:4px">${esc(t('loginLead'))}</p>
    ${FormField(t('adminUser'), 'adminUser', state.username)}
    ${FormField(t('adminPassword'), 'adminPass', '', 'password')}
    <label class="field">
      <span>${esc(t('language'))}</span>
      <select id="loginLang">
        <option value="en" ${state.lang === 'en' ? 'selected' : ''}>EN — English</option>
        <option value="pt" ${state.lang === 'pt' ? 'selected' : ''}>PT — Português</option>
      </select>
    </label>
    <button class="primary" type="submit">${esc(t('login'))}</button>
  </form>${state.toast ? `<div class="toast">${esc(messageText(state.toast))}</div>` : ''}</div>`;
}

function renderPage() {
  if (!state.session) return LoginPage();
  if (state.page === 'dashboard')       return AppShell(DashboardPage());
  if (state.page === 'data-explorer')   return AppShell(DataExplorerPage());
  if (state.page === 'sql-editor')      return AppShell(SqlEditorPage());
  if (state.page === 'collections')     return AppShell(CollectionsPage());
  if (state.page === 'tables')          return AppShell(TablesPage());
  if (['records','memories','documents'].includes(state.page)) return AppShell(ObjectListPage(state.page));
  if (state.page === 'files') return AppShell(FilesPage());
  if (state.page === 'ingestion-recall') return AppShell(IngestionRecallPage());
  if (state.page === 'graph')           return AppShell(GraphPage());
  if (state.page === 'api-reference')   return AppShell(ApiReferencePage());
  if (state.page === 'documentation')   return AppShell(DocumentationPage());
  if (state.page === 'tenants')         return AppShell(TenantsPage());
  if (state.page === 'integrations')    return AppShell(IntegrationsPage());
  if (state.page === 'prompts')         return AppShell(PromptsPage());
  if (state.page === 'observability')   return AppShell(ObservabilityPage());
  if (state.page === 'settings')        return AppShell(SettingsPage());
  return AppShell(DashboardPage());
}

function render(preservedFields = []) {
  document.documentElement.lang = state.lang === 'pt' ? 'pt-BR' : 'en';
  root.innerHTML = renderPage();
  restoreFormState(preservedFields);
}

/* ─── Data loading ───────────────────────────────────────────────────────── */
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
  if (!state.session || !state.tenant) {
    state.collections = [];
    return;
  }
  const params = `?${new URLSearchParams({ tenant_id: state.tenant })}`;
  state.collections = await request(`/admin/collections${params}`);
  if (!state.collection && state.collections.length > 0) {
    state.collection = state.collections[0].name;
  }
}

async function loadProviders() {
  try {
    state.providers = await request('/admin/llm-providers');
  } catch (_) {
    state.providers = [];
  }
}

async function loadList(kind) {
  if (!state.tenant) {
    state.lists[kind] = [];
    return;
  }
  const params = new URLSearchParams({ tenant_id: state.tenant });
  if (state.collection && ['memories','documents','records','files'].includes(kind)) {
    params.set('collection', state.collection);
  }
  state.lists[kind] = await request(`/admin/${kind}?${params}`);
}

async function hydratePage() {
  await loadBootstrap();
  const listPages = ['records','memories','documents','files'];
  if (state.page === 'data-explorer') {
    await Promise.all(EXPLORER_KINDS.map(([kind]) => loadList(kind)));
  }
  if (listPages.includes(state.page)) await loadList(state.page);
  if (state.page === 'graph' && state.tenant) {
    state.lists.graph = await request(`/admin/graph-edges?${new URLSearchParams({ tenant_id: state.tenant })}`);
  }
  if (state.page === 'tables' && state.tenant && state.collection) {
    const params = new URLSearchParams({ collection: state.collection });
    state.lists.tables = await request(`/admin/tenants/${encodeURIComponent(state.tenant)}/tables?${params}`);
  }
  if (state.page === 'integrations') {
    state.apiInfo = await request('/admin/api-info').catch(() => null);
  }
  if (state.page === 'prompts') {
    state.lists.prompts = await request(`/admin/prompts${state.tenant ? '?tenant_id=' + encodeURIComponent(state.tenant) : ''}`).catch(() => []);
  }
  render();
}

/* ─── Actions ────────────────────────────────────────────────────────────── */
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
  state.lastOperation = message('signedIn');
  persist();
  await hydratePage();
}

async function createTenant() {
  const id = document.getElementById('tenantFormId')?.value.trim();
  const name = document.getElementById('tenantFormName')?.value.trim() || id;
  if (!id) throw new Error(t('tenantIdRequired'));
  await request('/admin/tenants', { method: 'POST', body: JSON.stringify({ id, name }) });
  state.tenant = id;
  state.lastOperation = message('tenantCreatedNext', { id });
  await hydratePage();
  showToast(message('tenantCreated', { id }));
}

async function createCollection() {
  const name = document.getElementById('collectionName')?.value.trim();
  const description = document.getElementById('collectionDescription')?.value.trim();
  if (!state.tenant) throw new Error(t('selectTenantFirst'));
  if (!name) throw new Error(t('collectionNameRequired'));
  await request(`/admin/tenants/${encodeURIComponent(state.tenant)}/collections`, {
    method: 'POST',
    body: JSON.stringify({ name, description }),
  });
  state.collection = name;
  state.lastOperation = message('collectionCreatedNext', { name });
  await hydratePage();
  showToast(message('collectionCreated', { name }));
}

async function runSql() {
  const sql = document.getElementById('sqlInput')?.value.trim();
  if (!state.tenant) throw new Error(t('selectTenantTopbar'));
  state.sqlError = '';
  try {
    state.sqlResult = await request('/admin/sql', {
      method: 'POST',
      body: JSON.stringify({ tenant_id: state.tenant, sql }),
    });
    completeOnboardingStep('sql');
    state.lastOperation = message('sqlReturned', { count: state.sqlResult.row_count });
  } catch (err) {
    state.sqlResult = null;
    state.sqlError = err.message || String(err);
    state.lastOperation = message('sqlErrorMessage', { message: state.sqlError });
  }
  render();
}

async function storeMemory() {
  const tenant = document.getElementById('ingestTenant')?.value.trim() || state.tenant;
  const collection = document.getElementById('ingestCollection')?.value.trim() || state.collection;
  const text = document.getElementById('ingestContent')?.value.trim();
  if (!tenant || !collection || !text) throw new Error(t('ingestRequired'));
  state.lists.ingestResult = await request(`/admin/tenants/${encodeURIComponent(tenant)}/memories`, {
    method: 'POST',
    body: JSON.stringify({ collection, text, memory_type: 'semantic' }),
  });
  state.ingestDraft = { collection, table: state.ingestDraft.table, content: '' };
  state.lastOperation = message('memoryStored');
  showToast(message('memoryStoredToast'));
  await hydratePage();
}

async function storeDocument() {
  const tenant = document.getElementById('ingestTenant')?.value.trim() || state.tenant;
  const collection = document.getElementById('ingestCollection')?.value.trim() || state.collection;
  const text = document.getElementById('ingestContent')?.value.trim();
  if (!tenant || !collection || !text) throw new Error(t('ingestRequired'));
  state.lists.ingestResult = await request(`/admin/tenants/${encodeURIComponent(tenant)}/documents`, {
    method: 'POST',
    body: JSON.stringify({ collection, text }),
  });
  state.ingestDraft = { collection, table: state.ingestDraft.table, content: '' };
  state.lastOperation = message('documentStored');
  showToast(message('documentStoredToast'));
  await hydratePage();
}

async function storeRecord() {
  const tenant = document.getElementById('ingestTenant')?.value.trim() || state.tenant;
  const collection = document.getElementById('ingestCollection')?.value.trim() || state.collection;
  const table = document.getElementById('ingestTable')?.value.trim() || 'data';
  const raw = document.getElementById('ingestContent')?.value.trim();
  if (!tenant || !collection || !raw) throw new Error(t('recordRequired'));
  let payload;
  try { payload = JSON.parse(raw); } catch { throw new Error(t('recordJsonInvalid')); }
  if (typeof payload !== 'object' || Array.isArray(payload) || payload === null) {
    throw new Error(t('recordObjectRequired'));
  }
  state.lists.ingestResult = await request(`/admin/tenants/${encodeURIComponent(tenant)}/records`, {
    method: 'POST',
    body: JSON.stringify({ collection, table, payload }),
  });
  state.ingestDraft = { collection, table, content: '' };
  state.lastOperation = message('recordStored');
  showToast(message('recordStored'));
  await hydratePage();
}

async function runRecall(kind) {
  const tenant = document.getElementById('recallTenant')?.value.trim() || state.tenant;
  const collection = document.getElementById('recallCollection')?.value.trim() || state.collection;
  const query = document.getElementById('recallInput')?.value.trim();
  if (!tenant || !query) throw new Error(t('recallRequired'));
  const path = kind === 'context' ? 'context' : 'recall';
  const body = kind === 'context'
    ? { query, collection: collection || null, max_tokens: 1024, include_related: true }
    : { query, collection: collection || null, top_k: 5 };
  state.lists.recallResult = await request(`/admin/tenants/${encodeURIComponent(tenant)}/${path}`, {
    method: 'POST',
    body: JSON.stringify(body),
  });
  completeOnboardingStep('recall');
  state.recallKind = kind;
  state.recallTab = 'results';
  state.recallQuery = query;
  state.lastOperation = message(kind === 'context' ? 'contextBuilt' : 'recallCompleted');
  render();
}

async function rotateKey() {
  const result = await request('/admin/api-key/rotate', {
    method: 'POST',
    body: JSON.stringify({ length: 32 }),
  });
  state.detail = { titleKey: 'newServiceKeyTitle', value: result };
  state.lastOperation = message('serviceKeyRotated');
  showToast(message('serviceKeyRotatedToast'));
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
  state.lastOperation = message('providerSaved');
  showToast(message('providerSavedToast'));
  await hydratePage();
}

async function validateProvider() {
  state.providerResult = await request('/admin/llm-providers/validate', {
    method: 'POST',
    body: JSON.stringify(providerBody()),
  });
  state.lastOperation = message('providerValidated');
  render();
}

async function pingProvider() {
  const id = document.getElementById('providerId')?.value.trim();
  if (!id) { showError(t('providerIdRequired')); return; }
  state.providerResult = await request('/admin/llm-providers/ping', {
    method: 'POST',
    body: JSON.stringify({ id }),
  });
  state.lastOperation = message('providerPinged', { id });
  render();
}

async function savePrompt() {
  const name    = document.getElementById('promptName')?.value.trim();
  const desc    = document.getElementById('promptDesc')?.value.trim();
  const tenant  = document.getElementById('promptTenant')?.value.trim() || null;
  const content = document.getElementById('promptContent')?.value.trim();
  if (!name || !content) { showError(t('promptNameContentRequired')); return; }
  const editing = state.editingPrompt;
  const body = { id: editing?.id || null, name, description: desc, content, tenant_id: tenant };
  const method = editing ? 'PUT' : 'POST';
  const url = editing ? `/admin/prompts/${encodeURIComponent(editing.id)}` : '/admin/prompts';
  await request(url, { method, body: JSON.stringify(body) });
  state.editingPrompt = null;
  state.lastOperation = message('promptSaved', { name });
  showToast(message('promptSaved', { name }));
  await hydratePage();
}

async function deletePrompt() {
  const editing = state.editingPrompt;
  if (!editing) return;
  if (!window.confirm(tf('deletePromptConfirm', { name: editing.name }))) return;
  await request(`/admin/prompts/${encodeURIComponent(editing.id)}`, { method: 'DELETE' });
  state.editingPrompt = null;
  state.lastOperation = message('promptDeleteDone', { name: editing.name });
  showToast(message('promptDeleteDone', { name: editing.name }));
  await hydratePage();
}

async function runChat() {
  const tid    = document.getElementById('chatTenant')?.value.trim() || state.tenant;
  const col    = document.getElementById('chatCollection')?.value.trim() || state.collection || undefined;
  const pid    = document.getElementById('chatPromptId')?.value.trim() || undefined;
  const query  = document.getElementById('chatQuery')?.value.trim();
  if (!tid)   { showError(t('tenantRequired')); return; }
  if (!query) { showError(t('questionRequired')); return; }
  const output = document.getElementById('chatOutput');
  if (output) output.textContent = '…';
  state.chatResult = '';
  const body = { query, collection: col, system_prompt_id: pid };
  try {
    const resp = await fetch(`/admin/tenants/${encodeURIComponent(tid)}/chat`, {
      method: 'POST',
      headers: { 'content-type': 'application/json', 'x-admin-session': state.session },
      body: JSON.stringify(body),
    });
    if (!resp.ok) { if (output) output.textContent = `Error: HTTP ${resp.status}`; return; }
    const reader = resp.body.getReader();
    const dec = new TextDecoder();
    let buf = '';
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buf += dec.decode(value, { stream: true });
      const lines = buf.split('\n');
      buf = lines.pop();
      for (const line of lines) {
        if (line.startsWith('data:')) {
          const token = line.slice(5);
          state.chatResult += token;
          if (output) output.textContent = state.chatResult;
        } else if (line.startsWith('event:done')) {
          // sources in next data line — ignore for display
        }
      }
    }
  } catch (e) {
    if (output) output.textContent = `Error: ${e.message}`;
  }
}

function openDetail(list, index) {
  const rows = list === 'sql'
    ? state.sqlResult?.rows || []
    : list === 'collections'  ? state.collections || []
    : list === 'tenants'      ? state.bootstrap?.tenants || []
    : list === 'graph'        ? state.lists.graph || []
    : list === 'providers'    ? state.providers || []
    : state.lists[list] || [];
  const value = rows[Number(index)];
  if (!value) return;
  if (list === 'prompts') { state.editingPrompt = value; render(); return; }
  state.detail = { title: value.id || value.name || list, value };
  render();
}

function copyCode(value) {
  navigator.clipboard?.writeText(value).then(() => showToast(message('copied')));
}

/* ─── Event handling ─────────────────────────────────────────────────────── */
async function handleAction(target) {
  const action = target.dataset.action;
  if (!action) return;

  // Navigation shortcuts from Dashboard / empty states
  if (action === 'go-tenants')     { route('tenants');          return; }
  if (action === 'go-collections') { route('collections');       return; }
  if (action === 'go-sql')         { route('sql-editor');        return; }
  if (action === 'go-explorer')    { route('data-explorer');     return; }
  if (action === 'go-ingest')      { route('ingestion-recall');  return; }
  if (action === 'go-keys')        { route('integrations');      return; }
  if (action === 'go-docs')        { route('documentation');     return; }
  if (action === 'go-tables')      { route('tables');            return; }
  if (action === 'go-files')       { route('files');             return; }
  if (action === 'go-observability') { route('observability');    return; }
  if (action === 'ingest-explorer-kind') {
    if (state.explorerKind === 'files') { route('files'); return; }
    state.ingestType = { records: 'Record', memories: 'Memory', documents: 'Document' }[state.explorerKind] || 'Memory';
    route('ingestion-recall');
    return;
  }
  if (action === 'delete-table') {
    const name = target.dataset.name;
    if (!name) return;
    if (!window.confirm(tf('deleteTableConfirm', { name }))) return;
    const params = new URLSearchParams({ collection: state.collection });
    await request(
      `/admin/tenants/${encodeURIComponent(state.tenant)}/tables/${encodeURIComponent(name)}?${params}`,
      { method: 'DELETE' }
    );
    state.lastOperation = message('tableDeleted', { name });
    showToast(message('tableDeleted', { name }));
    await hydratePage();
    return;
  }
  if (action === 'focus-tenant-form') {
    document.getElementById('tenantFormId')?.focus();
    return;
  }
  if (action === 'focus-collection-form') {
    document.getElementById('collectionName')?.focus();
    return;
  }

  if (action === 'refresh')      { await hydratePage();    return; }
  if (action === 'logout') {
    state.session = ''; state.bootstrap = null;
    persist(); render();
    return;
  }
  if (action === 'choose-tenant') {
    state.tenant = target.dataset.tenant;
    state.collection = '';
    state.explorerSearch = '';
    await hydratePage();
    return;
  }
  if (action === 'choose-collection') {
    state.collection = target.dataset.collection;
    persist();
    await hydratePage();
    return;
  }
  if (action === 'choose-explorer-kind') {
    state.explorerKind = target.dataset.kind;
    render();
    return;
  }
  if (action === 'tab')     { state.tab    = target.dataset.tab; render(); return; }
  if (action === 'sql-tab') { state.sqlTab = target.dataset.tab; render(); return; }
  if (action === 'recall-tab') { state.recallTab = target.dataset.tab; render(); return; }
  if (action === 'doc-section') { state.docSection = target.dataset.section; render(); return; }
  if (action === 'use-sql') {
    const el = document.getElementById('sqlInput');
    if (el) el.value = target.dataset.sql;
    return;
  }
  if (action === 'run-sql')           { await runSql();                return; }
  if (action === 'create-tenant')     { await createTenant();          return; }
  if (action === 'create-collection') { await createCollection();      return; }
  if (action === 'store-memory')      { await storeMemory();           return; }
  if (action === 'store-document')    { await storeDocument();         return; }
  if (action === 'store-record')      { await storeRecord();           return; }
  if (action === 'run-recall')        { await runRecall('recall');     return; }
  if (action === 'build-context')     { await runRecall('context');    return; }
  if (action === 'rotate-key')        { await rotateKey();             return; }
  if (action === 'save-provider')     { await saveProvider();          return; }
  if (action === 'validate-provider') { await validateProvider();      return; }
  if (action === 'ping-provider')     { await pingProvider();          return; }
  if (action === 'save-prompt')       { await savePrompt();            return; }
  if (action === 'delete-prompt')     { await deletePrompt();          return; }
  if (action === 'cancel-edit-prompt') { state.editingPrompt = null; render(); return; }
  if (action === 'go-prompts')        { route('prompts');              return; }
  if (action === 'test-chat')         { await runChat();               return; }
  if (action === 'generate-traefik') {
    state.traefikDomain = document.getElementById('traefikDomain')?.value.trim() || '';
    state.traefikPath   = document.getElementById('traefikPath')?.value.trim() || '/api';
    render();
    return;
  }
  if (action === 'copy-traefik') {
    const pre = document.querySelector('pre');
    if (pre) navigator.clipboard?.writeText(pre.textContent).catch(() => {});
    showToast(message('copiedToClipboard'));
    return;
  }
  if (action === 'copy-compose') {
    const pres = document.querySelectorAll('pre');
    const last = pres[pres.length - 1];
    if (last) navigator.clipboard?.writeText(last.textContent).catch(() => {});
    showToast(message('copiedToClipboard'));
    return;
  }
  if (action === 'detail')            { openDetail(target.dataset.list, target.dataset.index); return; }
  if (action === 'close-detail')      { state.detail = null; render(); return; }
  if (action === 'copy-code')         { copyCode(target.dataset.code); return; }
  if (action === 'drop-zone-click')   { document.getElementById('fileInput')?.click(); return; }
}

/* ─── File upload via XHR (progress events) ─────────────────────────────── */
async function uploadFile(file) {
  if (!state.tenant) throw new Error(t('selectTenantFirst'));
  const collection = document.getElementById('fileCollection')?.value.trim() || state.collection;
  if (!collection) throw new Error(t('uploadCollectionRequired'));
  state.uploadProgress = 1;
  state.uploadResult = null;
  render();
  return new Promise((resolve, reject) => {
    const fd = new FormData();
    fd.append('file', file, file.name);
    fd.append('collection', collection);
    const xhr = new XMLHttpRequest();
    xhr.open('POST', `/admin/tenants/${encodeURIComponent(state.tenant)}/files`);
    xhr.setRequestHeader('x-admin-session', state.session);
    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable) {
        state.uploadProgress = Math.round((e.loaded / e.total) * 100);
        render();
      }
    };
    xhr.onload = () => {
      state.uploadProgress = 0;
      if (xhr.status >= 200 && xhr.status < 300) {
        try { state.uploadResult = JSON.parse(xhr.responseText); } catch (_) { state.uploadResult = { name: file.name, kind: 'unknown', count: 0 }; }
        resolve(state.uploadResult);
      } else {
        let msg = xhr.responseText;
        try { msg = JSON.parse(msg).error || msg; } catch (_) { /* keep raw */ }
        reject(new Error(`${file.name}: ${msg}`));
      }
    };
    xhr.onerror = () => {
      state.uploadProgress = 0;
      reject(new Error(t('uploadNetworkError')));
    };
    xhr.send(fd);
  });
}

function handleDroppedFiles(files) {
  Array.from(files).forEach((file) => {
    uploadFile(file)
      .then(() => {
        showToast(message('uploadComplete', { name: file.name }));
        state.lastOperation = message('uploadLastOperation', { name: file.name, kind: state.uploadResult?.kind });
        return hydratePage();
      })
      .catch(showError);
  });
}

root.addEventListener('click', (event) => {
  const routeTarget = event.target.closest('[data-route]');
  if (routeTarget) { route(routeTarget.dataset.route); return; }
  const actionTarget = event.target.closest('[data-action]');
  if (actionTarget) { handleAction(actionTarget).catch(showError); }
});

root.addEventListener('submit', (event) => {
  if (event.target.id === 'loginForm') { login(event).catch(showError); }
});

root.addEventListener('change', (event) => {
  if (event.target.id === 'globalTenant') {
    state.tenant = event.target.value;
    state.collection = '';
    state.explorerSearch = '';
    persist();
    hydratePage().catch(showError);
  }
  if (event.target.id === 'languageSelect' || event.target.id === 'loginLang') {
    const fields = captureFormState();
    state.lang = event.target.value;
    persist();
    render(fields);
  }
  if (event.target.id === 'fileInput' && event.target.files.length > 0) {
    handleDroppedFiles(event.target.files);
    event.target.value = '';
  }
  if (event.target.name === 'ingestType') {
    state.ingestDraft = {
      collection: document.getElementById('ingestCollection')?.value.trim() || state.collection,
      table: document.getElementById('ingestTable')?.value.trim() || state.ingestDraft.table || 'data',
      content: document.getElementById('ingestContent')?.value || '',
    };
    state.ingestType = event.target.value;
    render();
  }
});

root.addEventListener('input', (event) => {
  if (event.target.id !== 'explorerCollectionSearch') return;
  const query = event.target.value.trim().toLowerCase();
  state.explorerSearch = event.target.value;
  const nodes = [...root.querySelectorAll('[data-collection-node]')];
  let visible = 0;
  nodes.forEach((node) => {
    const matches = node.dataset.collectionNode.includes(query);
    node.hidden = !matches;
    if (matches) visible += 1;
  });
  const empty = root.querySelector('[data-search-empty]');
  if (empty) empty.hidden = visible > 0;
});

root.addEventListener('dragover', (event) => {
  if (event.target.closest('#dropZone')) {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'copy';
    event.target.closest('#dropZone').classList.add('drag-over');
  }
});

root.addEventListener('dragleave', (event) => {
  const zone = event.target.closest('#dropZone');
  if (zone && !zone.contains(event.relatedTarget)) {
    zone.classList.remove('drag-over');
  }
});

root.addEventListener('drop', (event) => {
  const zone = event.target.closest('#dropZone');
  if (zone) {
    event.preventDefault();
    zone.classList.remove('drag-over');
    if (event.dataTransfer.files.length > 0) {
      handleDroppedFiles(event.dataTransfer.files);
    }
  }
});

/* ─── Boot ───────────────────────────────────────────────────────────────── */
render();
if (state.session) {
  hydratePage().catch((err) => {
    state.session = '';
    showError(err);
    render();
  });
}

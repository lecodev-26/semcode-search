/* ============================================================
   SEMCODE SEARCH - main.js
   Frontend logic for Tauri + vanilla JavaScript
   ============================================================ */

(() => {
  "use strict";

  /* ============================================================
     GLOBAL STATE
     ============================================================ */

  const state = {
    lang: localStorage.getItem("semcode-lang") || "es",
    theme: localStorage.getItem("semcode-theme") || "dark",
    mode: "semantic",
    path: "",
    results: [],
    selectedIndex: -1,
    selectedResult: null,
    filters: {},
    config: {},
    aliases: [],
    projects: [],
    history: [],
    lastSearch: null,
    busy: false
  };

  /* ============================================================
     DOM HELPERS
     ============================================================ */

  const $ = (selector, root = document) => root.querySelector(selector);
  const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];

  /* ============================================================
     TAURI INVOKE
     ============================================================ */

  async function invoke(command, args = {}) {
    if (!window.__TAURI__?.core?.invoke) {
      throw new Error(`Tauri no está disponible para "${command}".`);
    }
    return window.__TAURI__.core.invoke(command, args);
  }

  /* ============================================================
     TRANSLATIONS
     ============================================================ */

  const TRANSLATIONS = {
    es: {
      search: "Buscar",
      index: "Indexar",
      stats: "Stats",
      projects: "Proyectos",
      history: "Historial",
      aliases: "Aliases",
      settings: "Configuración",
      settingsSubtitle: "Ajusta el comportamiento de Semcode Search.",
      heroTitle: "Encuentra código por significado.",
      heroSubtitle: "Describe lo que buscas en lenguaje natural y Semcode encontrará los fragmentos más relevantes.",
      searchPlaceholder: "¿Qué quieres encontrar en tu código?",
      filters: "Filtros",
      path: "Ruta",
      all: "Todo",
      results: "Resultados",
      readyToSearch: "Listo para buscar",
      lastSearch: "Última búsqueda",
      welcomeTitle: "Busca algo en tu codebase",
      welcomeSubtitle: "Usa lenguaje natural para encontrar el código que necesitas.",
      selectResult: "Selecciona un resultado",
      selectToSeeCode: "Selecciona un resultado para ver el código.",
      copy: "Copiar",
      editor: "Editor",
      explorer: "Explorador",
      indexed: "Indexado",
      files: "archivos",
      noProject: "Sin proyecto",
      shortcuts: "Atajos",
      advancedFilters: "Filtros avanzados",
      extensions: "Extensiones",
      exclude: "Excluir",
      minScore: "Min. relevancia",
      caseSensitive: "Case sensitive",
      reset: "Restablecer",
      applyFilters: "Aplicar filtros",
      statistics: "Estadísticas",
      scSearch: "Buscar",
      scRunSearch: "Ejecutar búsqueda",
      scClose: "Cerrar modal / preview",
      scNavigate: "Navegar resultados",
      scOpen: "Abrir resultado",
      loading: "Cargando...",
      noProjects: "Sin proyectos recientes",
      noHistory: "Sin historial",
      noAliases: "Sin aliases",
      noResults: "No encontramos resultados",
      noResultsHint: "Prueba una descripción más amplia o cambia los filtros.",
      searching: "Buscando...",
      error: "Error",
      codeRelevant: "Código relevante",
      semanticMatch: "Coincidencia semántica",
      clickToOpen: "Click para abrir preview",
      resultsCount: "resultados",
      noProjectSelected: "Selecciona un proyecto primero",
      noProjectSelectedHint: "Pulsa el botón de abajo para elegir la carpeta del proyecto.",
      selectProject: "Seleccionar proyecto"
    },
    en: {
      search: "Search",
      index: "Index",
      stats: "Stats",
      projects: "Projects",
      history: "History",
      aliases: "Aliases",
      settings: "Settings",
      settingsSubtitle: "Adjust how Semcode Search behaves.",
      heroTitle: "Find code by meaning.",
      heroSubtitle: "Describe what you're looking for in natural language and Semcode will find the most relevant fragments.",
      searchPlaceholder: "What do you want to find in your code?",
      filters: "Filters",
      path: "Path",
      all: "All",
      results: "Results",
      readyToSearch: "Ready to search",
      lastSearch: "Last search",
      welcomeTitle: "Search your codebase",
      welcomeSubtitle: "Use natural language to find the code you need.",
      selectResult: "Select a result",
      selectToSeeCode: "Select a result to see the code.",
      copy: "Copy",
      editor: "Editor",
      explorer: "Explorer",
      indexed: "Indexed",
      files: "files",
      noProject: "No project",
      shortcuts: "Shortcuts",
      advancedFilters: "Advanced filters",
      extensions: "Extensions",
      exclude: "Exclude",
      minScore: "Min. score",
      caseSensitive: "Case sensitive",
      reset: "Reset",
      applyFilters: "Apply filters",
      statistics: "Statistics",
      scSearch: "Search",
      scRunSearch: "Run search",
      scClose: "Close modal / preview",
      scNavigate: "Navigate results",
      scOpen: "Open result",
      loading: "Loading...",
      noProjects: "No recent projects",
      noHistory: "No history",
      noAliases: "No aliases",
      noResults: "No results found",
      noResultsHint: "Try a broader description or change the filters.",
      searching: "Searching...",
      error: "Error",
      codeRelevant: "Relevant code",
      semanticMatch: "Semantic match",
      clickToOpen: "Click to open preview",
      resultsCount: "results",
      noProjectSelected: "Select a project first",
      noProjectSelectedHint: "Click the button below to choose the project folder.",
      selectProject: "Select project"
    }
  };

  function t(key) {
    return TRANSLATIONS[state.lang]?.[key] || key;
  }

  function applyTranslations() {
    $$("[data-i18n]").forEach(el => {
      const key = el.dataset.i18n;
      if (TRANSLATIONS[state.lang]?.[key]) {
        el.textContent = t(key);
      }
    });

    $$("[data-i18n-placeholder]").forEach(el => {
      const key = el.dataset.i18nPlaceholder;
      if (TRANSLATIONS[state.lang]?.[key]) {
        el.placeholder = t(key);
      }
    });

    const langToggle = $("#languageToggle");
    if (langToggle) {
      langToggle.textContent = state.lang.toUpperCase();
    }
  }

  async function setLanguage(lang) {
    state.lang = lang;
    localStorage.setItem("semcode-lang", lang);

    try {
      await invoke("set_language", { lang });
    } catch (_) {}

    applyTranslations();
  }

  /* ============================================================
     TOAST
     ============================================================ */

  function toast(message, type = "info") {
    const container = $("#toastContainer");
    if (!container) return;

    const el = document.createElement("div");
    el.className = `toast ${type}`;
    el.textContent = message;
    container.appendChild(el);

    setTimeout(() => el.remove(), 3200);
  }

  /* ============================================================
     BUSY
     ============================================================ */

  function setBusy(busy) {
    state.busy = busy;
    const searchBtn = $("#searchBtn");
    if (!searchBtn) return;

    searchBtn.disabled = busy;
    searchBtn.style.opacity = busy ? ".65" : "1";

    const label = searchBtn.querySelector("span");
    if (label) {
      label.textContent = busy ? t("searching") : t("search");
    }
  }

  /* ============================================================
     THEME
     ============================================================ */

  function applyTheme() {
    document.documentElement.classList.toggle("light", state.theme === "light");
    const toggle = $("#themeToggle");
    if (toggle) {
      toggle.textContent = state.theme === "light" ? "\u2600" : "\u25D0";
    }
  }

  /* ============================================================
     RENDER "NO PROJECT" WELCOME
     ============================================================ */

  function renderNoProjectWelcome() {
    const list = $("#resultsList");
    if (!list) return;

    list.innerHTML = `
      <div class="welcome-card">
        <div class="welcome-icon">&#128193;</div>
        <h3>${escapeHtml(t("noProjectSelected"))}</h3>
        <p>${escapeHtml(t("noProjectSelectedHint"))}</p>
        <button id="selectProjectFromWelcome" class="primary-btn" style="margin-top:18px;">${escapeHtml(t("selectProject"))}</button>
      </div>
    `;

    const btn = $("#selectProjectFromWelcome");
    if (btn) {
      btn.addEventListener("click", chooseProject);
    }
  }

  /* ============================================================
     BOOTSTRAP
     ============================================================ */

  async function loadBootstrap() {
    await Promise.allSettled([
      loadProjects(),
      loadHistory(),
      loadAliases(),
      loadConfig(),
      loadSystemInfo(),
      loadVersion(),
      checkAI()
    ]);
    renderSettings();

    // Si no hay proyecto seleccionado, mostrar welcome de "sin proyecto"
    if (!state.path) {
      renderNoProjectWelcome();
      const meta = $("#resultsMeta");
      if (meta) meta.textContent = t("noProjectSelected");
    }
  }

  /* ============================================================
     PROJECTS
     ============================================================ */

  async function loadProjects() {
    try {
      const data = await invoke("list_recent_projects");
      state.projects = Array.isArray(data) ? data : (data?.projects || []);
      renderProjects();

      // Si no hay proyecto seleccionado pero hay recientes, NO auto-seleccionar.
      // Queremos que el usuario elija explícitamente.
    } catch (_) {
      const el = $("#projectsList");
      if (el) el.innerHTML = `<div class="empty-side">${t("noProjects")}</div>`;
    }
  }

  function normalizeProject(item) {
    if (typeof item === "string") {
      return { path: item, name: basename(item) };
    }
    return {
      path: item?.path || item?.project_path || "",
      name: item?.name || basename(item?.path || item?.project_path || "")
    };
  }

  function renderProjects() {
    const list = $("#projectsList");
    if (!list) return;

    list.innerHTML = "";

    if (!state.projects.length) {
      list.innerHTML = `<div class="empty-side">${t("noProjects")}</div>`;
      return;
    }

    state.projects.slice(0, 8).forEach(raw => {
      const item = normalizeProject(raw);
      const el = document.createElement("div");
      el.className = "side-item";
      el.title = item.path;
      el.innerHTML = `
        <span class="side-icon">&#9670;</span>
        <span class="side-label">${escapeHtml(item.name || item.path)}</span>
        <button class="remove" title="Eliminar">&#215;</button>
      `;

      el.addEventListener("click", async ev => {
        if (ev.target.classList.contains("remove")) {
          try {
            await invoke("remove_recent_project", { path: item.path });
            await loadProjects();
          } catch (e) {
            toast(errorText(e), "error");
          }
          return;
        }
        setPath(item.path);
      });

      list.appendChild(el);
    });
  }

  /* ============================================================
     HISTORY
     ============================================================ */

  async function loadHistory() {
    try {
      const data = await invoke("get_history", { limit: 8 });
      state.history = Array.isArray(data) ? data : (data?.history || []);
      renderHistory();
    } catch (_) {
      const el = $("#historyList");
      if (el) el.innerHTML = `<div class="empty-side">${t("noHistory")}</div>`;
    }
  }

  function normalizeHistory(item) {
    if (typeof item === "string") return { query: item };
    return {
      query: item?.query || item?.search_query || "",
      path: item?.path || "",
      mode: item?.mode || "semantic"
    };
  }

  function renderHistory() {
    const list = $("#historyList");
    if (!list) return;

    list.innerHTML = "";

    if (!state.history.length) {
      list.innerHTML = `<div class="empty-side">${t("noHistory")}</div>`;
      return;
    }

    state.history.slice(0, 8).forEach(raw => {
      const item = normalizeHistory(raw);
      const el = document.createElement("div");
      el.className = "side-item";
      el.title = item.query;
      el.innerHTML = `
        <span class="side-icon">&#8599;</span>
        <span class="side-label">${escapeHtml(item.query)}</span>
      `;

      el.addEventListener("click", () => {
        $("#searchInput").value = item.query;
        if (item.path) setPath(item.path);
        runSearch();
      });

      list.appendChild(el);
    });
  }

  /* ============================================================
     ALIASES
     ============================================================ */

  async function loadAliases() {
    try {
      const data = await invoke("list_aliases");
      state.aliases = Array.isArray(data) ? data : (data?.aliases || []);
      renderAliases();
    } catch (_) {
      const el = $("#aliasesList");
      if (el) el.innerHTML = `<div class="empty-side">${t("noAliases")}</div>`;
    }
  }

  function normalizeAlias(item) {
    if (typeof item === "string") return { name: item };
    return {
      name: item?.name || "",
      query: item?.query || "",
      params: item?.params || []
    };
  }

  function renderAliases() {
    const list = $("#aliasesList");
    if (!list) return;

    list.innerHTML = "";

    if (!state.aliases.length) {
      list.innerHTML = `<div class="empty-side">${t("noAliases")}</div>`;
      return;
    }

    state.aliases.slice(0, 8).forEach(raw => {
      const item = normalizeAlias(raw);
      const el = document.createElement("div");
      el.className = "side-item";
      el.title = item.query || item.name;
      el.innerHTML = `
        <span class="side-icon">&#9671;</span>
        <span class="side-label">${escapeHtml(item.name)}</span>
        <button class="remove" title="Eliminar">&#215;</button>
      `;

      el.addEventListener("click", async ev => {
        if (ev.target.classList.contains("remove")) {
          try {
            await invoke("delete_alias", { name: item.name });
            await loadAliases();
          } catch (e) {
            toast(errorText(e), "error");
          }
          return;
        }

        try {
          const result = await invoke("run_alias", { name: item.name });
          if (result?.results) {
            renderResults(result.results, result);
          } else if (result) {
            renderResults(result, {});
          }
        } catch (e) {
          toast(errorText(e), "error");
        }
      });

      list.appendChild(el);
    });
  }

  /* ============================================================
     CONFIG
     ============================================================ */

  async function loadConfig() {
    try {
      state.config = (await invoke("get_config")) || {};
    } catch (_) {
      state.config = {};
    }
  }

  /* ============================================================
     VERSION
     ============================================================ */

  async function loadVersion() {
    try {
      const version = await invoke("get_version");
      const value = typeof version === "string" ? version : (version?.version || "—");
      const label = $("#versionLabel");
      if (label) label.textContent = `Semcode v${value}`;
    } catch (_) {}
  }

  /* ============================================================
     SYSTEM INFO
     ============================================================ */

  async function loadSystemInfo() {
    try {
      const info = await invoke("get_system_info");
      const text = typeof info === "string"
        ? info
        : `${info?.os || "System"} ${info?.arch || ""}`.trim();
      const label = $("#systemLabel");
      if (label) label.textContent = text || "—";
    } catch (_) {}
  }

  /* ============================================================
     AI STATUS
     ============================================================ */

  async function checkAI() {
    const chip = $("#aiStatus");
    if (!chip) return;

    try {
      const status = await invoke("get_ai_status");
      const ready = status === true || status?.available === true || status?.enabled === true;

      chip.innerHTML = `
        <span class="status-dot ${ready ? "" : "off"}"></span>
        <span>${ready ? "AI Ready" : "AI Offline"}</span>
      `;
    } catch (_) {
      chip.innerHTML = `<span class="status-dot off"></span><span>AI Offline</span>`;
    }
  }

  /* ============================================================
     PATH
     ============================================================ */

  function setPath(path) {
    state.path = path || "";
    const pathLabel = $("#pathLabel");
    if (pathLabel) {
      pathLabel.textContent = path ? basename(path) : t("all");
    }
    const currentProject = $("#currentProject");
    if (currentProject) {
      currentProject.textContent = path ? basename(path) : t("noProject");
    }

    // Si hay proyecto, limpiar el welcome de "sin proyecto"
    if (path && state.results.length === 0) {
      const list = $("#resultsList");
      if (list) {
        list.innerHTML = `
          <div class="welcome-card">
            <div class="welcome-icon">&#8981;</div>
            <h3>${escapeHtml(t("welcomeTitle"))}</h3>
            <p>${escapeHtml(t("welcomeSubtitle"))}</p>
          </div>
        `;
      }
      const meta = $("#resultsMeta");
      if (meta) meta.textContent = t("readyToSearch");
    }
  }

  /* ============================================================
     CHOOSE PROJECT
     ============================================================ */

  async function chooseProject() {
    try {
      if (!window.__TAURI__?.dialog?.open) {
        throw new Error("El diálogo de Tauri no está disponible.");
      }

      const selected = await window.__TAURI__.dialog.open({
        directory: true,
        multiple: false,
        title: "Selecciona el proyecto"
      });

      if (!selected) return;

      const path = Array.isArray(selected) ? selected[0] : selected;
      setPath(path);

      try {
        await invoke("add_recent_project", { path });
      } catch (_) {}

      await loadProjects();
      await indexProject(path);
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  /* ============================================================
     INDEX
     ============================================================ */

  async function indexProject(path = state.path) {
    if (!path) {
      await chooseProject();
      return;
    }

    setBusy(true);

    const indexStatus = $("#indexStatus");
    if (indexStatus) {
      indexStatus.innerHTML = `<span class="status-dot"></span> ${t("searching")}`;
    }

    try {
      const command = state.mode === "ai" ? "index_with_ai" : "index_project";
      const result = await invoke(command, { path });

      if (indexStatus) {
        indexStatus.innerHTML = `<span class="status-dot"></span> ${t("indexed")}`;
      }

      const count = result?.total_files ?? result?.files_indexed ?? result?.file_count;
      if (count != null) {
        const fileCount = $("#fileCount");
        if (fileCount) fileCount.textContent = `${formatNumber(count)} ${t("files")}`;
      }

      toast(`${t("indexed")}: ${count ?? "?"} ${t("files")}`, "success");
      await loadProjects();
    } catch (e) {
      if (indexStatus) {
        indexStatus.innerHTML = `<span class="status-dot off"></span> ${t("error")}`;
      }
      toast(`${t("error")}: ${errorText(e)}`, "error");
    } finally {
      setBusy(false);
    }
  }

  /* ============================================================
     SEARCH
     ============================================================ */

  async function runSearch() {
    const input = $("#searchInput");
    if (!input) return;

    const query = input.value.trim();
    if (!query || state.busy) return;

    // 🔒 Comprobar que hay proyecto seleccionado
    if (!state.path) {
      toast(t("noProjectSelected"), "error");
      renderNoProjectWelcome();
      return;
    }

    setBusy(true);

    const resultsMeta = $("#resultsMeta");
    if (resultsMeta) resultsMeta.textContent = t("searching");

    const limit = Number($("#limitSelect")?.value || 20);

    try {
      let result;

      if (state.mode === "ai") {
        result = await invoke("search_with_ai", { query, path: state.path, limit });
      } else if (Object.keys(state.filters).length) {
        result = await invoke("search_advanced", {
          query,
          path: state.path,
          filters: state.filters,
          limit
        });
      } else {
        result = await invoke("search", { query, path: state.path, limit });
      }

      const results = Array.isArray(result) ? result : (result?.results || result?.items || []);

      state.lastSearch = { query, path: state.path, mode: state.mode, result };
      renderResults(results, result);
      await loadHistory();
    } catch (e) {
      if (resultsMeta) resultsMeta.textContent = t("error");
      toast(`${t("error")}: ${errorText(e)}`, "error");
    } finally {
      setBusy(false);
    }
  }

  /* ============================================================
     NORMALIZE RESULT
     ============================================================ */

  function normalizeResult(item) {
    if (typeof item === "string") {
      return { path: item, snippet: "", score: null };
    }
    return {
      path: item?.path || item?.file || item?.file_path || "",
      snippet: item?.snippet || item?.preview || item?.content || item?.text || "",
      score: item?.score ?? item?.similarity ?? item?.relevance ?? null,
      line: item?.line ?? item?.line_number ?? null,
      symbol: item?.symbol || item?.function || item?.name || "",
      language: item?.language || languageFromPath(item?.path || item?.file || "")
    };
  }

  /* ============================================================
     RENDER RESULTS
     ============================================================ */

  function renderResults(results, meta = {}) {
    state.results = Array.isArray(results) ? results : [];
    state.selectedIndex = -1;

    const list = $("#resultsList");
    if (!list) return;

    list.innerHTML = "";

    if (!state.results.length) {
      list.innerHTML = `
        <div class="welcome-card">
          <div class="welcome-icon">&#8981;</div>
          <h3>${escapeHtml(t("noResults"))}</h3>
          <p>${escapeHtml(t("noResultsHint"))}</p>
        </div>
      `;
      const metaEl = $("#resultsMeta");
      if (metaEl) metaEl.textContent = `0 ${t("resultsCount")}`;
      return;
    }

    const elapsed = meta?.time_ms ?? meta?.elapsed_ms;
    const metaEl = $("#resultsMeta");
    if (metaEl) {
      metaEl.textContent = `${state.results.length} ${t("resultsCount")}${elapsed != null ? ` · ${elapsed} ms` : ""}`;
    }

    state.results.forEach((raw, index) => {
      const result = normalizeResult(raw);
      const card = document.createElement("article");
      card.className = "result-card";
      card.tabIndex = 0;

      const score = result.score == null
        ? "—"
        : `${Math.round(Number(result.score) * 100)}%`;

      const symbol = result.symbol || t("codeRelevant");
      const snippet = String(result.snippet || "").slice(0, 800);

      card.innerHTML = `
        <div class="result-top">
          <span class="score">${escapeHtml(score)}</span>
          <span class="result-path">${escapeHtml(result.path || "Resultado")}</span>
          <span class="result-lang">${escapeHtml((result.language || "CODE").toUpperCase())}</span>
        </div>
        <div class="result-symbol">${escapeHtml(symbol)}${result.line != null ? ` · ${escapeHtml(String(result.line))}` : ""}</div>
        <div class="result-snippet">${escapeHtml(snippet || "Sin preview disponible.")}</div>
        <div class="result-footer">
          <span>${t("semanticMatch")}</span>
          <span>${t("clickToOpen")}</span>
        </div>
      `;

      card.addEventListener("click", () => selectResult(index));
      card.addEventListener("keydown", ev => { if (ev.key === "Enter") selectResult(index); });

      list.appendChild(card);
    });
  }

  /* ============================================================
     SELECT RESULT
     ============================================================ */

  async function selectResult(index) {
    if (index < 0 || index >= state.results.length) return;

    state.selectedIndex = index;

    $$(".result-card").forEach((el, i) => {
      el.classList.toggle("selected", i === index);
    });

    const result = normalizeResult(state.results[index]);
    state.selectedResult = result;

    const previewPanel = $("#previewPanel");
    if (previewPanel) previewPanel.classList.add("open");

    const previewName = $("#previewName");
    if (previewName) previewName.textContent = basename(result.path) || "Preview";

    const previewPath = $("#previewPath");
    if (previewPath) previewPath.textContent = result.path || "";

    const codePreview = $("#codePreview");
    if (codePreview) codePreview.innerHTML = `<div class="code-empty">${t("loading")}</div>`;

    try {
      const data = await invoke("read_file", { path: result.path });
      const content = typeof data === "string" ? data : (data?.content || data?.text || "");
      renderCode(content, result.language);
    } catch (e) {
      renderCode(result.snippet || `${t("error")}: ${errorText(e)}`, result.language);
    }
  }

  /* ============================================================
     CODE VIEW
     ============================================================ */

  function renderCode(content, language = "rust") {
    const codePreview = $("#codePreview");
    if (!codePreview) return;

    const lines = String(content).replace(/\r\n/g, "\n").split("\n");

    codePreview.innerHTML = lines.map((line, index) => `
      <div class="code-line">
        <span class="line-no">${index + 1}</span>
        <code class="line-code">${highlight(line, language)}</code>
      </div>
    `).join("");
  }

  /* ============================================================
     SIMPLE SYNTAX HIGHLIGHT
     ============================================================ */

  function highlight(line, language) {
    let s = escapeHtml(line);
    const lang = String(language || "").toLowerCase();

    if (lang === "rust" || lang === "rs") {
      s = s.replace(/(\/\/.*)$/g, '<span class="token-comment">$1</span>');
      s = s.replace(/(&quot;.*?&quot;|&#39;.*?&#39;)/g, '<span class="token-string">$1</span>');
      s = s.replace(/\b(\d+(?:\.\d+)?)\b/g, '<span class="token-number">$1</span>');
      s = s.replace(/\b(fn|let|mut|pub|impl|struct|enum|trait|use|mod|match|if|else|for|while|loop|return|async|await|move|where|in|const|static|type|self|Self|crate|super|as)\b/g, '<span class="token-keyword">$1</span>');
      s = s.replace(/\b(String|str|Result|Option|Vec|HashMap|bool|usize|u8|u16|u32|u64|i32|i64|f32|f64)\b/g, '<span class="token-type">$1</span>');
      s = s.replace(/\b([A-Za-z_][A-Za-z0-9_]*)\s*(?=\()/g, '<span class="token-function">$1</span>');
    }

    return s;
  }

  /* ============================================================
     OPEN EDITOR / EXPLORER
     ============================================================ */

  async function openEditor() {
    if (!state.selectedResult?.path) return;
    try {
      await invoke("open_in_editor", { path: state.selectedResult.path });
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  async function openExplorer() {
    if (!state.selectedResult?.path) return;
    try {
      await invoke("open_in_explorer", { path: state.selectedResult.path });
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  /* ============================================================
     COPY CODE
     ============================================================ */

  async function copyCode() {
    const lines = $$("#codePreview .line-code");
    const code = lines.map(el => el.textContent).join("\n");
    if (!code) return;

    try {
      await invoke("copy_to_clipboard", { text: code });
      toast(t("copy"), "success");
    } catch (e) {
      try {
        await navigator.clipboard.writeText(code);
        toast(t("copy"), "success");
      } catch (_) {
        toast(errorText(e), "error");
      }
    }
  }

  /* ============================================================
     STATS
     ============================================================ */

  async function showStats() {
    // 🔒 Comprobar que hay proyecto seleccionado
    if (!state.path) {
      toast(t("noProjectSelected"), "error");
      renderNoProjectWelcome();
      return;
    }

    const content = $("#statsContent");
    if (!content) return;

    content.innerHTML = `<div class="code-empty">${t("loading")}</div>`;

    const modal = $("#statsModal");
    if (modal) modal.hidden = false;

    try {
      const data = await invoke("get_stats", { path: state.path });
      const stats = flattenStats(data);

      if (stats.length) {
        content.innerHTML = stats.map(([label, value]) => `
          <div class="stat-card">
            <div class="stat-label">${escapeHtml(label)}</div>
            <div class="stat-value">${escapeHtml(formatStat(value))}</div>
          </div>
        `).join("");
      } else {
        content.innerHTML = `<div class="code-empty">No hay estadísticas disponibles.</div>`;
      }
    } catch (e) {
      content.innerHTML = `<div class="code-empty">${escapeHtml(errorText(e))}</div>`;
    }
  }

  function flattenStats(data) {
    if (!data || typeof data !== "object") {
      return data == null ? [] : [["Resultado", data]];
    }

    const preferred = [
      ["Archivos", data.total_files ?? data.files],
      ["Líneas", data.total_lines ?? data.lines],
      ["Tamaño", data.total_size ?? data.size],
      ["Embeddings", data.has_embeddings ? "Sí" : "No"]
    ].filter(([, v]) => v != null);

    if (preferred.length) return preferred;

    return Object.entries(data)
      .filter(([, v]) => typeof v !== "object")
      .slice(0, 12);
  }

  /* ============================================================
     SETTINGS
     ============================================================ */

  function renderSettings() {
    const container = $("#settingsForm");
    if (!container) return;

    container.innerHTML = `
      <div class="setting-card">
        <div>
          <h3>Idioma / Language</h3>
          <p>Idioma de la interfaz / Interface language.</p>
        </div>
        <select id="cfgLang">
          <option value="es">Español</option>
          <option value="en">English</option>
        </select>
      </div>

      <div class="setting-card">
        <div>
          <h3>Tema / Theme</h3>
          <p>Modo claro u oscuro / Light or dark mode.</p>
        </div>
        <select id="cfgTheme">
          <option value="dark">Oscuro / Dark</option>
          <option value="light">Claro / Light</option>
        </select>
      </div>
    `;

    const langSelect = $("#cfgLang");
    if (langSelect) {
      langSelect.value = state.lang;
      langSelect.addEventListener("change", async e => {
        await setLanguage(e.target.value);
      });
    }

    const themeSelect = $("#cfgTheme");
    if (themeSelect) {
      themeSelect.value = state.theme;
      themeSelect.addEventListener("change", e => {
        state.theme = e.target.value;
        localStorage.setItem("semcode-theme", state.theme);
        applyTheme();
      });
    }
  }

  /* ============================================================
     FILTERS
     ============================================================ */

  function openFilterModal() {
    const modal = $("#filterModal");
    if (modal) modal.hidden = false;
  }

  function applyFilters() {
    const extensions = $("#filterExtensions")?.value.trim();
    const exclude = $("#filterExclude")?.value.trim();
    const minScore = Number($("#filterScore")?.value || 0);
    const caseSensitive = $("#filterCase")?.value === "true";

    state.filters = {};

    if (extensions) {
      state.filters.extensions = extensions.split(",").map(x => x.trim()).filter(Boolean);
    }
    if (exclude) {
      state.filters.exclude = exclude.split(",").map(x => x.trim()).filter(Boolean);
    }
    if (minScore > 0) {
      state.filters.min_score = minScore;
    }
    if (caseSensitive) {
      state.filters.case_sensitive = true;
    }

    const count = Object.keys(state.filters).length;
    const badge = $("#filterCount");
    if (badge) {
      badge.hidden = !count;
      badge.textContent = count;
    }

    const modal = $("#filterModal");
    if (modal) modal.hidden = true;

    if ($("#searchInput")?.value.trim()) runSearch();
  }

  function resetFilters() {
    state.filters = {};
    const exts = $("#filterExtensions");
    const excl = $("#filterExclude");
    const score = $("#filterScore");
    const cs = $("#filterCase");

    if (exts) exts.value = "";
    if (excl) excl.value = "";
    if (score) score.value = "0";
    if (cs) cs.value = "false";

    const badge = $("#filterCount");
    if (badge) badge.hidden = true;
  }

  /* ============================================================
     NEW ALIAS
     ============================================================ */

  async function newAlias() {
    const name = prompt("Nombre del alias:");
    if (!name) return;

    const query = prompt("Consulta del alias:", $("#searchInput")?.value || "");
    if (!query) return;

    try {
      await invoke("save_alias", { name, query, params: [] });
      await loadAliases();
      toast("Alias guardado.", "success");
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  /* ============================================================
     VIEW MANAGEMENT
     ============================================================ */

  function showView(view) {
    $$(".view").forEach(el => el.classList.remove("active-view"));
    const target = $(`#${view}View`);
    if (target) target.classList.add("active-view");

    $$(".nav-item").forEach(btn => {
      btn.classList.toggle("active", btn.dataset.view === view);
    });
  }

  function closeOverlays() {
    $$(".modal").forEach(m => { m.hidden = true; });
    const preview = $("#previewPanel");
    if (preview) preview.classList.remove("open");
  }

  /* ============================================================
     EVENTS
     ============================================================ */

  function setupEvents() {
    $("#searchBtn")?.addEventListener("click", runSearch);

    $("#searchInput")?.addEventListener("input", e => {
      const clearBtn = $("#clearSearchBtn");
      if (clearBtn) clearBtn.hidden = !e.target.value;
    });

    $("#searchInput")?.addEventListener("keydown", e => {
      if (e.key === "Enter") {
        e.preventDefault();
        runSearch();
        return;
      }
      if (e.key === "ArrowDown" && state.results.length) {
        e.preventDefault();
        selectResult(Math.min(state.selectedIndex + 1, state.results.length - 1));
        return;
      }
      if (e.key === "ArrowUp" && state.results.length) {
        e.preventDefault();
        selectResult(Math.max(state.selectedIndex - 1, 0));
      }
    });

    $("#clearSearchBtn")?.addEventListener("click", () => {
      $("#searchInput").value = "";
      $("#clearSearchBtn").hidden = true;
      $("#searchInput").focus();
    });

    $$(".mode-btn").forEach(btn => {
      btn.addEventListener("click", () => {
        $$(".mode-btn").forEach(b => b.classList.remove("active"));
        btn.classList.add("active");
        state.mode = btn.dataset.mode;
      });
    });

    $$(".suggestions button").forEach(btn => {
      btn.addEventListener("click", () => {
        $("#searchInput").value = btn.dataset.query;
        $("#clearSearchBtn").hidden = false;
        runSearch();
      });
    });

    $("#filterBtn")?.addEventListener("click", openFilterModal);
    $("#applyFiltersBtn")?.addEventListener("click", applyFilters);
    $("#resetFiltersBtn")?.addEventListener("click", resetFilters);

    $("#pathBtn")?.addEventListener("click", chooseProject);
    $("#addProjectBtn")?.addEventListener("click", chooseProject);

    $("#newAliasBtn")?.addEventListener("click", newAlias);

    $("#clearHistoryBtn")?.addEventListener("click", async () => {
      try {
        await invoke("clear_history");
        await loadHistory();
        toast("Historial limpiado.", "success");
      } catch (e) {
        toast(errorText(e), "error");
      }
    });

    $("#lastSearchBtn")?.addEventListener("click", async () => {
      try {
        const data = await invoke("get_last_search");
        const item = data?.query ? data : (data?.last_search || data);

        if (item?.query) {
          $("#searchInput").value = item.query;
          if (item.path) setPath(item.path);
          runSearch();
        } else {
          toast("No hay una última búsqueda.", "info");
        }
      } catch (e) {
        toast(errorText(e), "error");
      }
    });

    $("#themeToggle")?.addEventListener("click", () => {
      state.theme = state.theme === "dark" ? "light" : "dark";
      localStorage.setItem("semcode-theme", state.theme);
      applyTheme();
    });

    $("#languageToggle")?.addEventListener("click", () => {
      setLanguage(state.lang === "es" ? "en" : "es");
    });

    $("#closePreviewBtn")?.addEventListener("click", () => {
      $("#previewPanel")?.classList.remove("open");
    });

    $("#copyCodeBtn")?.addEventListener("click", copyCode);
    $("#openEditorBtn")?.addEventListener("click", openEditor);
    $("#openExplorerBtn")?.addEventListener("click", openExplorer);

    $("#shortcutsBtn")?.addEventListener("click", () => {
      $("#shortcutsModal").hidden = false;
    });

    $$("[data-action=index]").forEach(btn => {
      btn.addEventListener("click", () => indexProject());
    });

    $$("[data-action=stats]").forEach(btn => {
      btn.addEventListener("click", showStats);
    });

    $$("[data-view]").forEach(btn => {
      btn.addEventListener("click", () => showView(btn.dataset.view));
    });

    $$(".modal-close, .modal-backdrop").forEach(el => {
      el.addEventListener("click", closeOverlays);
    });

    document.addEventListener("keydown", e => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        showView("search");
        $("#searchInput")?.focus();
        $("#searchInput")?.select();
        return;
      }
      if (e.key === "Escape") {
        closeOverlays();
      }
    });

    $("#minimizeBtn")?.addEventListener("click", async () => {
      try {
        const w = window.__TAURI__?.window?.getCurrentWindow;
        if (w) await w().minimize();
      } catch (_) {}
    });

    $("#maximizeBtn")?.addEventListener("click", async () => {
      try {
        const w = window.__TAURI__?.window?.getCurrentWindow;
        if (w) await w().toggleMaximize();
      } catch (_) {}
    });

    $("#closeBtn")?.addEventListener("click", async () => {
      try {
        const w = window.__TAURI__?.window?.getCurrentWindow;
        if (w) await w().close();
      } catch (_) {}
    });
  }

  /* ============================================================
     HELPERS
     ============================================================ */

  function escapeHtml(value) {
    return String(value ?? "").replace(/[&<>"']/g, character => {
      const map = {
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;"
      };
      return map[character];
    });
  }

  function basename(path) {
    if (!path) return "";
    return String(path).replace(/[\\/]+$/, "").split(/[\\/]/).pop() || path;
  }

  function languageFromPath(path) {
    const ext = String(path).split(".").pop()?.toLowerCase();
    const map = {
      rs: "rust", js: "javascript", ts: "typescript",
      html: "html", css: "css", json: "json", toml: "toml",
      py: "python", go: "go", java: "java", c: "c", cpp: "cpp", h: "c"
    };
    return map[ext] || ext || "code";
  }

  function formatNumber(number) {
    return Number(number).toLocaleString(state.lang === "es" ? "es-ES" : "en-US");
  }

  function formatStat(value) {
    return typeof value === "number" ? formatNumber(value) : String(value);
  }

  function errorText(error) {
    return error?.message || error?.toString?.() || "Error desconocido";
  }

  /* ============================================================
     INIT
     ============================================================ */

  applyTheme();
  applyTranslations();
  setupEvents();
  loadBootstrap();

})();
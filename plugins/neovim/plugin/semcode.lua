-- Plugin de Neovim para semcode-search
-- Autor: lecodev-26
-- Licencia: MIT

local M = {}

-- Función para ejecutar semcode-search y mostrar el resultado
local function run_semcode(args)
    local cmd = { 'semcode-search' }
    for _, arg in ipairs(args) do
        table.insert(cmd, arg)
    end

    local output = vim.fn.system(cmd)
    local exit_code = vim.v.shell_error

    if exit_code ~= 0 then
        vim.notify('Error en semcode-search: ' .. output, vim.log.levels.ERROR)
        return nil
    end

    return output
end

-- Búsqueda normal
function M.search()
    vim.ui.input({ prompt = '🔍 Buscar en el código: ' }, function(query)
        if not query or query == '' then
            return
        end

        local output = run_semcode({ 'search', '--query', query, '--path', '.' })
        if not output then
            return
        end

        -- Abrir en un buffer flotante
        local buf = vim.api.nvim_create_buf(false, true)
        local lines = vim.split(output, '\n')
        vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

        local width = math.floor(vim.o.columns * 0.8)
        local height = math.floor(vim.o.lines * 0.8)
        local col = math.floor((vim.o.columns - width) / 2)
        local row = math.floor((vim.o.lines - height) / 2)

        local win = vim.api.nvim_open_win(buf, true, {
            relative = 'editor',
            width = width,
            height = height,
            col = col,
            row = row,
            style = 'minimal',
            border = 'rounded',
            title = ' 🔍 Semcode Search: ' .. query .. ' ',
            title_pos = 'center'
        })

        vim.api.nvim_buf_set_option(buf, 'modifiable', false)
        vim.api.nvim_buf_set_keymap(buf, 'n', 'q', ':close<CR>', { noremap = true, silent = true })
    end)
end

-- Búsqueda con IA
function M.search_ai()
    vim.ui.input({ prompt = '🧠 Buscar con IA: ' }, function(query)
        if not query or query == '' then
            return
        end

        local output = run_semcode({ 'search', '--query', query, '--path', '.', '--ai' })
        if not output then
            return
        end

        local buf = vim.api.nvim_create_buf(false, true)
        local lines = vim.split(output, '\n')
        vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

        local width = math.floor(vim.o.columns * 0.8)
        local height = math.floor(vim.o.lines * 0.8)
        local col = math.floor((vim.o.columns - width) / 2)
        local row = math.floor((vim.o.lines - height) / 2)

        local win = vim.api.nvim_open_win(buf, true, {
            relative = 'editor',
            width = width,
            height = height,
            col = col,
            row = row,
            style = 'minimal',
            border = 'rounded',
            title = ' 🧠 Semcode AI: ' .. query .. ' ',
            title_pos = 'center'
        })

        vim.api.nvim_buf_set_option(buf, 'modifiable', false)
        vim.api.nvim_buf_set_keymap(buf, 'n', 'q', ':close<CR>', { noremap = true, silent = true })
    end)
end

-- Indexar proyecto
function M.index()
    vim.notify('📁 Indexando proyecto...', vim.log.levels.INFO)
    vim.fn.jobstart({ 'semcode-search', 'index', '--path', '.' }, {
        on_exit = function(_, code)
            if code == 0 then
                vim.notify('✅ Proyecto indexado', vim.log.levels.INFO)
            else
                vim.notify('❌ Error al indexar', vim.log.levels.ERROR)
            end
        end
    })
end

-- Mostrar stats
function M.stats()
    local output = run_semcode({ 'stats' })
    if not output then
        return
    end

    local buf = vim.api.nvim_create_buf(false, true)
    local lines = vim.split(output, '\n')
    vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

    local width = math.floor(vim.o.columns * 0.6)
    local height = math.floor(vim.o.lines * 0.6)
    local col = math.floor((vim.o.columns - width) / 2)
    local row = math.floor((vim.o.lines - height) / 2)

    local win = vim.api.nvim_open_win(buf, true, {
        relative = 'editor',
        width = width,
        height = height,
        col = col,
        row = row,
        style = 'minimal',
        border = 'rounded',
        title = ' 📊 Semcode Stats ',
        title_pos = 'center'
    })

    vim.api.nvim_buf_set_option(buf, 'modifiable', false)
    vim.api.nvim_buf_set_keymap(buf, 'n', 'q', ':close<CR>', { noremap = true, silent = true })
end

-- Registrar comandos
vim.api.nvim_create_user_command('SemcodeSearch', M.search, {})
vim.api.nvim_create_user_command('SemcodeSearchAI', M.search_ai, {})
vim.api.nvim_create_user_command('SemcodeIndex', M.index, {})
vim.api.nvim_create_user_command('SemcodeStats', M.stats, {})

return M
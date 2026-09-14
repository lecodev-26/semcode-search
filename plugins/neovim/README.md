# Semcode Search para Neovim

Plugin de Neovim para usar `semcode-search` directamente desde el editor.

## 📋 Requisitos

1. **Tener instalado `semcode-search`**:
   ```bash
   cargo install semcode-search
Neovim 0.7+ (con soporte de vim.ui.input).

## 🚀 Instalación
Con lazy.nvim
lua
{
    'lecodev-26/semcode-search',
    config = function()
        require('plugin.semcode')
    end,
    keys = {
        { '<leader>ss', '<cmd>SemcodeSearch<cr>', desc = 'Semcode Search' },
        { '<leader>sa', '<cmd>SemcodeSearchAI<cr>', desc = 'Semcode AI Search' },
        { '<leader>si', '<cmd>SemcodeIndex<cr>', desc = 'Semcode Index' },
        { '<leader>st', '<cmd>SemcodeStats<cr>', desc = 'Semcode Stats' },
    }
}
Manual
Copia la carpeta plugins/neovim/ a tu configuración:


cp -r plugins/neovim/plugin ~/.config/nvim/lua/semcode/
Añade a tu init.lua:

lua
require('semcode.semcode')

## 🎯 Comandos
Comando	Qué hace
:SemcodeSearch	Búsqueda por texto
:SemcodeSearchAI	Búsqueda con IA
:SemcodeIndex	Indexar proyecto
:SemcodeStats	Ver estadísticas

## ⌨️ Atajos recomendados

lua
vim.keymap.set('n', '<leader>ss', '<cmd>SemcodeSearch<cr>', { desc = 'Semcode Search' })
vim.keymap.set('n', '<leader>sa', '<cmd>SemcodeSearchAI<cr>', { desc = 'Semcode AI Search' })
vim.keymap.set('n', '<leader>si', '<cmd>SemcodeIndex<cr>', { desc = 'Semcode Index' })
vim.keymap.set('n', '<leader>st', '<cmd>SemcodeStats<cr>', { desc = 'Semcode Stats' })

## 📖 Ejemplos

Búsqueda normal
Pulsa <leader>ss.

Escribe fn main.

Se abre una ventana flotante con los resultados.

Pulsa q para cerrar.

Búsqueda con IA
Pulsa <leader>sa.

Escribe función para validar emails.

Encuentra funciones por significado.

## 📄 Licencia

MIT
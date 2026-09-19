; ============================================================
; semcode-parser — queries Tree-sitter para Rust
; ============================================================

; ---------- FUNCIONES LIBRES ----------
(function_item
  name: (identifier) @name
  parameters: (parameters) @params) @function

; ---------- STRUCTS ----------
(struct_item
  name: (type_identifier) @name) @struct

; ---------- ENUMS ----------
(enum_item
  name: (type_identifier) @name) @enum

; ---------- TRAITS ----------
(trait_item
  name: (type_identifier) @name) @trait

; ---------- MÉTODOS EN IMPLS ----------
(impl_item
  type: (_) @impl_type
  (declaration_list
    (function_item
      name: (identifier) @name
      parameters: (parameters) @params) @method))

; ---------- IMPORTS / USE ----------
(use_declaration
  argument: (_) @path) @use

; ---------- CONSTANTES ----------
(const_item
  name: (identifier) @name
  value: (_) @value) @const

; ---------- ESTÁTICAS ----------
(static_item
  name: (identifier) @name
  value: (_) @value) @static

; ---------- TYPE ALIASES ----------
(type_item
  name: (type_identifier) @name) @type_alias

; ---------- MÓDULOS ----------
(mod_item
  name: (identifier) @name) @module

; ---------- MACROS ----------
(macro_definition
  name: (identifier) @name) @macro

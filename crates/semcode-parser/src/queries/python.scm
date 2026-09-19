; ============================================================
; semcode-parser — queries Tree-sitter para Python
; ============================================================

; ---------- FUNCIONES ----------
(function_definition
  name: (identifier) @name
  parameters: (parameters) @params) @function

; ---------- CLASES ----------
(class_definition
  name: (identifier) @name) @class

; ---------- MÉTODOS ----------
(class_definition
  (block
    (function_definition
      name: (identifier) @name
      parameters: (parameters) @params) @method))

; ---------- IMPORTS ----------
(import_statement
  name: (dotted_name) @path) @import

(import_from_statement
  module_name: (dotted_name) @path) @import_from

; ============================================================
; semcode-parser — queries Tree-sitter para JavaScript
; ============================================================

; ---------- FUNCIONES DECLARATIVAS ----------
(function_declaration
  name: (identifier) @name
  parameters: (formal_parameters) @params) @function

; ---------- ARROW FUNCTIONS ----------
(variable_declarator
  name: (identifier) @name
  value: (arrow_function
    parameters: (formal_parameters) @params)) @arrow_function

; ---------- FUNCIONES EXPRESIÓN ----------
(variable_declarator
  name: (identifier) @name
  value: (function_expression
    parameters: (formal_parameters) @params)) @function_expression

; ---------- CLASES ----------
(class_declaration
  name: (identifier) @name) @class

; ---------- MÉTODOS DE CLASE ----------
(class_declaration
  (class_body
    (method_definition
      name: (property_identifier) @name
      parameters: (formal_parameters) @params) @method))

; ---------- IMPORTS ----------
(import_statement
  source: (string) @path) @import

; ---------- EXPORTS ----------
(export_statement
  declaration: (_) @exported) @export

; ---------- VARIABLES ----------
(program
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name
      value: (_) @value) @variable))

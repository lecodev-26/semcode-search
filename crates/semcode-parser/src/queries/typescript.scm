; ============================================================
; semcode-parser — queries Tree-sitter para TypeScript
; ============================================================

; ---------- FUNCIONES ----------
(function_declaration
  name: (identifier) @name
  parameters: (formal_parameters) @params) @function

; ---------- ARROW FUNCTIONS ----------
(variable_declarator
  name: (identifier) @name
  value: (arrow_function
    parameters: (formal_parameters) @params)) @arrow_function

; ---------- CLASES ----------
(class_declaration
  name: (type_identifier) @name) @class

; ---------- MÉTODOS DE CLASE ----------
(class_declaration
  (class_body
    (method_definition
      name: (property_identifier) @name
      parameters: (formal_parameters) @params) @method))

; ---------- INTERFACES ----------
(interface_declaration
  name: (type_identifier) @name) @interface

; ---------- TYPE ALIASES ----------
(type_alias_declaration
  name: (type_identifier) @name
  value: (_) @type) @type_alias

; ---------- ENUMS ----------
(enum_declaration
  name: (identifier) @name) @enum

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

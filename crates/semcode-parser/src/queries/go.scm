; ============================================================
; semcode-parser — queries Tree-sitter para Go
; ============================================================

; ---------- FUNCIONES ----------
(function_declaration
  name: (identifier) @name
  parameters: (parameter_list) @params) @function

; ---------- MÉTODOS ----------
(method_declaration
  receiver: (parameter_list) @receiver
  name: (field_identifier) @name
  parameters: (parameter_list) @params) @method

; ---------- STRUCTS ----------
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (struct_type)) @struct)

; ---------- INTERFACES ----------
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (interface_type)) @interface)

; ---------- TYPE ALIASES ----------
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (_) @type) @type_alias)

; ---------- CONSTANTES ----------
(const_declaration
  (const_spec
    name: (identifier) @name
    value: (_) @value) @const)

; ---------- VARIABLES ----------
(var_declaration
  (var_spec
    name: (identifier) @name
    value: (_) @value) @var)

; ---------- IMPORTS ----------
(import_declaration
  (import_spec
    path: (interpreted_string_literal) @path) @import)

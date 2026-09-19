; ============================================================
; semcode-parser — queries Tree-sitter para JSON
; ============================================================
; JSON no tiene funciones ni clases, pero podemos extraer
; las claves de objetos como "símbolos" para buscarlas.
; Extraemos claves de nivel 1 (raíz) y nivel 2 (anidadas).
; ============================================================

; ---------- CLAVES DE NIVEL 1 ----------
(document
  (object
    (pair
      key: (string) @name
      value: (_) @value) @pair))

; ---------- CLAVES DE NIVEL 2 (objetos anidados) ----------
(document
  (object
    (pair
      key: (string) @parent_name
      value: (object
        (pair
          key: (string) @name
          value: (_) @value) @nested_pair))))

; ---------- ARRAYS DE OBJETOS ----------
(document
  (object
    (pair
      key: (string) @array_name
      value: (array
        (object
          (pair
            key: (string) @field_name
            value: (_) @field_value) @field) @item) @array)))

# Semcode Search para VSCode

Extensión de VSCode para usar `semcode-search` directamente desde el editor.

## 📋 Requisitos

1. **Tener instalado `semcode-search`**:
   ```bash
   cargo install semcode-search
   ```

2. **TTenerlo en el PATH del sistema.**T

## 🚀 Instalación

1. Copia la carpeta plugins/vscode/ a tu carpeta de extensiones de VSCode:

Windows: %USERPROFILE%\.vscode\extensions\semcode-search-vscode

Linux/macOS: ~/.vscode/extensions/semcode-search-vscode

2. **Reinicia VSCode.**

3. **Abre la paleta de comandos (Ctrl + Shift + P) y escribe Semcode.**

## 🎯 Comandos
```marckdown
Comando	Atajo	Qué hace
Semcode: Search	Ctrl + Shift + F	Búsqueda por texto
Semcode: Search with AI	-	Búsqueda con IA (embeddings)
Semcode: Index Project	-	Indexar el proyecto actual
Semcode: Show Stats	-	Ver estadísticas
```

## 📖 Ejemplos

**Búsqueda normal**

1. Pulsa Ctrl + Shift + F.

2. Escribe fn main.

3. Se abre un panel con los resultados.

## Búsqueda con IA

1. Ctrl + Shift + P → Semcode: Search with AI.

2. Escribe función para validar emails.

3. Encuentra funciones aunque no contengan esas palabras exactas.

## Indexar proyecto

1. Ctrl + Shift + P → Semcode: Index Project.

2. Aparece una notificación de progreso.

3. Cuando termina, la caché está lista.

## ⚙️ Configuración
Puedes configurar la ruta del binario en settings.json:
```bash
json
{
  "semcode.binaryPath": "C:\\Users\\tu-usuario\\.cargo\\bin\\semcode-search.exe"
}
```
## 🐛 Problemas conocidos

1. "semcode-search no se encuentra" → Asegúrate de que está en el PATH.

2. No hay carpeta abierta → Abre una carpeta de proyecto en VSCode.

## 📄 Licencia

MIT
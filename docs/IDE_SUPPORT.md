# IDE Support and Type Hints for nu_plugin_secret

This guide helps developers configure their IDEs for optimal development experience when using the `nu_plugin_secret` plugin.

## 🎯 Overview

The `nu_plugin_secret` plugin provides 8 secret types and 12 commands for handling sensitive data in Nushell. While Nushell's type system is dynamic, proper IDE configuration can provide better developer experience through:

- Command completion and suggestions
- Type hints and documentation
- Error detection and linting
- Integration with development workflows

## 🔧 IDE Configuration

### Visual Studio Code

#### Nushell Extension Setup

1. **Install Nushell Extension**
   ```bash
   # Install the official Nushell extension
   code --install-extension nushell.nushell-lang
   ```

2. **Configure Nushell LSP**
   Add to `settings.json`:
   ```json
   {
     "nushell.lsp.enabled": true,
     "nushell.lsp.maxNumberOfProblems": 100,
     "nushell.enableCodelens": true
   }
   ```

3. **Plugin-specific Snippets**
   Create `.vscode/snippets/nushell.json`:
   ```json
   {
     "Secret String Wrap": {
       "prefix": "secret-string",
       "body": ["$${1:value} | secret wrap"],
       "description": "Wrap string value as SecretString"
     },
     "Secret Int Wrap": {
       "prefix": "secret-int", 
       "body": ["$${1:value} | secret wrap"],
       "description": "Wrap integer value as SecretInt"
     },
     "Secret Record Wrap": {
       "prefix": "secret-record",
       "body": ["$${1:record} | secret wrap"],
       "description": "Wrap record as SecretRecord"
     },
     "Secret Unwrap": {
       "prefix": "secret-unwrap",
       "body": ["$${1:secret} | secret unwrap"],
       "description": "Unwrap secret value (use carefully)"
     },
     "Secret Validate": {
       "prefix": "secret-validate", 
       "body": ["$${1:value} | secret validate"],
       "description": "Check if value is a secret type"
     },
     "Secret Type Check": {
       "prefix": "secret-typeof",
       "body": ["$${1:secret} | secret type-of"],
       "description": "Get underlying type of secret"
     }
   }
   ```

4. **Error Highlighting**
   Add custom problem matchers in `.vscode/tasks.json`:
   ```json
   {
     "version": "2.0.0",
     "tasks": [
       {
         "label": "Check Secret Types",
         "type": "shell",
         "command": "nu",
         "args": ["-c", "source ${workspaceFolder}/scripts/check-secrets.nu"],
         "group": {
           "kind": "test",
           "isDefault": true
         },
         "problemMatcher": {
           "owner": "nushell-secrets",
           "fileLocation": "relative",
           "pattern": {
             "regexp": "^(.*):(\\d+):(\\d+):\\s+(warning|error):\\s+(.*)$",
             "file": 1,
             "line": 2,
             "column": 3,
             "severity": 4,
             "message": 5
           }
         }
       }
     ]
   }
   ```

#### Type Hints in Comments

Use structured comments to provide type information:

```nushell
# Type: SecretString - API key for external service
let api_key = ($env.API_KEY | secret wrap)

# Type: SecretRecord - Database credentials with multiple fields
let db_config = {
    host: "localhost",
    username: "user",
    password: "pass",
    port: 5432
} | secret wrap

# Type: SecretList - Array of backup codes
let backup_codes = ["code1", "code2", "code3"] | secret wrap

# Function that accepts SecretString and returns processed data
# @param api_key: SecretString - Must be wrapped secret type
# @returns: any - API response data
def call_api [api_key: any] {
    if not ($api_key | secret validate) {
        error make {msg: "api_key must be SecretString type"}
    }
    # ... function implementation
}
```

### Neovim Configuration

#### Nushell LSP Setup

1. **Install Nushell LSP support**
   ```lua
   -- In your init.lua or relevant config file
   require('lspconfig').nushell.setup{
     cmd = { 'nu', '--lsp' },
     filetypes = { 'nu' },
     single_file_support = true,
   }
   ```

2. **Custom Snippets with LuaSnip**
   ```lua
   local ls = require("luasnip")
   local s = ls.snippet
   local t = ls.text_node
   local i = ls.insert_node
   
   ls.add_snippets("nu", {
     s("swrap", {
       i(1, "value"), t(" | secret wrap")
     }),
     s("swrap-any", {
       i(1, "value"), t(" | secret wrap") 
     }),
     s("swrap-unified", {
       i(1, "data"), t(" | secret wrap")
     }),
     s("sunwrap", {
       i(1, "secret"), t(" | secret unwrap")
     }),
   })
   ```

3. **Custom Keybindings**
   ```lua
   -- Quick secret type operations
   vim.keymap.set('n', '<leader>ss', function()
     vim.ui.input({prompt = 'Wrap as secret: '}, function(choice)
       if choice then
         local cursor = vim.api.nvim_win_get_cursor(0)
         local line = vim.api.nvim_get_current_line()
         local new_line = line .. ' | secret wrap-' .. choice
         vim.api.nvim_set_current_line(new_line)
       end
     end)
   end, {desc = 'Wrap as secret type'})
   ```

### Emacs Configuration

#### Nushell Mode Setup

```elisp
;; Nushell mode configuration
(use-package nushell-mode
  :mode "\\.nu\\'"
  :config
  (setq nushell-indent-offset 2))

;; Custom snippets for secret types
(use-package yasnippet
  :config
  (yas-global-mode 1))

;; Add to nushell-mode snippets
;; ~/.emacs.d/snippets/nushell-mode/secret-string
;; # -*- mode: snippet -*-
;; # name: secret wrap
;; # key: sws
;; # --
;; ${1:value} | secret wrap

;; Custom functions for secret type management
(defun nushell-wrap-region-as-secret (type)
  "Wrap selected region as secret type"
  (interactive "sSecret type: ")
  (when (use-region-p)
    (let ((region-text (buffer-substring-no-properties 
                       (region-beginning) (region-end))))
      (delete-region (region-beginning) (region-end))
      (insert (format "%s | secret wrap-%s" region-text type)))))
```

## 🎨 Syntax Highlighting

### Custom Highlighting Rules

#### VS Code Theme Customization
Add to `settings.json`:
```json
{
  "editor.tokenColorCustomizations": {
    "textMateRules": [
      {
        "scope": "keyword.control.nushell.secret",
        "settings": {
          "foreground": "#ff6b6b",
          "fontStyle": "bold"
        }
      },
      {
        "scope": "string.quoted.double.nushell.redacted",
        "settings": {
          "foreground": "#888888", 
          "fontStyle": "italic"
        }
      }
    ]
  }
}
```

#### Vim/Neovim Syntax Highlighting
Add to `after/syntax/nu.vim`:
```vim
" Highlight secret commands
syn keyword nuSecretCommand secret contained
syn keyword nuSecretOperation wrap unwrap validate type-of info contained

" Highlight redacted content
syn match nuRedacted /<redacted:\w\+>/ contained

" Apply highlighting
hi def link nuSecretCommand Keyword
hi def link nuSecretOperation Function  
hi def link nuRedacted Comment
```

## 🧹 Linting and Code Quality

### Custom Lint Rules

Create `scripts/check-secrets.nu` for security linting:

```nushell
#!/usr/bin/env nu

# Security linter for secret type usage
def main [file_path: path] {
    let content = open $file_path | lines
    
    # Check for potential security issues
    mut issues = []
    
    # Rule 1: Detect unwrapped secrets in echo/print statements
    let echo_issues = $content 
        | enumerate 
        | where item =~ "echo.*unwrap|print.*unwrap"
        | each { |line|
            {
                line: ($line.index + 1),
                severity: "warning", 
                message: "Potential secret exposure in output statement",
                rule: "no-unwrap-in-output"
            }
        }
    
    # Rule 2: Detect plain text that should be secrets
    let plaintext_secrets = $content
        | enumerate
        | where item =~ "api_key|password|secret|token" and item !~ "secret wrap"
        | each { |line|
            {
                line: ($line.index + 1),
                severity: "info",
                message: "Consider wrapping sensitive data as secret type", 
                rule: "wrap-sensitive-data"
            }
        }
        
    # Rule 3: Check for proper secret validation in functions
    let validation_issues = $content
        | enumerate 
        | where item =~ "def.*\[.*secret.*\]" and item !~ "secret validate"
        | each { |line|
            {
                line: ($line.index + 1),
                severity: "warning",
                message: "Function accepting secrets should validate input types",
                rule: "validate-secret-params"
            }
        }
    
    # Combine all issues
    $issues = ([$echo_issues, $plaintext_secrets, $validation_issues] | flatten)
    
    # Output in standard format
    $issues | each { |issue|
        echo $"($file_path):($issue.line):0: ($issue.severity): ($issue.message) [($issue.rule)]"
    }
    
    # Return exit code based on severity
    let errors = ($issues | where severity == "error" | length)
    if $errors > 0 {
        exit 1
    }
}
```

### Integration with Git Hooks

Create `.git/hooks/pre-commit`:
```bash
#!/bin/bash

# Check all Nushell files for secret type security issues
echo "Running secret type security checks..."

# Find all .nu files
NU_FILES=$(find . -name "*.nu" -not -path "./.git/*")

if [ -n "$NU_FILES" ]; then
    for file in $NU_FILES; do
        echo "Checking $file..."
        nu scripts/check-secrets.nu "$file"
        if [ $? -ne 0 ]; then
            echo "❌ Security issues found in $file"
            echo "Fix the issues above before committing."
            exit 1
        fi
    done
fi

echo "✅ All secret type security checks passed"
```

## 🔍 Debugging and Development

### Debug Configuration

#### VS Code Debug Setup
Create `.vscode/launch.json`:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch", 
            "name": "Debug Nushell with Secret Plugin",
            "program": "/usr/local/bin/nu",
            "args": ["-c", "plugin use secret; source ${workspaceFolder}/test-script.nu"],
            "cwd": "${workspaceFolder}",
            "environment": [
                {
                    "name": "RUST_LOG", 
                    "value": "debug"
                },
                {
                    "name": "RUST_BACKTRACE",
                    "value": "1"
                }
            ]
        }
    ]
}
```

#### Development Helper Functions

Create `dev-helpers.nu`:
```nushell
# Development helpers for secret type debugging

# Print secret type information without exposing content
def debug-secret [secret: any] {
    if ($secret | secret validate) {
        let secret_type = ($secret | secret type-of)
        echo $"Secret Type: ($secret_type)"
        echo $"Display: ($secret)"
        echo $"Is Valid: true"
    } else {
        echo "Not a secret type"
        echo $"Actual Type: ($secret | describe)"
        echo $"Value: ($secret)"
    }
}

# Test all secret type operations
def test-secret-operations [] {
    echo "Testing SecretString..."
    let s_string = "test" | secret wrap
    debug-secret $s_string
    
    echo "Testing SecretInt..."
    let s_int = 42 | secret wrap  
    debug-secret $s_int
    
    echo "Testing SecretRecord..."
    let s_record = {key: "value"} | secret wrap
    debug-secret $s_record
}

# Validate secret type security properties
def validate-security [secret: any] {
    echo "Security Validation Report:"
    echo $"1. Display Protection: ($secret)"
    echo $"2. Type Validation: ($secret | secret validate)"
    echo $"3. Type Information: ($secret | secret type-of)"
    echo "4. JSON Serialization: "
    echo ($secret | to json)
    echo "5. Debug Representation:"
    echo ($secret | debug)
}
```

## 🎓 Learning and Documentation

### Interactive Learning Scripts

Create `examples/interactive-tutorial.nu`:
```nushell
#!/usr/bin/env nu

# Interactive tutorial for secret types
def main [] {
    echo "🔐 Welcome to nu_plugin_secret Interactive Tutorial!"
    echo ""
    
    tutorial-basic-types
    tutorial-advanced-usage
    tutorial-security-best-practices
}

def tutorial-basic-types [] {
    echo "📚 Part 1: Basic Secret Types"
    echo ""
    
    echo "Let's create a secret string:"
    echo '> let api_key = "sk-1234567890" | secret wrap'
    let api_key = "sk-1234567890" | secret wrap
    echo $"Result: ($api_key)"
    echo ""
    
    input "Press Enter to continue..."
    
    echo "Now let's check its type:"
    echo '> $api_key | secret type-of'  
    echo ($api_key | secret type-of)
    echo ""
    
    echo "Validate it's a secret:"
    echo '> $api_key | secret validate'
    echo ($api_key | secret validate)
}

def tutorial-advanced-usage [] {
    echo "🚀 Part 2: Advanced Usage Patterns"
    
    echo "Creating mixed sensitivity record:"
    let config = {
        host: "api.example.com",
        api_key: ("secret123" | secret wrap),
        timeout: 30
    }
    echo $"Config: ($config)"
}

def tutorial-security-best-practices [] {
    echo "🛡️  Part 3: Security Best Practices"
    
    echo "❌ Don't do this - exposes secret:"
    echo '> echo $"API key is: {($secret | secret unwrap)}"'
    
    echo "✅ Do this instead:"  
    echo '> echo $"API key is: {$secret}"'
}
```

### Code Templates

#### Project Template Structure
```
project-template/
├── .vscode/
│   ├── settings.json
│   ├── snippets/nushell.json
│   └── launch.json
├── scripts/
│   ├── check-secrets.nu
│   └── dev-helpers.nu
├── src/
│   └── main.nu
└── README.md
```

#### Function Template
```nushell
# Template for functions that work with secret types
# Copy and modify as needed

# @param secret_value: any - Must be a secret type
# @returns: any - Processed result  
def process_secret_template [secret_value: any] {
    # 1. Validate input is a secret type
    if not ($secret_value | secret validate) {
        error make {
            msg: "Expected secret type",
            help: "Use: $value | secret wrap-<type>"
        }
    }
    
    # 2. Get type information if needed
    let secret_type = ($secret_value | secret type-of)
    
    # 3. Only unwrap when absolutely necessary
    # let unwrapped = $secret_value | secret unwrap
    
    # 4. Return result (re-wrap if returning sensitive data)
    $secret_value  # or process and re-wrap
}
```

## 📋 IDE Integration Checklist

### Setup Checklist
- [ ] Nushell language extension installed
- [ ] LSP configured and working
- [ ] Custom snippets added for secret operations
- [ ] Syntax highlighting configured for secret types
- [ ] Linting rules implemented and integrated
- [ ] Debug configuration setup
- [ ] Git hooks configured for security checks
- [ ] Development helper functions available

### Usage Checklist  
- [ ] Use type hints in comments for better documentation
- [ ] Leverage IDE snippets for common secret operations
- [ ] Run security linting before commits
- [ ] Use debug helpers for development and testing
- [ ] Validate secret type usage with custom tools
- [ ] Follow consistent naming conventions for secret variables

### Maintenance Checklist
- [ ] Keep IDE extensions updated
- [ ] Update snippets when new commands are added
- [ ] Review and update linting rules regularly
- [ ] Update syntax highlighting for new features
- [ ] Test debug configurations with new plugin versions
- [ ] Update documentation templates as needed

---

**This IDE configuration enhances developer productivity while maintaining security best practices when working with the nu_plugin_secret plugin.**
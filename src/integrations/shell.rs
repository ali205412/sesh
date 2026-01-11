//! Shell integration
//!
//! Provides shell hooks, abbreviations, and auto-suggestions.
//! Special focus on fish shell with proper function autoloading.

use std::path::Path;

/// Generate shell integration script for bash
pub fn bash_integration() -> &'static str {
    r#"
# sesh shell integration for bash
# Add to ~/.bashrc or source from ~/.config/sesh/sesh.bash

# Auto-suggest session when cd'ing to a project directory
_sesh_chpwd_hook() {
    if [ -f ".sesh" ] || [ -f ".screen" ]; then
        local session_name=$(basename "$PWD")
        if screen -ls 2>/dev/null | grep -q "\.$session_name[[:space:]]"; then
            echo -e "\033[0;36msesh:\033[0m session '\033[1m$session_name\033[0m' exists. Press \033[1mCtrl+s\033[0m to attach."
        fi
    fi
}

# Add to PROMPT_COMMAND for bash
if [[ ! "$PROMPT_COMMAND" =~ "_sesh_chpwd_hook" ]]; then
    PROMPT_COMMAND="_sesh_chpwd_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi

# Keybinding: Ctrl+s to launch sesh
bind '"\C-s":"sesh\n"'

# sesh function wrapper with better completion
sesh() {
    if [ "$1" = "attach" ] && [ -z "$2" ]; then
        command sesh
    else
        command sesh "$@"
    fi
}

# Bash completion for sesh
_sesh_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"

    case "$prev" in
        sesh)
            COMPREPLY=($(compgen -W "list new attach detach kill start templates windows" -- "$cur"))
            ;;
        attach|detach|kill|windows)
            local sessions=$(screen -ls 2>/dev/null | grep -oP '\d+\.\K[^\s]+' 2>/dev/null)
            COMPREPLY=($(compgen -W "$sessions" -- "$cur"))
            ;;
        start)
            local templates=$(find ~/.config/sesh/templates -name "*.yaml" -o -name "*.yml" 2>/dev/null | xargs -I{} basename {} .yaml | sed 's/\.yml$//')
            COMPREPLY=($(compgen -W "$templates" -- "$cur"))
            ;;
    esac
}
complete -F _sesh_completions sesh
"#
}

/// Generate shell integration script for zsh
pub fn zsh_integration() -> &'static str {
    r#"
# sesh shell integration for zsh
# Add to ~/.zshrc or source from ~/.config/sesh/sesh.zsh

# Auto-suggest session when cd'ing to a project directory
_sesh_chpwd_hook() {
    if [[ -f ".sesh" ]] || [[ -f ".screen" ]]; then
        local session_name=$(basename "$PWD")
        if screen -ls 2>/dev/null | grep -q "\.$session_name[[:space:]]"; then
            print -P "%F{cyan}sesh:%f session '%B$session_name%b' exists. Press %BCtrl+s%b to attach."
        fi
    fi
}

# Add to chpwd hooks for zsh
autoload -Uz add-zsh-hook
add-zsh-hook chpwd _sesh_chpwd_hook

# Keybinding: Ctrl+s to launch sesh
bindkey -s '^s' 'sesh\n'

# sesh function wrapper
sesh() {
    if [[ "$1" == "attach" ]] && [[ -z "$2" ]]; then
        command sesh
    else
        command sesh "$@"
    fi
}

# Zsh completion for sesh
_sesh() {
    local -a commands sessions templates
    commands=(
        'list:List all screen sessions'
        'new:Create a new session'
        'attach:Attach to a session'
        'detach:Detach a session'
        'kill:Kill a session'
        'start:Create session from template'
        'templates:List available templates'
        'windows:Show windows in a session'
    )

    case "$words[2]" in
        attach|detach|kill|windows)
            sessions=(${(f)"$(screen -ls 2>/dev/null | grep -oP '\d+\.\K[^\s]+' 2>/dev/null)"})
            _describe 'session' sessions
            ;;
        start)
            templates=(${(f)"$(find ~/.config/sesh/templates -name '*.yaml' -o -name '*.yml' 2>/dev/null | xargs -I{} basename {} .yaml | sed 's/\.yml$//')"})
            _describe 'template' templates
            ;;
        *)
            _describe 'command' commands
            ;;
    esac
}
compdef _sesh sesh
"#
}

/// Generate shell integration script for fish
/// This is the most comprehensive integration, designed for fish shell users
pub fn fish_integration() -> &'static str {
    r#"
# sesh shell integration for fish
# Save to ~/.config/fish/conf.d/sesh.fish

# ============================================================
# Auto-suggest session when cd'ing to a project directory
# ============================================================
function __sesh_check_project --on-variable PWD
    if test -f ".sesh" -o -f ".screen"
        set -l session_name (basename $PWD)
        if screen -ls 2>/dev/null | string match -qr "\.$session_name\s"
            set_color cyan; echo -n "sesh: "; set_color normal
            echo -n "session '"; set_color --bold; echo -n "$session_name"; set_color normal
            echo -n "' exists. Press "; set_color --bold; echo -n "Ctrl+S"; set_color normal
            echo " to attach."
        end
    end
end

# ============================================================
# Keybinding: Ctrl+S to launch sesh TUI
# ============================================================
function __sesh_launch
    commandline -C 0
    commandline -r 'sesh'
    commandline -f execute
end

bind \cs __sesh_launch

# ============================================================
# Abbreviations for common sesh commands
# ============================================================
# These expand as you type, showing the full command in history

abbr --add ss 'sesh'                          # Launch TUI
abbr --add sl 'sesh list'                     # List sessions
abbr --add sn 'sesh new'                      # New session
abbr --add sa 'sesh attach'                   # Attach
abbr --add sd 'sesh detach'                   # Detach
abbr --add sk 'sesh kill'                     # Kill
abbr --add st 'sesh start'                    # Start from template
abbr --add sw 'sesh windows'                  # Show windows

# Screen shortcuts (for when you're inside screen)
abbr --add scr 'screen -r'                    # Reattach
abbr --add scls 'screen -ls'                  # List

# ============================================================
# Helper functions (autoloaded by fish)
# ============================================================

# Quick attach: sesh-attach [session]
function sesh-attach --description "Attach to a screen session"
    if test (count $argv) -eq 0
        sesh
    else
        sesh attach $argv[1]
    end
end

# Quick new: sesh-new <name> [directory]
function sesh-new --description "Create a new screen session"
    if test (count $argv) -eq 0
        echo "Usage: sesh-new <session-name> [directory]"
        return 1
    end

    set -l name $argv[1]
    set -l dir $argv[2]

    if test -n "$dir"
        sesh new $name --dir $dir
    else
        sesh new $name
    end
end

# Project session: creates or attaches to session named after current dir
function sesh-here --description "Create/attach to session for current directory"
    set -l session_name (basename $PWD)

    if screen -ls 2>/dev/null | string match -qr "\.$session_name\s"
        echo "Attaching to existing session: $session_name"
        sesh attach $session_name
    else
        echo "Creating new session: $session_name"
        sesh new $session_name
        sesh attach $session_name
    end
end

# ============================================================
# Completions for sesh
# ============================================================
function __fish_sesh_needs_command
    set -l cmd (commandline -opc)
    test (count $cmd) -eq 1
end

function __fish_sesh_using_command
    set -l cmd (commandline -opc)
    test (count $cmd) -gt 1 -a "$cmd[2]" = "$argv[1]"
end

function __fish_sesh_sessions
    screen -ls 2>/dev/null | string match -r '\d+\.(\S+)' | string replace -r '\d+\.' ''
end

function __fish_sesh_templates
    for f in ~/.config/sesh/templates/*.yaml ~/.config/sesh/templates/*.yml
        if test -f $f
            basename $f .yaml | string replace -r '\.yml$' ''
        end
    end
end

# Main commands
complete -c sesh -f
complete -c sesh -n __fish_sesh_needs_command -a list -d "List all screen sessions"
complete -c sesh -n __fish_sesh_needs_command -a new -d "Create a new session"
complete -c sesh -n __fish_sesh_needs_command -a attach -d "Attach to a session"
complete -c sesh -n __fish_sesh_needs_command -a detach -d "Detach a session"
complete -c sesh -n __fish_sesh_needs_command -a kill -d "Kill a session"
complete -c sesh -n __fish_sesh_needs_command -a start -d "Create session from template"
complete -c sesh -n __fish_sesh_needs_command -a templates -d "List available templates"
complete -c sesh -n __fish_sesh_needs_command -a windows -d "Show windows in a session"

# Session name completions
complete -c sesh -n "__fish_sesh_using_command attach" -a "(__fish_sesh_sessions)" -d "Session"
complete -c sesh -n "__fish_sesh_using_command detach" -a "(__fish_sesh_sessions)" -d "Session"
complete -c sesh -n "__fish_sesh_using_command kill" -a "(__fish_sesh_sessions)" -d "Session"
complete -c sesh -n "__fish_sesh_using_command windows" -a "(__fish_sesh_sessions)" -d "Session"

# Template completions
complete -c sesh -n "__fish_sesh_using_command start" -a "(__fish_sesh_templates)" -d "Template"

# Options
complete -c sesh -s c -l config -d "Custom config file path"
complete -c sesh -s H -l host -d "Connect to remote host"
complete -c sesh -s d -l debug -d "Enable debug logging"
"#
}

/// Detect the current shell
pub fn detect_shell() -> Option<String> {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(|s| s.to_string()))
}

/// Get integration script for current shell
pub fn get_integration_script() -> Option<&'static str> {
    detect_shell().and_then(|shell| match shell.as_str() {
        "bash" => Some(bash_integration()),
        "zsh" => Some(zsh_integration()),
        "fish" => Some(fish_integration()),
        _ => None,
    })
}

/// Check if a directory has a sesh marker file
pub fn has_sesh_marker(dir: &Path) -> bool {
    dir.join(".sesh").exists() || dir.join(".screen").exists()
}

/// Get suggested session name for a directory
pub fn get_suggested_session_name(dir: &Path) -> Option<String> {
    dir.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell() {
        // Just check it doesn't panic
        let _ = detect_shell();
    }

    #[test]
    fn test_integration_scripts() {
        assert!(!bash_integration().is_empty());
        assert!(!zsh_integration().is_empty());
        assert!(!fish_integration().is_empty());
    }
}

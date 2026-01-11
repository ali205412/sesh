%global crate sesh

Name:           sesh
Version:        0.1.0
Release:        1%{?dist}
Summary:        A fully-featured TUI manager for GNU Screen

License:        MIT
URL:            https://github.com/ali205412/sesh
Source0:        %{url}/archive/v%{version}/%{crate}-%{version}.tar.gz

BuildRequires:  cargo >= 1.70
BuildRequires:  rust >= 1.70
BuildRequires:  gcc
BuildRequires:  openssl-devel
BuildRequires:  libgit2-devel
BuildRequires:  pkgconfig

Requires:       screen

%description
sesh is a fully-featured Terminal User Interface for managing GNU Screen
sessions. It eliminates the need for manual screen commands and provides
fast, rich, keyboard-driven session management.

Features:
- Session management (list, create, attach, detach, kill)
- Window management within sessions
- Live terminal preview
- Session templates (YAML)
- SSH integration for remote sessions
- Git status integration
- Fish/Bash/Zsh shell integration
- fzf fuzzy finder integration

%prep
%autosetup -n %{crate}-%{version}

%build
cargo build --release

%install
install -Dm755 target/release/%{crate} %{buildroot}%{_bindir}/%{crate}

# Install shell completions
install -Dm644 -t %{buildroot}%{_datadir}/fish/vendor_completions.d/ \
    /dev/null 2>/dev/null || true

# Install config examples
install -Dm644 config/default.toml %{buildroot}%{_datadir}/%{crate}/config.toml.example
install -Dm644 templates/default.yaml %{buildroot}%{_datadir}/%{crate}/templates/default.yaml
install -Dm644 templates/webdev.yaml %{buildroot}%{_datadir}/%{crate}/templates/webdev.yaml

%check
cargo test --release

%files
%license LICENSE
%doc README.md
%{_bindir}/%{crate}
%{_datadir}/%{crate}/

%changelog
* Sat Jan 11 2025 Ali <ali205412@users.noreply.github.com> - 0.1.0-1
- Initial package release
- Full TUI for GNU Screen session management
- Support for local and remote sessions
- Template system for quick session creation
- Shell integrations for fish, bash, and zsh

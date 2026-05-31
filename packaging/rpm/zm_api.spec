# RPM spec for zm_api — covers Fedora / RHEL / Rocky / AlmaLinux and openSUSE.
# Build locally with:  rpmbuild -bb packaging/rpm/zm_api.spec  (after placing a
# source tarball in ~/rpmbuild/SOURCES), or submit to COPR / openSUSE OBS.

Name:           zm_api
Version:        3.0.0
# Pre-release ordering: 0.<n>.<prerel> sorts before the stable "1%{?dist}".
# For the stable 3.0.0, set this back to "1%{?dist}".
Release:        0.1.alpha1%{?dist}
Summary:        ZoneMinder REST API and daemon supervisor

License:        AGPL-3.0-or-later
URL:            https://github.com/SteveGilvarry/zm-api
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  systemd-rpm-macros
# FFmpeg + OpenSSL development libraries, requested via pkg-config virtual
# provides so the right package resolves on each distro (Fedora's *-free-devel,
# openSUSE's libav*-devel / Packman, etc.). ffmpeg-sys-next links these.
BuildRequires:  pkgconfig(libavcodec)
BuildRequires:  pkgconfig(libavformat)
BuildRequires:  pkgconfig(libavutil)
BuildRequires:  pkgconfig(libavfilter)
BuildRequires:  pkgconfig(libavdevice)
BuildRequires:  pkgconfig(libswscale)
BuildRequires:  pkgconfig(libswresample)
BuildRequires:  pkgconfig(openssl)

# useradd/usermod: shadow-utils on Fedora/EL, shadow on openSUSE.
%if 0%{?suse_version}
Requires:       shadow
%else
Requires:       shadow-utils
%endif
Requires:       openssl
Requires(post): systemd
Requires(preun): systemd
Requires(postun): systemd

%description
zm_api is a Rust REST API for managing a ZoneMinder installation. It can run
passively alongside stock ZoneMinder (REST API only) or, after disabling
zoneminder.service, take over supervision of the ZoneMinder daemons.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --locked

%install
install -D -m 0755 target/release/%{name}            %{buildroot}%{_bindir}/%{name}
install -D -m 0755 packaging/zm_api-takeover.sh       %{buildroot}%{_bindir}/%{name}-takeover
install -D -m 0755 packaging/setup-instance.sh        %{buildroot}%{_datadir}/%{name}/setup-instance.sh
install -D -m 0644 settings/base.toml                 %{buildroot}%{_sysconfdir}/%{name}/base.toml
install -D -m 0644 settings/prod.toml                 %{buildroot}%{_sysconfdir}/%{name}/prod.toml
install -D -m 0644 packaging/systemd/zm_api.env       %{buildroot}%{_sysconfdir}/%{name}/zm_api.env
install -D -m 0644 packaging/systemd/zm_api.service   %{buildroot}%{_unitdir}/%{name}.service
# NOTE: static/ is not packaged — it holds dev JWT keys. Per-install keys are
# generated into /var/lib/zm_api/keys by setup-instance.sh.

%post
# Provision user/dirs/JWT keys (idempotent), then register the unit. Ships in
# passive mode, so this never disturbs a running ZoneMinder.
[ -x %{_datadir}/%{name}/setup-instance.sh ] && %{_datadir}/%{name}/setup-instance.sh || :
%systemd_post %{name}.service
if [ $1 -eq 1 ] && [ -d /run/systemd/system ]; then
  systemctl enable --now %{name}.service || :
fi

%preun
%systemd_preun %{name}.service

%postun
%systemd_postun_with_restart %{name}.service

%files
%license LICENSE
%{_bindir}/%{name}
%{_bindir}/%{name}-takeover
%dir %{_datadir}/%{name}
%{_datadir}/%{name}/setup-instance.sh
%{_unitdir}/%{name}.service
%dir %{_sysconfdir}/%{name}
%config(noreplace) %{_sysconfdir}/%{name}/base.toml
%config(noreplace) %{_sysconfdir}/%{name}/prod.toml
%config(noreplace) %{_sysconfdir}/%{name}/zm_api.env

%changelog
* Sat May 31 2026 Steve Gilvarry <SteveGilvarry@users.noreply.github.com> - 3.0.0-0.1.alpha1
- First Rust release (3.0.0-alpha.1). Passive by default; zm_api-takeover for
  daemon control. Initial RPM packaging.

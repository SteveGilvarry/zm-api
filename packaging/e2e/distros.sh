# shellcheck shell=bash
# Distro table for the packaging end-to-end tests. Sourced by build.sh and
# run.sh; sets the E2E_* variables for one distro name.
#
#   E2E_IMAGE    base container image
#   E2E_FAMILY   package format the distro installs: deb | rpm | arch
#   E2E_ZM       where a ZoneMinder master build comes from:
#                  zmrepo-deb  https://zmrepo.zoneminder.com/debian/master
#                  zmrepo-rpm  https://zmrepo.zoneminder.com/{el,fc}/<n>
#                  none        upstream publishes no master package here
#   E2E_WEB_USER / E2E_WEB_GROUP
#                the account ZoneMinder's own packaging runs its daemons as on
#                this distro (upstream CMakeLists.txt and distros/*). The
#                install must end up running zm-api as this account.
#
# A distro with E2E_ZM=none still gets the package smoke test (install, fresh
# database, account resolution, upgrade, removal). Only the passive, takeover
# and revert stages need a real ZoneMinder.

e2e_distro() {
  case "$1" in
    ubuntu-24.04)
      E2E_IMAGE=ubuntu:24.04 E2E_FAMILY=deb E2E_ZM=zmrepo-deb
      E2E_WEB_USER=www-data E2E_WEB_GROUP=www-data ;;
    ubuntu-22.04)
      E2E_IMAGE=ubuntu:22.04 E2E_FAMILY=deb E2E_ZM=zmrepo-deb
      E2E_WEB_USER=www-data E2E_WEB_GROUP=www-data ;;
    debian-13)
      E2E_IMAGE=debian:13 E2E_FAMILY=deb E2E_ZM=zmrepo-deb
      E2E_WEB_USER=www-data E2E_WEB_GROUP=www-data ;;
    debian-12)
      E2E_IMAGE=debian:12 E2E_FAMILY=deb E2E_ZM=zmrepo-deb
      E2E_WEB_USER=www-data E2E_WEB_GROUP=www-data ;;
    rocky-9)
      E2E_IMAGE=rockylinux/rockylinux:9 E2E_FAMILY=rpm E2E_ZM=zmrepo-rpm
      E2E_WEB_USER=apache E2E_WEB_GROUP=apache ;;
    fedora-44)
      # zmrepo's Fedora builds stopped at 1.39.23, before the stream socket
      # landed, so Fedora gets the smoke test only.
      E2E_IMAGE=fedora:44 E2E_FAMILY=rpm E2E_ZM=none
      E2E_WEB_USER=apache E2E_WEB_GROUP=apache ;;
    opensuse-tumbleweed)
      E2E_IMAGE=opensuse/tumbleweed:latest E2E_FAMILY=rpm E2E_ZM=none
      E2E_WEB_USER=wwwrun E2E_WEB_GROUP=www ;;
    arch)
      E2E_IMAGE=archlinux:latest E2E_FAMILY=arch E2E_ZM=none
      E2E_WEB_USER=http E2E_WEB_GROUP=http ;;
    *)
      echo "unknown distro '$1'. Known: $(e2e_distros)" >&2
      return 1 ;;
  esac
}

e2e_distros() {
  echo "ubuntu-24.04 ubuntu-22.04 debian-13 debian-12 rocky-9 fedora-44 opensuse-tumbleweed arch"
}

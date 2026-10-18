#!/usr/bin/env bash
# A local MariaDB, as a single-host ZoneMinder install has.
. /e2e/lib.sh

section "MariaDB"
case "$E2E_FAMILY" in
  deb)
    apt-get update -qq
    sys_install mariadb-server >/dev/null ;;
  rpm)
    if command -v zypper >/dev/null 2>&1; then
      sys_install mariadb mariadb-client >/dev/null
    else
      sys_install mariadb-server >/dev/null
    fi ;;
  arch)
    sys_install mariadb >/dev/null
    mariadb-install-db --user=mysql --basedir=/usr --datadir=/var/lib/mysql >/dev/null ;;
esac

unit=mariadb.service
systemctl cat "$unit" >/dev/null 2>&1 || unit=mysql.service
systemctl enable --now "$unit"
check "database server is running" wait_until 60 svc_active "$unit"

finish

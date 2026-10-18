#!/usr/bin/env bash
# For distros where upstream publishes no ZoneMinder master package: lay out
# what its packaging would have created that the zm-api install depends on.
# That is the web server's account, an empty database with ZoneMinder's
# credentials, and a zm.conf with the ownership upstream gives it. No
# ZoneMinder daemons run, so this tests the package and its account handling,
# not capture.
. /e2e/lib.sh

section "Web server account (${E2E_WEB_USER}:${E2E_WEB_GROUP})"
case "$E2E_FAMILY" in
  deb)  : ;;  # www-data is in base-passwd
  rpm)
    if command -v zypper >/dev/null 2>&1; then
      sys_install apache2 >/dev/null
    else
      sys_install httpd >/dev/null
    fi ;;
  arch) : ;;  # http is in the filesystem package
esac
check "account exists" id "$E2E_WEB_USER"

section "Database and zm.conf"
client=mariadb; command -v mariadb >/dev/null 2>&1 || client=mysql
"$client" -uroot -e "CREATE DATABASE zm;
  CREATE USER 'zmuser'@'localhost' IDENTIFIED BY 'zmpass';
  GRANT ALL PRIVILEGES ON zm.* TO 'zmuser'@'localhost'; FLUSH PRIVILEGES;"

install -d -m 0755 /etc/zm/conf.d
cat >/etc/zm/zm.conf <<CONF
ZM_DB_TYPE=mysql
ZM_DB_HOST=localhost
ZM_DB_NAME=zm
ZM_DB_USER=zmuser
ZM_DB_PASS=zmpass
ZM_WEB_USER=${E2E_WEB_USER}
ZM_WEB_GROUP=${E2E_WEB_GROUP}
CONF
cat >/etc/zm/conf.d/01-system-paths.conf <<CONF
ZM_PATH_SOCKS=/run/zm
ZM_PATH_BIN=/usr/bin
CONF
# Upstream's ownership: Debian gives the files to the web user and closes them
# to its group, the RPM spec keeps root as owner and opens them to the group.
# Either way only ZoneMinder's account can read the database password.
if [[ "$E2E_FAMILY" == "deb" ]]; then
  chown -R "${E2E_WEB_USER}:root" /etc/zm
else
  chown -R "root:${E2E_WEB_GROUP}" /etc/zm
fi
chmod 0640 /etc/zm/zm.conf /etc/zm/conf.d/*.conf

id "$E2E_WEB_USER" >"$STATE/web-user.before"
finish

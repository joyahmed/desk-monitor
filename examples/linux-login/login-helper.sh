#!/usr/bin/env bash
# Desk Monitor "Log in" helper for a Linux client with GDM (tested on Ubuntu 26.04, Wayland).
# Logs the ssh user in on the machine's own screen, or unlocks it - but only after checking that
# user's password with PAM. Desk Monitor sends the password as line 1 of stdin over ssh; it never
# appears on a command line. Autologin is switched on only for the one login, and the fail-safe
# (autologin-failsafe.sh as a gdm ExecStartPre) switches it off at any other gdm start, so a power
# cut mid-login still boots to the password screen.
#
# Install:  sudo install -m 755 login-helper.sh /usr/local/sbin/desk-login
#           sudo install -m 755 autologin-failsafe.sh /usr/local/sbin/desk-autologin-failsafe
#           sudo install -D -m 644 gdm-autologin-failsafe.conf /etc/systemd/system/gdm.service.d/desk-autologin-failsafe.conf
#           sudo systemctl daemon-reload
#           sudoers: <user> ALL=(root) NOPASSWD: /usr/local/sbin/desk-login
# In Desk Monitor, set the machine's login_command to:  sudo -n /usr/local/sbin/desk-login
# Exit codes: 0 logged in / unlocked, 2 no password, 3 wrong password, 4 another login running.
set -u
U=${SUDO_USER:?run me through sudo}; CONF=/etc/gdm3/custom.conf

# First line of stdin = the password. PAM's own helper checks it as that user; it never goes on a command line.
IFS= read -r PW || true
PW=${PW%$'\r'}   # Windows PowerShell ends a piped line with CRLF
if [ -z "${PW:-}" ]; then echo "No password given - nothing changed."; exit 2; fi
if ! printf '%s\0' "$PW" | runuser -u "$U" -- /usr/sbin/unix_chkpwd "$U" nonull; then
  unset PW; sleep 2; echo "Wrong password - nothing changed."; exit 3
fi
unset PW
# The desktop icon and the app at once would restart gdm under each other's fresh login.
exec 9>/run/desk-login.lock
flock -n 9 || { echo "Another log in is already running - wait a minute and look at the server screen."; exit 4; }
FLAG=/run/desk-autologin-once
off(){ sed -i 's/^AutomaticLoginEnable=.*/AutomaticLoginEnable=False/' "$CONF"; rm -f "$FLAG"; }
trap off EXIT
monitor_session(){ loginctl list-sessions --no-legend | awk -v u="$U" '$3==u && $4=="seat0" {print $1}'; }

s=$(monitor_session)
if [ -n "$s" ]; then
  loginctl unlock-session "$s"
  echo "Already logged in on the server monitor - unlocked."; exit 0
fi
# GNOME refuses a second session for the same user, so the RDP one goes first.
for s in $(loginctl list-sessions --no-legend | awk -v u="$U" '$3==u {print $1}'); do
  if [ "$(loginctl show-session "$s" -p Service --value)" = xrdp-sesman ]; then
    echo "Closing the RDP session ($s) - one login at a time."; loginctl terminate-session "$s"
  fi
done
sleep 3
# A package upgrade can replace custom.conf and drop these lines (Ubuntu 26.04, 2026-10-12): put them back.
grep -q '^AutomaticLoginEnable=' "$CONF" || sed -i "/^\[daemon\]/a AutomaticLoginEnable=False" "$CONF"
grep -q '^AutomaticLogin=' "$CONF" || sed -i "/^\[daemon\]/a AutomaticLogin=$U" "$CONF"
touch "$FLAG"   # tells the fail-safe that the next gdm start is this login
sed -i 's/^AutomaticLoginEnable=.*/AutomaticLoginEnable=True/; s/^AutomaticLogin=.*/AutomaticLogin='"$U"'/' "$CONF"
systemctl restart gdm3
for i in $(seq 1 30); do
  [ -n "$(monitor_session)" ] && { echo "Logged in on the server monitor."; exit 0; }
  sleep 1
done
echo "Login did not appear within 30s - check: journalctl -u gdm3 -n 30"; exit 1

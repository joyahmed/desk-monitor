#!/bin/sh
# gdm ExecStartPre (gdm-autologin-failsafe.conf). Autologin may be ON only for the one gdm start that
# login-helper.sh asked for, after checking the user's password; it leaves /run/desk-autologin-once for
# that start. Any other start - a boot (/run is empty), a power cut or kill mid-login, a manual
# restart - forces it OFF.
FLAG=/run/desk-autologin-once
if [ -e "$FLAG" ]; then
  rm -f "$FLAG"
else
  sed -i 's/^AutomaticLoginEnable=.*/AutomaticLoginEnable=False/' /etc/gdm3/custom.conf
fi
exit 0
